#![cfg(feature = "zk-halo2-kzg")]

use crate::{core::verifier::ProofVerifier, engine::PublicInputs};
use halo2_proofs_kzg::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector,
        VerifyingKey, keygen_vk, verify_proof,
    },
    poly::{
        Rotation,
        commitment::{Params as _, ParamsProver},
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::VerifierSHPLONK,
            strategy::AccumulatorStrategy,
        },
    },
    transcript::{Blake2bRead, Challenge255, TranscriptReadBuffer},
};
use halo2curves_kzg::{
    bn256::{Bn256, Fr, G1Affine},
    ff::PrimeField,
};
use zkcg_common::errors::ProtocolError;

#[cfg(test)]
use halo2_proofs_kzg::{
    plonk::{create_proof, keygen_pk},
    poly::kzg::multiopen::ProverSHPLONK,
    transcript::{Blake2bWrite, TranscriptWriterBuffer},
};
#[cfg(test)]
use rand::rngs::OsRng;

pub const HALO2_KZG_SYSTEM_NAME: &str = "halo2-kzg";
const DIFF_BITS: usize = 16;
const HALO2_KZG_K: u32 = 6;

#[derive(Clone)]
struct ScoreCircuit<F: PrimeField> {
    score: Value<F>,
    threshold: Value<F>,
}

#[derive(Clone, Debug)]
struct ScoreConfig {
    score: Column<Advice>,
    diff: Column<Advice>,
    bits: Column<Advice>,
    threshold: Column<Instance>,
    selector: Selector,
}

impl<F: PrimeField> Circuit<F> for ScoreCircuit<F> {
    type Config = ScoreConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            score: Value::unknown(),
            threshold: Value::unknown(),
        }
    }

    fn configure(cs: &mut ConstraintSystem<F>) -> Self::Config {
        let score = cs.advice_column();
        let diff = cs.advice_column();
        let bits = cs.advice_column();
        let threshold = cs.instance_column();
        let selector = cs.selector();

        cs.enable_equality(score);
        cs.enable_equality(diff);
        cs.enable_equality(threshold);

        cs.create_gate("bits are boolean", |meta| {
            let s = meta.query_selector(selector);
            let mut constraints = vec![];
            for i in 0..DIFF_BITS {
                let bit = meta.query_advice(bits, Rotation(i as i32));
                constraints.push(s.clone() * bit.clone() * (Expression::Constant(F::ONE) - bit));
            }
            constraints
        });

        cs.create_gate("threshold = score + diff", |meta| {
            let s = meta.query_selector(selector);
            let score_expr = meta.query_advice(score, Rotation::cur());
            let diff_expr = meta.query_advice(diff, Rotation::cur());
            let threshold_expr = meta.query_advice(score, Rotation::next());

            vec![s * (threshold_expr - score_expr - diff_expr)]
        });

        cs.create_gate("diff reconstruction", |meta| {
            let s = meta.query_selector(selector);
            let diff_expr = meta.query_advice(diff, Rotation::cur());

            let mut reconstructed = Expression::Constant(F::ZERO);
            for i in 0..DIFF_BITS {
                let bit = meta.query_advice(bits, Rotation(i as i32));
                reconstructed =
                    reconstructed + bit * Expression::Constant(F::from_u128(1u128 << i));
            }

            vec![s * (diff_expr - reconstructed)]
        });

        ScoreConfig {
            score,
            diff,
            bits,
            threshold,
            selector,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let threshold_cell = layouter.assign_region(
            || "score <= threshold",
            |mut region| {
                config.selector.enable(&mut region, 0)?;

                region.assign_advice(|| "score", config.score, 0, || self.score)?;

                let diff_value = self.threshold.zip(self.score).map(|(t, s)| t - s);

                region.assign_advice(|| "diff", config.diff, 0, || diff_value)?;
                let threshold_cell = region.assign_advice(
                    || "threshold advice",
                    config.score,
                    1,
                    || self.threshold,
                )?;

                let bits_values = diff_value.map(|diff| {
                    let mut bits = [false; DIFF_BITS];
                    let bytes = diff.to_repr();
                    let mut acc = 0u64;
                    for (j, b) in bytes.as_ref().iter().take(8).enumerate() {
                        acc |= (*b as u64) << (8 * j);
                    }
                    for i in 0..DIFF_BITS {
                        bits[i] = ((acc >> i) & 1) == 1;
                    }
                    bits
                });

                for i in 0..DIFF_BITS {
                    region.assign_advice(
                        || format!("diff bit {}", i),
                        config.bits,
                        i,
                        || bits_values.map(|bits| F::from(bits[i] as u64)),
                    )?;
                }

                Ok(threshold_cell)
            },
        )?;

        layouter.constrain_instance(threshold_cell.cell(), config.threshold, 0)?;

        Ok(())
    }
}

pub struct Halo2KzgVerifier {
    pub vk: VerifyingKey<G1Affine>,
    pub params: ParamsKZG<Bn256>,
}

impl Halo2KzgVerifier {
    pub fn bundled() -> Self {
        let params = ParamsKZG::<Bn256>::new(HALO2_KZG_K);
        let vk = keygen_vk(&params, &empty_score_circuit())
            .expect("kzg verifying key generation failed");

        Self { vk, params }
    }
}

impl ProofVerifier for Halo2KzgVerifier {
    fn verify(
        &self,
        proof_bytes: &[u8],
        public_inputs: &PublicInputs,
    ) -> Result<(), ProtocolError> {
        let threshold = Fr::from(
            public_inputs
                .phase1()
                .ok_or(ProtocolError::InvalidProof)?
                .threshold,
        );
        let instance_values = vec![vec![threshold]];
        let instance_slices: Vec<&[Fr]> = instance_values.iter().map(|v| v.as_slice()).collect();
        let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];

        let verifier_params = self.params.verifier_params();
        let strategy = AccumulatorStrategy::new(verifier_params);
        let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<G1Affine>>::init(proof_bytes);

        verify_proof::<
            KZGCommitmentScheme<Bn256>,
            VerifierSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
            AccumulatorStrategy<'_, Bn256>,
        >(
            verifier_params,
            &self.vk,
            strategy,
            &all_instances,
            &mut transcript,
            verifier_params.n(),
        )
        .map_err(|_| ProtocolError::InvalidProof)?;

        Ok(())
    }
}

fn empty_score_circuit() -> ScoreCircuit<Fr> {
    ScoreCircuit {
        score: Value::unknown(),
        threshold: Value::unknown(),
    }
}

#[cfg(test)]
pub(crate) fn generate_test_proof(score: u64, threshold: u64) -> Vec<u8> {
    let params = ParamsKZG::<Bn256>::new(HALO2_KZG_K);
    let empty = empty_score_circuit();
    let vk = keygen_vk(&params, &empty).expect("kzg verifying key generation failed");
    let pk = keygen_pk(&params, vk, &empty).expect("kzg proving key generation failed");

    let circuit = ScoreCircuit::<Fr> {
        score: Value::known(Fr::from(score)),
        threshold: Value::known(Fr::from(threshold)),
    };

    let public_inputs = vec![vec![Fr::from(threshold)]];
    let instance_slices: Vec<&[Fr]> = public_inputs
        .iter()
        .map(|values| values.as_slice())
        .collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];

    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(vec![]);

    create_proof::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<'_, Bn256>, _, _, _, _>(
        &params,
        &pk,
        &[circuit],
        &all_instances,
        OsRng,
        &mut transcript,
    )
    .expect("kzg proof generation failed");

    transcript.finalize()
}
