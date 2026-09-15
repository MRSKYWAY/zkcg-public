#![cfg(all(feature = "zk-halo2", feature = "zk-vm"))]

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
    types::{
        DecisionCode, EligibilityClass, InvestorTypeCode, RwaCreditTransferClaims,
        RwaCreditTransferDecisionCommitment, RwaCreditTransferFacts,
    },
};
use zkcg_zkvm_host::prove_rwa_transfer;

#[derive(Clone, Copy)]
struct Scenario {
    category: &'static str,
    case: usize,
    facts: RwaCreditTransferFacts,
}

fn base_facts(case: usize) -> RwaCreditTransferFacts {
    let mut facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional,
        attestation_expired: false, receiver_accredited: true, receiver_kyc_passed: true,
        receiver_aml_cleared: true, receiver_sanctions_clear: true, receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true, receiver_residency_allowed: true, sender_revoked: false,
        receiver_revoked: false, holding_period_met: true, position_limit_exceeded: false,
        concentration_limit_exceeded: false, transfer_amount_units: 100 + case as u64,
        post_transfer_position_units: 200 + case as u64, wallet_position_limit_units: 400 + case as u64,
        post_transfer_concentration_bps: 1_500 + case as u64, concentration_limit_bps: 2_500 + case as u64,
        expires_at: 1_900_000_000 + case as u64, evaluation_time: 1_800_000_000,
    };
    facts.issuer_id_hash[0] = case as u8;
    facts.asset_id_hash[0] = (case.wrapping_mul(3)) as u8;
    facts.sender_wallet[0] = (case.wrapping_mul(5)) as u8;
    facts.receiver_wallet[0] = (case.wrapping_mul(7)) as u8;
    facts
}

fn scenarios() -> Vec<Scenario> {
    let mut out = Vec::with_capacity(100);
    for case in 0..100usize {
        let category = match case / 10 {
            0 => "baseline-approved", 1 => "attestation", 2 => "kyc", 3 => "aml-sanctions",
            4 => "jurisdiction-residency", 5 => "accreditation-and-investor-type", 6 => "holding-period",
            7 => "position-limit", 8 => "concentration-limit", _ => "wallet-revocation",
        };
        let mut facts = base_facts(case);
        match case / 10 {
            0 => {}
            1 => facts.attestation_expired = true,
            2 => facts.receiver_kyc_passed = false,
            3 => { if case % 2 == 0 { facts.receiver_aml_cleared = false; } else { facts.receiver_sanctions_clear = false; } }
            4 => { if case % 2 == 0 { facts.receiver_jurisdiction_allowed = false; } else { facts.receiver_residency_allowed = false; } }
            5 => { facts.receiver_investor_type = InvestorTypeCode::Retail; facts.receiver_accredited = case % 2 == 0; }
            6 => facts.holding_period_met = false,
            7 => facts.position_limit_exceeded = true,
            8 => facts.concentration_limit_exceeded = true,
            _ => { if case % 2 == 0 { facts.sender_revoked = true; } else { facts.receiver_revoked = true; } }
        }
        out.push(Scenario { category, case, facts });
    }
    out
}

fn independent_oracle(facts: &RwaCreditTransferFacts) -> RwaCreditTransferDecisionCommitment {
    const ATTESTATION_EXPIRED: u32 = 1 << 0;
    const KYC_MISSING: u32 = 1 << 1;
    const AML_FAILED: u32 = 1 << 2;
    const SANCTIONS_HIT: u32 = 1 << 3;
    const JURISDICTION_BLOCKED: u32 = 1 << 4;
    const ACCREDITATION_MISSING: u32 = 1 << 5;
    const WALLET_REVOKED: u32 = 1 << 6;
    const HOLDING_PERIOD_NOT_MET: u32 = 1 << 7;
    const POSITION_LIMIT_EXCEEDED: u32 = 1 << 8;
    const CONCENTRATION_LIMIT_EXCEEDED: u32 = 1 << 9;
    let institutional = matches!(facts.receiver_investor_type, InvestorTypeCode::Institutional);
    let mut onboarding_reasons = 0u32;
    if facts.attestation_expired { onboarding_reasons |= ATTESTATION_EXPIRED; }
    if !facts.receiver_kyc_passed { onboarding_reasons |= KYC_MISSING; }
    if !facts.receiver_aml_cleared { onboarding_reasons |= AML_FAILED; }
    if !facts.receiver_sanctions_clear { onboarding_reasons |= SANCTIONS_HIT; }
    if !facts.receiver_jurisdiction_allowed || !facts.receiver_residency_allowed { onboarding_reasons |= JURISDICTION_BLOCKED; }
    if !institutional && !facts.receiver_accredited { onboarding_reasons |= ACCREDITATION_MISSING; }
    if facts.receiver_revoked { onboarding_reasons |= WALLET_REVOKED; }
    let onboarding_passed = onboarding_reasons == 0;
    let mut reasons = onboarding_reasons;
    if !facts.holding_period_met { reasons |= HOLDING_PERIOD_NOT_MET; }
    if facts.position_limit_exceeded { reasons |= POSITION_LIMIT_EXCEEDED; }
    if facts.concentration_limit_exceeded { reasons |= CONCENTRATION_LIMIT_EXCEEDED; }
    if facts.sender_revoked { reasons |= WALLET_REVOKED; }
    let decision = if reasons == 0 { DecisionCode::Approved } else { DecisionCode::Denied };
    let eligibility_class = if onboarding_passed { if institutional { EligibilityClass::Institutional } else { EligibilityClass::Accredited } } else { EligibilityClass::None };
    RwaCreditTransferDecisionCommitment {
        decision, eligibility_class, reason_bits: reasons, expires_at: facts.expires_at,
        issuer_id_hash: facts.issuer_id_hash, asset_id_hash: facts.asset_id_hash,
        sender_wallet: facts.sender_wallet, receiver_wallet: facts.receiver_wallet,
        transfer_amount_units: facts.transfer_amount_units,
    }
}

