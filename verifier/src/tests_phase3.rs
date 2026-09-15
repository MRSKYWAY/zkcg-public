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

fn sample_claims() -> RwaCreditTransferClaims {
    let facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional,
        attestation_expired: false, receiver_accredited: true, receiver_kyc_passed: true,
        receiver_aml_cleared: true, receiver_sanctions_clear: true, receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true, receiver_residency_allowed: true, sender_revoked: false,
        receiver_revoked: false, holding_period_met: true, position_limit_exceeded: false,
        concentration_limit_exceeded: false, transfer_amount_units: 100, post_transfer_position_units: 200,
        wallet_position_limit_units: 400, post_transfer_concentration_bps: 1_500,
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
    let instances = [transfer_instance_values::<Fr>(claims)];
    let instance_slices: Vec<&[Fr]> = instances.iter().map(Vec::as_slice).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());
    create_proof(&params, &pk, &[circuit], &all_instances, OsRng, &mut transcript).unwrap();
    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

fn malformed_variants(proof: &[u8]) -> Vec<Vec<u8>> {
    assert!(!proof.is_empty());
    let len = proof.len();
    let mut variants = Vec::with_capacity(64);
    for case in 0..32usize {
        let mut mutated = proof.to_vec();
        let index = (case * 7919 + 17) % len;
        mutated[index] ^= 0x01u8 << (case % 8);
        variants.push(mutated);
    }
    for cut in 1..=16usize {
        let keep = len.saturating_sub(cut).max(1);
        variants.push(proof[..keep].to_vec());
    }
    for byte in 1u8..=8u8 {
        let mut mutated = proof.to_vec();
        mutated.push(byte);
        variants.push(mutated);
    }
    for case in 0..8usize {
        let mut mutated = proof.to_vec();
        let start = (case * 104729 + 31) % len;
        for offset in 0..4usize { mutated[(start + offset) % len] ^= 0xA5; }
        variants.push(mutated);
    }
    assert_eq!(variants.len(), 64);
    variants
}

#[test]
fn halo2_rejects_64_malformed_proof_variants() {
    let claims = sample_claims();
    let proof = halo2_transfer_proof(&claims);
    for (index, mutated_bytes) in malformed_variants(&proof.data).into_iter().enumerate() {
        let mutated = Proof::new(ProofSystem::Halo2, mutated_bytes);
        let result = Verifier::verify(&mutated, &PublicInputs::RwaCreditTransferV1(claims));
        assert!(result.is_err(), "malformed proof variant #{index} was accepted");
    }
}

#[test]
fn halo2_rejects_replayed_proof_against_changed_public_inputs() {
    let claims = sample_claims();
    let proof = halo2_transfer_proof(&claims);
    let mut changed = claims;
    changed.facts.transfer_amount_units += 1;
    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(changed)).is_err());
}

#[test]
fn halo2_rejects_replayed_proof_with_changed_identity_binding() {
    let claims = sample_claims();
    let proof = halo2_transfer_proof(&claims);
    let mut changed = claims;
    changed.facts.issuer_id_hash[0] ^= 0x01;
    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(changed)).is_err());
}

#[test]
fn halo2_rejects_proof_under_wrong_system_tag() {
    let claims = sample_claims();
    let halo2 = halo2_transfer_proof(&claims);
    let tagged = Proof::new(ProofSystem::custom("phase3-wrong-system"), halo2.data);
    assert!(Verifier::verify(&tagged, &PublicInputs::RwaCreditTransferV1(claims)).is_err());
}
