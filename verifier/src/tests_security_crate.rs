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
use zkcg_circuits::rwa_circuit::{RWA_TRANSFER_K, RwaTransferCircuit, TRANSFER_INSTANCE_LEN, transfer_instance_values};
use zkcg_common::{rwa::evaluate_rwa_credit_transfer_v1, types::{InvestorTypeCode, RwaCreditTransferClaims, RwaCreditTransferFacts}};
use zkcg_halo2_test::{
    Acceptance, AdversarialCase, AdversarialHarness, BindingCase, CircuitMutationCase,
    CircuitMutationRunner, DifferentialCase, DifferentialHarness, MalformedProofMutation,
    MutationCase, MutationHarness, ReplayHarness, SecurityOracle, VerificationBackend,
    mutate_proof,
};

fn sample_claims() -> RwaCreditTransferClaims {
    let facts = RwaCreditTransferFacts {
        issuer_id_hash: [5u8; 32], asset_id_hash: [6u8; 32], sender_wallet: [7u8; 20], receiver_wallet: [8u8; 20],
        receiver_investor_type: InvestorTypeCode::Institutional, attestation_expired: false, receiver_accredited: true,
        receiver_kyc_passed: true, receiver_aml_cleared: true, receiver_sanctions_clear: true, receiver_jurisdiction_code: 840,
        receiver_jurisdiction_allowed: true, receiver_residency_allowed: true, sender_revoked: false, receiver_revoked: false,
        holding_period_met: true, position_limit_exceeded: false, concentration_limit_exceeded: false,
        transfer_amount_units: 100, post_transfer_position_units: 200, wallet_position_limit_units: 400,
        post_transfer_concentration_bps: 1_500, concentration_limit_bps: 2_500, expires_at: 1_900_000_000, evaluation_time: 1_800_000_000,
    };
    RwaCreditTransferClaims { expected: evaluate_rwa_credit_transfer_v1(&facts), facts }
}

fn halo2_proof(claims: &RwaCreditTransferClaims) -> Proof {
    let params = Params::<G1Affine>::new(RWA_TRANSFER_K);
    let empty = RwaTransferCircuit::<Fr> { public_values: vec![Fr::ZERO; TRANSFER_INSTANCE_LEN] };
    let vk = keygen_vk(&params, &empty).unwrap();
    let pk = keygen_pk(&params, vk, &empty).unwrap();
    let public_values = transfer_instance_values::<Fr>(claims);
    let circuit = RwaTransferCircuit::<Fr> { public_values: public_values.clone() };
    let instances = [public_values];
    let instance_slices: Vec<&[Fr]> = instances.iter().map(Vec::as_slice).collect();
    let all_instances: Vec<&[&[Fr]]> = vec![instance_slices.as_slice()];
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<G1Affine>>::init(Vec::new());
    create_proof(&params, &pk, &[circuit], &all_instances, OsRng, &mut transcript).unwrap();
    Proof::new(ProofSystem::Halo2, transcript.finalize())
}

struct Halo2Backend;
impl VerificationBackend<RwaCreditTransferClaims> for Halo2Backend {
    type Error = String;
    fn verify(&self, claims: &RwaCreditTransferClaims) -> Result<Acceptance, Self::Error> {
        let proof = halo2_proof(claims);
        Ok(if Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(*claims)).is_ok() { Acceptance::Accepted } else { Acceptance::Rejected })
    }
}

struct Oracle;
impl SecurityOracle<RwaCreditTransferClaims> for Oracle {
    type Expected = bool;
    fn expected(&self, input: &RwaCreditTransferClaims) -> Self::Expected {
        evaluate_rwa_credit_transfer_v1(&input.facts).decision == input.expected.decision
    }
}

