#![cfg(feature = "zk-halo2")]

//! Phase 8 exercises circuit-soundness mutations end-to-end with the real Halo2
//! prover/verifier APIs. The circuits are deliberately tiny and self-contained:
//! they are mutation benchmarks, not production ZKCG circuits.

use std::io::Cursor;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Constraints, Error, Expression, Instance, Selector, SingleVerifier, create_proof, keygen_pk, keygen_vk, verify_proof},
    poly::{Rotation, commitment::Params},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use rand::rngs::OsRng;

const PHASE8_K: u32 = 5;
const INSTANCE_LEN: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MutationKind { MissingCopyConstraint, MissingEqualityConstraint, MissingBooleanConstraint, SelectorOmission, MissingOutputBinding }

#[derive(Clone, Debug)]
struct Phase8Config { advice: Column<Advice>, instance: Column<Instance>, square_selector: Selector, multiply_selector: Selector, boolean_selector: Selector }

#[derive(Clone)]
struct MutationCircuit { kind: MutationKind, mutated: bool, witnesses: Vec<Fr> }

impl Circuit<Fr> for MutationCircuit {
    type Config = Phase8Config;
    type FloorPlanner = SimpleFloorPlanner;
    fn without_witnesses(&self) -> Self { Self { kind: self.kind, mutated: self.mutated, witnesses: vec![Fr::ZERO; self.witnesses.len()] } }
    fn configure(cs: &mut ConstraintSystem<Fr>) -> Self::Config {
        let advice = cs.advice_column(); let instance = cs.instance_column();
        let square_selector = cs.selector(); let multiply_selector = cs.selector(); let boolean_selector = cs.selector();
        cs.enable_equality(advice); cs.enable_equality(instance);
        cs.create_gate("square relation", |meta| { let s = meta.query_selector(square_selector); let value = meta.query_advice(advice, Rotation::cur()); let output = meta.query_advice(advice, Rotation(1)); Constraints::with_selector(s, [output - value.clone() * value]) });
        cs.create_gate("multiply relation", |meta| { let s = meta.query_selector(multiply_selector); let left = meta.query_advice(advice, Rotation::cur()); let right = meta.query_advice(advice, Rotation(1)); let output = meta.query_advice(advice, Rotation(2)); Constraints::with_selector(s, [output - left * right]) });
        cs.create_gate("boolean relation", |meta| { let s = meta.query_selector(boolean_selector); let value = meta.query_advice(advice, Rotation::cur()); Constraints::with_selector(s, [value.clone() * (Expression::Constant(Fr::ONE) - value)]) });
        Phase8Config { advice, instance, square_selector, multiply_selector, boolean_selector }
    }
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fr>) -> Result<(), Error> {
        let cells = layouter.assign_region(|| "phase 8 mutation benchmark", |mut region| {
            let mut cells = Vec::with_capacity(self.witnesses.len());
            for (row, witness) in self.witnesses.iter().enumerate() { let cell = region.assign_advice(|| format!("witness {row}"), config.advice, row, || Value::known(*witness))?; cells.push(cell.cell()); }
            match self.kind {
                MutationKind::MissingCopyConstraint => { config.square_selector.enable(&mut region, 0)?; }
                MutationKind::MissingEqualityConstraint => { config.multiply_selector.enable(&mut region, 0)?; }
                MutationKind::MissingBooleanConstraint => { if !self.mutated { config.boolean_selector.enable(&mut region, 0)?; } }
                MutationKind::SelectorOmission => { if !self.mutated { config.multiply_selector.enable(&mut region, 0)?; } }
                MutationKind::MissingOutputBinding => { config.square_selector.enable(&mut region, 0)?; }
            }
            if self.kind == MutationKind::MissingEqualityConstraint && !self.mutated { region.constrain_equal(cells[1], cells[3])?; }
            Ok(cells)
        })?;
        match self.kind {
            MutationKind::MissingCopyConstraint => { if !self.mutated { layouter.constrain_instance(cells[0], config.instance, 0)?; } layouter.constrain_instance(cells[1], config.instance, 1)?; }
            MutationKind::MissingEqualityConstraint => { layouter.constrain_instance(cells[0], config.instance, 0)?; layouter.constrain_instance(cells[3], config.instance, 1)?; layouter.constrain_instance(cells[2], config.instance, 2)?; }
            MutationKind::MissingBooleanConstraint => { layouter.constrain_instance(cells[0], config.instance, 0)?; }
            MutationKind::SelectorOmission => { layouter.constrain_instance(cells[0], config.instance, 0)?; layouter.constrain_instance(cells[1], config.instance, 1)?; layouter.constrain_instance(cells[2], config.instance, 2)?; }
            MutationKind::MissingOutputBinding => { layouter.constrain_instance(cells[0], config.instance, 0)?; if !self.mutated { layouter.constrain_instance(cells[1], config.instance, 1)?; } }
        }
        Ok(())
    }
}

