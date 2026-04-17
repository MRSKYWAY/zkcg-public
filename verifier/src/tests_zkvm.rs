#![cfg(feature = "zk-vm")]

use crate::{
    Proof, ProofSystem, Verifier,
    backend_zkvm::ZkVmBackend,
    engine::{PublicInputs, VerifierEngine},
};
use zkcg_common::{errors::ProtocolError, state::ProtocolState, types::Commitment};
use zkcg_zkvm_host::{ZkVmProverError, prove};

fn commitment() -> Commitment {
    Commitment([42u8; 32])
}

fn valid_inputs() -> PublicInputs {
    PublicInputs::phase1_score(10, [9u8; 32], 7)
}

fn test_inputs() -> PublicInputs {
    PublicInputs::phase1_score(10, [0u8; 32], 1)
}

fn mock_state(inputs: &PublicInputs) -> ProtocolState {
    let phase1 = inputs.phase1().unwrap();
    ProtocolState {
        state_root: phase1.old_state_root,
        nonce: phase1.nonce - 1,
        ..ProtocolState::genesis()
    }
}

fn prove_valid(inputs: &PublicInputs) -> Vec<u8> {
    let phase1 = inputs.phase1().unwrap();
    prove(5, phase1.threshold, phase1.old_state_root, phase1.nonce).expect("valid proof")
}

#[test]
fn zkvm_valid_transition_succeeds() {
    let inputs = test_inputs();
    let state = mock_state(&inputs);

    let mut engine = VerifierEngine::new(state.clone(), Box::new(ZkVmBackend));

    let proof_bytes = prove_valid(&inputs);
    let proof = Proof::new(ProofSystem::ZkVm, proof_bytes.clone());

    assert!(Verifier::verify(&proof, &inputs).is_ok());

    let result = engine.process_transition(&proof_bytes, inputs, commitment());

    assert!(result.is_ok(), "Valid transition should succeed");
}

#[test]
fn zkvm_policy_violation_is_rejected() {
    let inputs = valid_inputs();
    let phase1 = inputs.phase1().unwrap();
    let result = prove(20, 10, phase1.old_state_root, phase1.nonce);

    assert!(matches!(result, Err(ZkVmProverError::PolicyViolation)));
}

#[test]
fn zkvm_tampered_proof_is_rejected() {
    let inputs = test_inputs();
    let mut proof_bytes = prove_valid(&inputs);

    proof_bytes[0] ^= 0xFF;

    let proof = Proof::new(ProofSystem::ZkVm, proof_bytes);
    let result = Verifier::verify(&proof, &inputs);

    assert!(matches!(result, Err(ProtocolError::InvalidProof)));
}

#[test]
fn zkvm_empty_proof_is_rejected() {
    let proof = Proof::new(ProofSystem::ZkVm, Vec::<u8>::new());
    let result = Verifier::verify(&proof, &test_inputs());

    assert!(result.is_err());
}

#[test]
fn zkvm_threshold_mismatch_is_rejected() {
    let inputs = test_inputs();
    let proof = Proof::new(ProofSystem::ZkVm, prove_valid(&inputs));
    let phase1 = inputs.phase1().unwrap();
    let mismatched_inputs =
        PublicInputs::phase1_score(phase1.threshold + 1, phase1.old_state_root, phase1.nonce);

    let result = Verifier::verify(&proof, &mismatched_inputs);

    assert!(matches!(result, Err(ProtocolError::InvalidProof)));
}

#[test]
fn zkvm_state_root_mismatch_is_rejected() {
    let inputs = test_inputs();
    let proof = Proof::new(ProofSystem::ZkVm, prove_valid(&inputs));
    let phase1 = inputs.phase1().unwrap();
    let mismatched_inputs = PublicInputs::phase1_score(phase1.threshold, [7u8; 32], phase1.nonce);

    let result = Verifier::verify(&proof, &mismatched_inputs);

    assert!(matches!(result, Err(ProtocolError::InvalidProof)));
}

#[test]
fn zkvm_nonce_mismatch_is_rejected() {
    let inputs = test_inputs();
    let proof = Proof::new(ProofSystem::ZkVm, prove_valid(&inputs));
    let phase1 = inputs.phase1().unwrap();
    let mismatched_inputs =
        PublicInputs::phase1_score(phase1.threshold, phase1.old_state_root, phase1.nonce + 1);

    let result = Verifier::verify(&proof, &mismatched_inputs);

    assert!(matches!(result, Err(ProtocolError::InvalidProof)));
}

#[test]
fn zkvm_overflow_inputs_rejected() {
    let inputs = valid_inputs();
    let phase1 = inputs.phase1().unwrap();
    let result = prove(u64::MAX, u64::MAX - 1, phase1.old_state_root, phase1.nonce);

    assert!(result.is_err());
}