fn claims_for(facts: RwaCreditTransferFacts) -> RwaCreditTransferClaims {
    RwaCreditTransferClaims { expected: independent_oracle(&facts), facts }
}

fn halo2_prove(claims: &RwaCreditTransferClaims, params: &Params<G1Affine>, pk: &halo2_proofs::plonk::ProvingKey<G1Affine>) -> Proof {
    let public_values = transfer_instance_values::<Fr>(claims);
    let circuit = RwaTransferCircuit::<Fr> { public_values: public_values.clone() };
    let instances = vec![public_values];
    let instance_slices: Vec<&[Fr]> = instances.iter().map(Vec::as_slice).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());
    create_proof(params, pk, &[circuit], &all_instances, OsRng, &mut transcript).unwrap();
    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

fn halo2_keys() -> (Params<G1Affine>, halo2_proofs::plonk::ProvingKey<G1Affine>) {
    let params = Params::<G1Affine>::new(RWA_TRANSFER_K);
    let empty = RwaTransferCircuit::<Fr> { public_values: vec![Fr::ZERO; TRANSFER_INSTANCE_LEN] };
    let vk = keygen_vk(&params, &empty).unwrap();
    let pk = keygen_pk(&params, vk, &empty).unwrap();
    (params, pk)
}

fn risc0_shard() -> (usize, usize) {
    let shard: usize = std::env::var("PHASE5_RISC0_SHARD").unwrap_or_else(|_| "0".to_owned()).parse().expect("PHASE5_RISC0_SHARD must be an integer");
    let shards: usize = std::env::var("PHASE5_RISC0_SHARDS").unwrap_or_else(|_| "10".to_owned()).parse().expect("PHASE5_RISC0_SHARDS must be an integer");
    assert!(shards > 0, "PHASE5_RISC0_SHARDS must be > 0");
    assert!(shard < shards, "PHASE5_RISC0_SHARD must be < PHASE5_RISC0_SHARDS");
    (shard, shards)
}

#[test]
fn rwa_differential_100_vectors_halo2_match_independent_oracle() {
    let (params, pk) = halo2_keys();
    for scenario in scenarios() {
        let claims = claims_for(scenario.facts);
        let halo2 = halo2_prove(&claims, &params, &pk);
        let result = Verifier::verify(&halo2, &PublicInputs::RwaCreditTransferV1(claims));
        assert!(result.is_ok(), "Halo2 mismatch in {} case {}", scenario.category, scenario.case);
    }
}

#[test]
fn rwa_differential_100_vectors_risc0_match_independent_oracle() {
    let (shard, shards) = risc0_shard();
    let all = scenarios();
    let shard_cases: Vec<_> = all.iter().enumerate().filter(|(index, _)| index % shards == shard).map(|(_, scenario)| *scenario).collect();
    assert_eq!(shard_cases.len(), 10, "expected 10 vectors per RISC0 shard");
    for scenario in shard_cases {
        let claims = claims_for(scenario.facts);
        let zkvm_bytes = prove_rwa_transfer(claims.clone()).expect("RISC0 transfer proof");
        let zkvm = Proof::new(ProofSystem::ZkVm, zkvm_bytes);
        let result = Verifier::verify(&zkvm, &PublicInputs::RwaCreditTransferV1(claims));
        assert!(result.is_ok(), "RISC0 mismatch in {} case {}", scenario.category, scenario.case);
    }
}

#[test]
fn rwa_differential_wrong_commitment_canary_is_detected() {
    let (params, pk) = halo2_keys();
    let scenario = scenarios().into_iter().next().unwrap();
    let mut claims = claims_for(scenario.facts);
    let correct = claims.expected.clone();
    claims.expected.decision = match correct.decision { DecisionCode::Approved => DecisionCode::Denied, DecisionCode::Denied => DecisionCode::Approved, _ => unreachable!(), };
    let halo2_proof = halo2_prove(&claims, &params, &pk);
    assert!(Verifier::verify(&halo2_proof, &PublicInputs::RwaCreditTransferV1(claims.clone())).is_err(), "Halo2 accepted deliberately incorrect commitment");
    let zkvm = prove_rwa_transfer(claims);
    assert!(zkvm.is_err(), "RISC0 accepted deliberately incorrect commitment");
}

#[test]
fn rwa_differential_oracle_matches_production_evaluator() {
    for scenario in scenarios() {
        let oracle = independent_oracle(&scenario.facts);
        let production = evaluate_rwa_credit_transfer_v1(&scenario.facts);
        assert_eq!(oracle, production, "oracle drift in {} case {}", scenario.category, scenario.case);
    }
}