fn scenario(kind: MutationKind, mutated: bool) -> (MutationCircuit, Vec<Fr>) {
    let (witnesses, instances) = match kind {
        MutationKind::MissingCopyConstraint => (vec![Fr::from(2u64), Fr::from(4u64), Fr::ZERO, Fr::ZERO], vec![Fr::from(3u64), Fr::from(4u64), Fr::ZERO, Fr::ZERO]),
        MutationKind::MissingEqualityConstraint => (vec![Fr::from(2u64), Fr::from(4u64), Fr::from(8u64), Fr::from(3u64)], vec![Fr::from(2u64), Fr::from(3u64), Fr::from(8u64), Fr::ZERO]),
        MutationKind::MissingBooleanConstraint => (vec![Fr::from(2u64), Fr::ZERO, Fr::ZERO, Fr::ZERO], vec![Fr::from(2u64), Fr::ZERO, Fr::ZERO, Fr::ZERO]),
        MutationKind::SelectorOmission => (vec![Fr::from(2u64), Fr::from(3u64), Fr::from(7u64), Fr::ZERO], vec![Fr::from(2u64), Fr::from(3u64), Fr::from(7u64), Fr::ZERO]),
        MutationKind::MissingOutputBinding => (vec![Fr::from(2u64), Fr::from(4u64), Fr::ZERO, Fr::ZERO], vec![Fr::from(2u64), Fr::from(5u64), Fr::ZERO, Fr::ZERO]),
    };
    (MutationCircuit { kind, mutated, witnesses }, instances)
}

fn prove_and_verify(kind: MutationKind, mutated: bool) -> bool {
    let (circuit, instances) = scenario(kind, mutated);
    let params = Params::<G1Affine>::new(PHASE8_K);
    let vk = match keygen_vk(&params, &circuit) { Ok(vk) => vk, Err(_) => return false };
    let pk = match keygen_pk(&params, vk.clone(), &circuit) { Ok(pk) => pk, Err(_) => return false };
    assert_eq!(instances.len(), INSTANCE_LEN);
    let instance_slices: Vec<&[Fr]> = vec![instances.as_slice()];
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut prover_transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());
    if create_proof(&params, &pk, &[circuit], &all_instances, OsRng, &mut prover_transcript).is_err() { return false; }
    let proof = prover_transcript.finalize();
    let mut reader = Cursor::new(proof.as_slice());
    let mut verifier_transcript = Blake2bRead::<_, G1Affine, Challenge255<G1Affine>>::init(&mut reader);
    let strategy = SingleVerifier::new(&params);
    if verify_proof(&params, &vk, strategy, &all_instances, &mut verifier_transcript).is_err() { return false; }
    reader.position() == proof.len() as u64
}

fn assert_mutation_is_detectable(kind: MutationKind, label: &str) {
    assert!(!prove_and_verify(kind, false), "{label}: correct circuit unexpectedly accepted the malicious witness");
    assert!(prove_and_verify(kind, true), "{label}: deliberately weakened circuit did not expose the expected soundness regression");
}

#[test] fn phase8_missing_copy_constraint() { assert_mutation_is_detectable(MutationKind::MissingCopyConstraint, "missing copy constraint"); }
#[test] fn phase8_missing_equality_constraint() { assert_mutation_is_detectable(MutationKind::MissingEqualityConstraint, "missing equality constraint"); }
#[test] fn phase8_missing_boolean_constraint() { assert_mutation_is_detectable(MutationKind::MissingBooleanConstraint, "missing boolean/range constraint"); }
#[test] fn phase8_selector_omission() { assert_mutation_is_detectable(MutationKind::SelectorOmission, "selector omission"); }
#[test] fn phase8_missing_output_binding() { assert_mutation_is_detectable(MutationKind::MissingOutputBinding, "missing output binding"); }
#[test]
fn phase8_mutation_matrix_has_expected_shape() {
    let cases = [MutationKind::MissingCopyConstraint, MutationKind::MissingEqualityConstraint, MutationKind::MissingBooleanConstraint, MutationKind::SelectorOmission, MutationKind::MissingOutputBinding];
    for kind in cases { assert!(!prove_and_verify(kind, false)); assert!(prove_and_verify(kind, true)); }
}
