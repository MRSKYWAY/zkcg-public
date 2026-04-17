use crate::{Proof, ProofSystem, VerificationRequest, Verifier, engine::PublicInputs};
use zkcg_halo2_prover::{
    Halo2PayoutContext, PayoutPolicy, PayoutRecipientSnapshot, PayoutRecipientStatus, PayoutRow,
};

fn sample_rows() -> Vec<PayoutRow> {
    vec![
        PayoutRow {
            recipient_address: "0x1111111111111111111111111111111111111111".to_string(),
            amount_units: 10,
        },
        PayoutRow {
            recipient_address: "0x2222222222222222222222222222222222222222".to_string(),
            amount_units: 25,
        },
    ]
}

fn sample_policy() -> PayoutPolicy {
    PayoutPolicy {
        operator_id: "miner-a".to_string(),
        program_id: "pool-main".to_string(),
        asset_id: "btc".to_string(),
        round_id: "round-42".to_string(),
        round_cap_units: 1_000,
        per_recipient_cap_units: 100,
        max_rows_per_round: 10,
        max_chunks_per_round: 4,
        round_nonce: 42,
        release_window_ends_at: 4_000_000_000,
    }
}

fn sample_snapshot() -> PayoutRecipientSnapshot {
    PayoutRecipientSnapshot {
        expires_at: 4_000_000_000,
        recipients: vec![
            PayoutRecipientStatus {
                recipient_address: "0x1111111111111111111111111111111111111111".to_string(),
                approved: true,
                kyc_passed: true,
                aml_cleared: true,
                sanctions_clear: true,
            },
            PayoutRecipientStatus {
                recipient_address: "0x2222222222222222222222222222222222222222".to_string(),
                approved: true,
                kyc_passed: true,
                aml_cleared: true,
                sanctions_clear: true,
            },
        ],
    }
}

#[test]
fn halo2_payout_proof_verifies_through_default_registry() {
    let artifact = Halo2PayoutContext::new()
        .prove_round(&sample_rows(), &sample_policy(), 1_900_000_000, false)
        .expect("proof should be generated");
    let build = artifact.build;
    let proof_bytes = artifact.proof;
    let proof = Proof::new(ProofSystem::Halo2, proof_bytes);

    assert!(Verifier::verify(&proof, &PublicInputs::BulkPayoutRoundV1(build.claims)).is_ok());
}

#[test]
fn halo2_payout_batch_verifies_in_parallel() {
    let artifact = Halo2PayoutContext::new()
        .prove_round(&sample_rows(), &sample_policy(), 1_900_000_000, false)
        .expect("proof should be generated");
    let build = artifact.build;
    let proof_bytes = artifact.proof;
    let request = VerificationRequest::new(
        Proof::new(ProofSystem::Halo2, proof_bytes),
        PublicInputs::BulkPayoutRoundV1(build.claims),
    );

    assert!(Verifier::verify_batch_parallel(&[request.clone(), request]).is_ok());
}

#[test]
fn tampered_payout_claims_are_rejected() {
    let artifact = Halo2PayoutContext::new()
        .prove_round(&sample_rows(), &sample_policy(), 1_900_000_000, false)
        .expect("proof should be generated");
    let mut tampered = artifact.build.claims;
    tampered.expected.total_amount_units += 1;
    let proof = Proof::new(ProofSystem::Halo2, artifact.proof);

    assert!(Verifier::verify(&proof, &PublicInputs::BulkPayoutRoundV1(tampered)).is_err());
}

#[test]
fn halo2_payout_release_proof_verifies_through_default_registry() {
    let artifact = Halo2PayoutContext::new()
        .prove_release(
            &sample_rows(),
            &sample_policy(),
            &sample_snapshot(),
            1_900_000_000,
            false,
        )
        .expect("proof should be generated");
    let proof = Proof::new(ProofSystem::Halo2, artifact.proof);

    assert!(
        Verifier::verify(
            &proof,
            &PublicInputs::PayoutReleaseV1(artifact.build.claims)
        )
        .is_ok()
    );
}

#[test]
fn tampered_payout_release_snapshot_hash_is_rejected() {
    let artifact = Halo2PayoutContext::new()
        .prove_release(
            &sample_rows(),
            &sample_policy(),
            &sample_snapshot(),
            1_900_000_000,
            false,
        )
        .expect("proof should be generated");
    let mut tampered = artifact.build.claims;
    tampered.expected.recipient_snapshot_hash[0] ^= 1;
    let proof = Proof::new(ProofSystem::Halo2, artifact.proof);

    assert!(Verifier::verify(&proof, &PublicInputs::PayoutReleaseV1(tampered)).is_err());
}
