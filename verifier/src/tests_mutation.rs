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
use zkcg_common::types::{
    CONCENTRATION_LIMIT_EXCEEDED_BIT, DecisionCode, EligibilityClass, HOLDING_PERIOD_NOT_MET_BIT,
    InvestorTypeCode, POSITION_LIMIT_EXCEEDED_BIT, RwaCreditTransferClaims,
    RwaCreditTransferDecisionCommitment, RwaCreditTransferFacts, WALLET_REVOKED_BIT,
};

#[derive(Clone, Copy)]
enum Target { HoldingPeriod, PositionLimit, ConcentrationLimit, SenderRevoked, ReceiverRevoked }

fn forged_claims(target: Target) -> RwaCreditTransferClaims {
    let facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional,
        attestation_expired: false, receiver_accredited: true, receiver_kyc_passed: true,
        receiver_aml_cleared: true, receiver_sanctions_clear: true, receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true, receiver_residency_allowed: true,
        sender_revoked: matches!(target, Target::SenderRevoked),
        receiver_revoked: matches!(target, Target::ReceiverRevoked),
        holding_period_met: !matches!(target, Target::HoldingPeriod),
        position_limit_exceeded: matches!(target, Target::PositionLimit),
        concentration_limit_exceeded: matches!(target, Target::ConcentrationLimit),
        transfer_amount_units: 100, post_transfer_position_units: 200, wallet_position_limit_units: 400,
        post_transfer_concentration_bps: 1_500, concentration_limit_bps: 2_500,
        expires_at: 1_900_000_000, evaluation_time: 1_800_000_000,
    };
    let mut reasons = 0;
    if !facts.holding_period_met { reasons |= HOLDING_PERIOD_NOT_MET_BIT; }
    if facts.position_limit_exceeded { reasons |= POSITION_LIMIT_EXCEEDED_BIT; }
    if facts.concentration_limit_exceeded { reasons |= CONCENTRATION_LIMIT_EXCEEDED_BIT; }
    if facts.sender_revoked || facts.receiver_revoked { reasons |= WALLET_REVOKED_BIT; }
    RwaCreditTransferClaims {
        facts,
        expected: RwaCreditTransferDecisionCommitment {
            decision: DecisionCode::Approved, eligibility_class: EligibilityClass::Institutional,
            reason_bits: reasons, expires_at: 1_900_000_000,
            issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
            transfer_amount_units: 100,
        },
    }
}

fn prove(claims: &RwaCreditTransferClaims) -> Proof {
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

fn reject(target: Target) {
    let claims = forged_claims(target);
    let proof = prove(&claims);
    assert!(Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(claims)).is_err());
}

#[test] fn oracle_rejects_missing_holding_period_gate() { reject(Target::HoldingPeriod); }
#[test] fn oracle_rejects_missing_position_limit_gate() { reject(Target::PositionLimit); }
#[test] fn oracle_rejects_missing_concentration_limit_gate() { reject(Target::ConcentrationLimit); }
#[test] fn oracle_rejects_missing_sender_revocation_gate() { reject(Target::SenderRevoked); }
#[test] fn oracle_rejects_missing_receiver_revocation_gate() { reject(Target::ReceiverRevoked); }
