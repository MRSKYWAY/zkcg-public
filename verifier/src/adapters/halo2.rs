#![cfg(feature = "zk-halo2")]

use crate::{core::verifier::ProofVerifier, engine::PublicInputs};
use halo2_proofs::plonk::keygen_vk;
use halo2_proofs::{
    arithmetic::Field,
    plonk::{SingleVerifier, VerifyingKey, verify_proof},
    poly::commitment::Params,
    transcript::{Blake2bRead, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use zkcg_circuits::{
    halo2_artifacts::verifier_artifacts,
    rwa_circuit::{
        ONBOARDING_INSTANCE_LEN, RWA_ONBOARDING_K, RWA_TRANSFER_K, RwaOnboardingCircuit,
        RwaTransferCircuit, TRANSFER_INSTANCE_LEN, onboarding_instance_values,
        transfer_instance_values,
    },
};
use zkcg_common::errors::ProtocolError;
use zkcg_halo2_prover::{verify_bulk_payout_round_proof, verify_payout_release_proof};

/// Real Halo2 verifier adapter (runtime keys, KZG implicit).
struct CpuHalo2Verifier {
    pub score_vk: VerifyingKey<G1Affine>,
    pub score_params: Params<G1Affine>,
    pub rwa_onboarding_vk: VerifyingKey<G1Affine>,
    pub rwa_onboarding_params: Params<G1Affine>,
    pub rwa_transfer_vk: VerifyingKey<G1Affine>,
    pub rwa_transfer_params: Params<G1Affine>,
}

impl CpuHalo2Verifier {
    pub fn bundled() -> Self {
        let artifacts = verifier_artifacts();
        let rwa_onboarding_params = Params::<G1Affine>::new(RWA_ONBOARDING_K);
        let rwa_onboarding_vk = keygen_vk(
            &rwa_onboarding_params,
            &RwaOnboardingCircuit::<Fr> {
                public_values: vec![Fr::ZERO; ONBOARDING_INSTANCE_LEN],
            },
        )
        .expect("rwa onboarding verifying key generation failed");
        let rwa_transfer_params = Params::<G1Affine>::new(RWA_TRANSFER_K);
        let rwa_transfer_vk = keygen_vk(
            &rwa_transfer_params,
            &RwaTransferCircuit::<Fr> {
                public_values: vec![Fr::ZERO; TRANSFER_INSTANCE_LEN],
            },
        )
        .expect("rwa transfer verifying key generation failed");

        Self {
            score_vk: artifacts.vk,
            score_params: artifacts.params,
            rwa_onboarding_vk,
            rwa_onboarding_params,
            rwa_transfer_vk,
            rwa_transfer_params,
        }
    }

    fn verify_score(&self, proof_bytes: &[u8], threshold: u64) -> Result<(), ProtocolError> {
        let threshold = Fr::from(threshold);
        let instance_values = vec![vec![threshold]];
        verify_instances(
            &self.score_params,
            &self.score_vk,
            proof_bytes,
            instance_values,
        )
    }

    fn verify_rwa_onboarding(
        &self,
        proof_bytes: &[u8],
        claims: &zkcg_common::types::RwaCreditOnboardingClaims,
    ) -> Result<(), ProtocolError> {
        if !claims.identifiers_match() {
            return Err(ProtocolError::InvalidProof);
        }

        verify_instances(
            &self.rwa_onboarding_params,
            &self.rwa_onboarding_vk,
            proof_bytes,
            vec![onboarding_instance_values::<Fr>(claims)],
        )
    }

    fn verify_rwa_transfer(
        &self,
        proof_bytes: &[u8],
        claims: &zkcg_common::types::RwaCreditTransferClaims,
    ) -> Result<(), ProtocolError> {
        if !claims.identifiers_match() {
            return Err(ProtocolError::InvalidProof);
        }

        verify_instances(
            &self.rwa_transfer_params,
            &self.rwa_transfer_vk,
            proof_bytes,
            vec![transfer_instance_values::<Fr>(claims)],
        )
    }
}

impl ProofVerifier for CpuHalo2Verifier {
    fn verify(
        &self,
        proof_bytes: &[u8],
        public_inputs: &PublicInputs,
    ) -> Result<(), ProtocolError> {
        match public_inputs {
            PublicInputs::Phase1ScoreV1(claims) => self.verify_score(proof_bytes, claims.threshold),
            PublicInputs::RwaCreditOnboardingV1(claims) => {
                self.verify_rwa_onboarding(proof_bytes, claims)
            }
            PublicInputs::RwaCreditTransferV1(claims) => {
                self.verify_rwa_transfer(proof_bytes, claims)
            }
            PublicInputs::BulkPayoutRoundV1(claims) => {
                verify_bulk_payout_round_proof(proof_bytes, claims)
                    .map_err(|_| ProtocolError::InvalidProof)
            }
            PublicInputs::PayoutReleaseV1(claims) => {
                verify_payout_release_proof(proof_bytes, claims)
                    .map_err(|_| ProtocolError::InvalidProof)
            }
        }
    }
}

pub struct Halo2Verifier {
    cpu: CpuHalo2Verifier,
    #[cfg(feature = "zk-halo2-kzg")]
    kzg: crate::adapters::halo2_kzg::Halo2KzgVerifier,
}

impl Halo2Verifier {
    pub fn bundled() -> Self {
        Self {
            cpu: CpuHalo2Verifier::bundled(),
            #[cfg(feature = "zk-halo2-kzg")]
            kzg: crate::adapters::halo2_kzg::Halo2KzgVerifier::bundled(),
        }
    }

    pub fn from_cpu_parts(vk: VerifyingKey<G1Affine>, params: Params<G1Affine>) -> Self {
        Self {
            cpu: CpuHalo2Verifier {
                score_vk: vk,
                score_params: params,
                ..CpuHalo2Verifier::bundled()
            },
            #[cfg(feature = "zk-halo2-kzg")]
            kzg: crate::adapters::halo2_kzg::Halo2KzgVerifier::bundled(),
        }
    }
}

impl ProofVerifier for Halo2Verifier {
    fn verify(
        &self,
        proof_bytes: &[u8],
        public_inputs: &PublicInputs,
    ) -> Result<(), ProtocolError> {
        if self.cpu.verify(proof_bytes, public_inputs).is_ok() {
            return Ok(());
        }

        #[cfg(feature = "zk-halo2-kzg")]
        if matches!(public_inputs, PublicInputs::Phase1ScoreV1(_))
            && self.kzg.verify(proof_bytes, public_inputs).is_ok()
        {
            return Ok(());
        }

        Err(ProtocolError::InvalidProof)
    }
}

fn verify_instances(
    params: &Params<G1Affine>,
    vk: &VerifyingKey<G1Affine>,
    proof_bytes: &[u8],
    instance_values: Vec<Vec<Fr>>,
) -> Result<(), ProtocolError> {
    let instance_slices: Vec<&[Fr]> = instance_values.iter().map(|v| v.as_slice()).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<G1Affine>>::init(proof_bytes);
    let strategy = SingleVerifier::new(params);

    verify_proof(params, vk, strategy, &all_instances, &mut transcript)
        .map_err(|_| ProtocolError::InvalidProof)
}