#[test]
fn phase1_adversarial_harness_records_numeric_constraint_gap() {
    let mut claims = sample_claims();
    claims.facts.post_transfer_position_units = claims.facts.wallet_position_limit_units + 1;
    let proof = halo2_proof(&claims);
    let claims_for_verification = claims;
    let harness = AdversarialHarness::new(move |proof: &Proof| {
        Ok::<_, String>(if Verifier::verify(proof, &PublicInputs::RwaCreditTransferV1(claims_for_verification)).is_ok() { Acceptance::Accepted } else { Acceptance::Rejected })
    });
    let result = harness.run(AdversarialCase { name: "numeric position limit with false normalized flag".into(), input: proof, expected: Acceptance::Accepted }).unwrap();
    assert_eq!(result.observed, Acceptance::Accepted);
}

#[test]
fn phase2_mutation_harness_rejects_changed_claim_commitment() {
    let claims = sample_claims(); let proof = halo2_proof(&claims); let mut mutated = claims; mutated.facts.transfer_amount_units += 1;
    let harness = MutationHarness::new(|proof: &Proof, expected: &RwaCreditTransferClaims| Ok::<_, String>(if Verifier::verify(proof, &PublicInputs::RwaCreditTransferV1(*expected)).is_ok() { Acceptance::Accepted } else { Acceptance::Rejected }));
    let result = harness.run(MutationCase { name: "changed transfer amount".into(), input: proof, expected: mutated }).unwrap();
    assert!(result.expected_rejection);
}

#[test]
fn phase3_malformed_and_replay_harnesses_work_against_real_verifier() {
    let claims = sample_claims(); let proof = halo2_proof(&claims);
    for mutation in [MalformedProofMutation::Empty, MalformedProofMutation::Truncate(proof.data.len().saturating_sub(1)), MalformedProofMutation::Append(vec![1]), MalformedProofMutation::FlipBit { offset: 0, bit: 0 }] {
        let bytes = mutate_proof(&proof.data, mutation); let mutated = Proof::new(ProofSystem::Halo2, bytes);
        assert!(Verifier::verify(&mutated, &PublicInputs::RwaCreditTransferV1(claims)).is_err());
    }
    let mut changed = claims; changed.facts.transfer_amount_units += 1; let original_claims = claims;
    let replay = ReplayHarness::new(move |input: &RwaCreditTransferClaims| { let proof = halo2_proof(&original_claims); Ok::<_, String>(if Verifier::verify(&proof, &PublicInputs::RwaCreditTransferV1(*input)).is_ok() { Acceptance::Accepted } else { Acceptance::Rejected }) });
    let result = replay.run(BindingCase { name: "amount substitution".into(), original: claims, mutated: changed }).unwrap();
    assert!(result.mutated_rejected);
}

#[test]
fn phase5_differential_harness_can_compare_real_halo2_backends() {
    let claims = sample_claims(); let expected = evaluate_rwa_credit_transfer_v1(&claims.facts).decision == claims.expected.decision;
    let harness = DifferentialHarness::new(Halo2Backend, Halo2Backend, move |_input: &RwaCreditTransferClaims| Ok::<_, String>(if expected { Acceptance::Accepted } else { Acceptance::Rejected }));
    let result = harness.run(DifferentialCase { name: "reference vector".into(), input: claims }).unwrap();
    assert_eq!(result.backend_a, result.oracle); assert_eq!(result.backend_b, result.oracle);
}

#[test]
fn phase8_circuit_mutation_runner_exercises_consumer_contract() {
    let runner = CircuitMutationRunner::new(|input: &u64, mutated: bool| Ok::<_, String>(if mutated && *input == 7 { Acceptance::Accepted } else { Acceptance::Rejected }));
    let result = runner.run(CircuitMutationCase { name: "missing output binding".into(), malicious_input: 7 }).unwrap();
    assert!(result.intact_rejected); assert!(result.mutated_accepted);
}

#[test]
fn oracle_trait_is_available_to_consumers() { let oracle = Oracle; let claims = sample_claims(); assert!(oracle.expected(&claims)); }
