#![cfg(feature = "zk-halo2")]

use crate::{Proof, ProofSystem, Verifier, engine::PublicInputs};
use halo2_proofs::{
    arithmetic::Field,
    plonk::{create_proof, keygen_pk, keygen_vk},
    poly::commitment::Params,
    transcript::{Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Fr, G1Affine};
use rand::rngs::OsRng;
use zkcg_circuits::rwa_circuit::{
    ONBOARDING_INSTANCE_LEN, RWA_ONBOARDING_K, RWA_TRANSFER_K, RwaOnboardingCircuit,
    RwaTransferCircuit, TRANSFER_INSTANCE_LEN, onboarding_instance_values,
    transfer_instance_values,
};
use zkcg_common::{
    rwa::{evaluate_rwa_credit_onboarding_v1, evaluate_rwa_credit_transfer_v1},
    types::{
        InvestorTypeCode, RwaCreditOnboardingClaims, RwaCreditOnboardingFacts,
        RwaCreditTransferClaims, RwaCreditTransferFacts,
    },
};

#[cfg(feature = "zk-vm")]
use zkcg_zkvm_host::{prove_rwa_onboarding, prove_rwa_transfer};

fn sample_onboarding_claims() -> RwaCreditOnboardingClaims {
    let facts = RwaCreditOnboardingFacts {
        issuer_id_hash: [1u8; 32],
        asset_id_hash: [2u8; 32],
        wallet_address: [3u8; 20],
        investor_type: InvestorTypeCode::Accredited,
        attestation_expired: false,
        accredited: true,
        kyc_passed: true,
        aml_cleared: true,
        sanctions_clear: true,
        jurisdiction_code: 840,
        jurisdiction_allowed: true,
        residency_allowed: true,
        wallet_revoked: false,
        expires_at: 1_900_000_000,
        evaluation_time: 1_800_000_000,
    };

    RwaCreditOnboardingClaims {
        expected: evaluate_rwa_credit_onboarding_v1(&facts),
        facts,
    }
}

fn sample_transfer_claims() -> RwaCreditTransferClaims {
    let facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32],
        asset_id_hash: [6u8; 32],
        sender_wallet: [7u8; 20],
        receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional,
        attestation_expired: false,
        receiver_accredited: true,
        receiver_kyc_passed: true,
        receiver_aml_cleared: true,
        receiver_sanctions_clear: true,
        receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true,
        receiver_residency_allowed: true,
        sender_revoked: false,
        receiver_revoked: false,
        holding_period_met: true,
        position_limit_exceeded: false,
        concentration_limit_exceeded: false,
        transfer_amount_units: 100,
        post_transfer_position_units: 200,
        wallet_position_limit_units: 400,
        post_transfer_concentration_bps: 1_500,
        concentration_limit_bps: 2_500,
        expires_at: 1_900_000_000,
        evaluation_time: 1_800_000_000,
    };

    RwaCreditTransferClaims {
        expected: evaluate_rwa_credit_transfer_v1(&facts),
        facts,
    }
}

fn halo2_onboarding_proof(claims: &RwaCreditOnboardingClaims) -> Proof {
    let params = Params::<G1Affine>::new(RWA_ONBOARDING_K);
    let empty = RwaOnboardingCircuit::<Fr> {
        public_values: vec![Fr::ZERO; ONBOARDING_INSTANCE_LEN],
    };
    let vk = keygen_vk(&params, &empty).unwrap();
    let pk = keygen_pk(&params, vk, &empty).unwrap();
    let circuit = RwaOnboardingCircuit::<Fr> {
        public_values: onboarding_instance_values::<Fr>(claims),
    };
    let public_inputs = vec![onboarding_instance_values::<Fr>(claims)];
    let instance_slices: Vec<&[Fr]> = public_inputs
        .iter()
        .map(|values| values.as_slice())
        .collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());

    create_proof(
        &params,
        &pk,
        &[circuit],
        &all_instances,
        OsRng,
        &mut transcript,
    )
    .unwrap();

    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

fn halo2_transfer_proof(claims: &RwaCreditTransferClaims) -> Proof {
    let params = Params::<G1Affine>::new(RWA_TRANSFER_K);
    let empty = RwaTransferCircuit::<Fr> {
        public_values: vec![Fr::ZERO; TRANSFER_INSTANCE_LEN],
    };
    let vk = keygen_vk(&params, &empty).unwrap();
    let pk = keygen_pk(&params, vk, &empty).unwrap();
    let circuit = RwaTransferCircuit::<Fr> {
        public_values: transfer_instance_values::<Fr>(claims),
    };
    let public_inputs = vec![transfer_instance_values::<Fr>(claims)];
    let instance_slices: Vec<&[Fr]> = public_inputs
        .iter()
        .map(|values| values.as_slice())
        .collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());

    create_proof(
        &params,
        &pk,
        &[circuit],
        &all_instances,
        OsRng,
        &mut transcript,
    )
    .unwrap();

    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

#[test]
fn rwa_onboarding_halo2_proof_is_accepted() {
    let claims = sample_onboarding_claims();
    let proof = halo2_onboarding_proof(&claims);

    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditOnboardingV1(claims)).is_ok());
}

#[test]
fn rwa_transfer_halo2_proof_is_accepted() {
    let claims = sample_transfer_claims();
    let proof = halo2_transfer_proof(&claims);

    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(claims)).is_ok());
}

#[cfg(feature = "zk-vm")]
#[test]
fn rwa_onboarding_backends_match() {
    let claims = sample_onboarding_claims();
    let halo2 = halo2_onboarding_proof(&claims);
    let zkvm = Proof::new(
        ProofSystem::ZkVm,
        prove_rwa_onboarding(claims).expect("zkvm onboarding proof"),
    );

    assert!(Verifier::verify(&halo2, &PublicInputs::RwaCreditOnboardingV1(claims)).is_ok());
    assert!(Verifier::verify(&zkvm, &PublicInputs::RwaCreditOnboardingV1(claims)).is_ok());
}

#[cfg(feature = "zk-vm")]
#[test]
fn rwa_transfer_backends_match() {
    let claims = sample_transfer_claims();
    let halo2 = halo2_transfer_proof(&claims);
    let zkvm = Proof::new(
        ProofSystem::ZkVm,
        prove_rwa_transfer(claims).expect("zkvm transfer proof"),
    );

    assert!(Verifier::verify(&halo2, &PublicInputs::RwaCreditTransferV1(claims)).is_ok());
    assert!(Verifier::verify(&zkvm, &PublicInputs::RwaCreditTransferV1(claims)).is_ok());
}
