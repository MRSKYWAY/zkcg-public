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
    RWA_TRANSFER_K, RwaTransferCircuit, TRANSFER_INSTANCE_LEN, transfer_instance_values,
};
use zkcg_common::{
    rwa::evaluate_rwa_credit_transfer_v1,
    types::{InvestorTypeCode, RwaCreditTransferClaims, RwaCreditTransferFacts},
};

fn inconsistent_transfer_claims(
    position_violation: bool,
    concentration_violation: bool,
) -> RwaCreditTransferClaims {
    let facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional,
        attestation_expired: false, receiver_accredited: true, receiver_kyc_passed: true,
        receiver_aml_cleared: true, receiver_sanctions_clear: true, receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true, receiver_residency_allowed: true, sender_revoked: false,
        receiver_revoked: false, holding_period_met: true,
        position_limit_exceeded: false, concentration_limit_exceeded: false,
        transfer_amount_units: 100,
        post_transfer_position_units: if position_violation { 401 } else { 200 },
        wallet_position_limit_units: 400,
        post_transfer_concentration_bps: if concentration_violation { 2_501 } else { 1_500 },
        concentration_limit_bps: 2_500, expires_at: 1_900_000_000, evaluation_time: 1_800_000_000,
    };
    RwaCreditTransferClaims { expected: evaluate_rwa_credit_transfer_v1(&facts), facts }
}

fn halo2_transfer_proof(claims: &RwaCreditTransferClaims) -> Proof {
    let params = Params::<G1Affine>::new(RWA_TRANSFER_K);
    let empty = RwaTransferCircuit::<Fr> { public_values: vec![Fr::ZERO; TRANSFER_INSTANCE_LEN] };
    let vk = keygen_vk(&params, &empty).unwrap();
    let pk = keygen_pk(&params, vk, &empty).unwrap();
    let circuit = RwaTransferCircuit::<Fr> { public_values: transfer_instance_values::<Fr>(claims) };
    let public_inputs = [transfer_instance_values::<Fr>(claims)];
    let instance_slices: Vec<&[Fr]> = public_inputs.iter().map(|values| values.as_slice()).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());
    create_proof(&params, &pk, &[circuit], &all_instances, OsRng, &mut transcript).unwrap();
    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

#[test]
fn transfer_circuit_accepts_position_numeric_violation_with_false_flag() {
    let claims = inconsistent_transfer_claims(true, false);
    assert!(claims.facts.post_transfer_position_units > claims.facts.wallet_position_limit_units);
    assert!(!claims.facts.position_limit_exceeded);
    assert_eq!(claims.expected.reason_bits, 0);
    let proof = halo2_transfer_proof(&claims);
    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(claims)).is_ok());
}

#[test]
fn transfer_circuit_accepts_concentration_numeric_violation_with_false_flag() {
    let claims = inconsistent_transfer_claims(false, true);
    assert!(claims.facts.post_transfer_concentration_bps > claims.facts.concentration_limit_bps);
    assert!(!claims.facts.concentration_limit_exceeded);
    assert_eq!(claims.expected.reason_bits, 0);
    let proof = halo2_transfer_proof(&claims);
    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(claims)).is_ok());
}
