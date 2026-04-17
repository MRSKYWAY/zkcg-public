use crate::backend_stub::StubBackend;
use crate::engine::{PublicInputs, VerifierEngine};
use zkcg_common::{errors::ProtocolError, state::ProtocolState, types::Commitment};

fn dummy_commitment() -> Commitment {
    Commitment([42u8; 32])
}

fn initial_state() -> ProtocolState {
    ProtocolState::genesis()
}

fn valid_inputs(state: &ProtocolState) -> PublicInputs {
    PublicInputs::phase1_score(10, state.state_root, state.nonce + 1)
}

#[test]
fn valid_state_transition_succeeds() {
    let state = initial_state();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(StubBackend::default()));

    let inputs = valid_inputs(&state);
    let commitment = dummy_commitment();

    let result = engine.process_transition(b"valid-proof", inputs, commitment);

    assert!(result.is_ok());
}

#[test]
fn invalid_nonce_is_rejected() {
    let state = initial_state();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(StubBackend::default()));

    let commitment = dummy_commitment();

    let err = engine
        .process_transition(
            b"valid-proof",
            PublicInputs::phase1_score(10, state.state_root, state.nonce),
            commitment,
        )
        .unwrap_err();

    assert!(matches!(err, ProtocolError::InvalidNonce));
}

#[test]
fn state_root_mismatch_is_rejected() {
    let state = initial_state();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(StubBackend::default()));

    let commitment = dummy_commitment();

    let err = engine
        .process_transition(
            b"valid-proof",
            PublicInputs::phase1_score(10, [1u8; 32], state.nonce + 1),
            commitment,
        )
        .unwrap_err();

    assert!(matches!(err, ProtocolError::StateMismatch));
}

#[cfg(not(feature = "zk-halo2"))]
#[test]
fn policy_violation_is_rejected() {
    let state = initial_state();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(StubBackend::default()));

    let commitment = dummy_commitment();

    let err = engine
        .process_transition(
            b"valid-proof",
            PublicInputs::phase1_score(0, state.state_root, state.nonce + 1),
            commitment,
        )
        .unwrap_err();

    assert!(matches!(err, ProtocolError::PolicyViolation));
}

#[test]
fn state_updates_after_valid_transition() {
    let state = initial_state();
    let mut engine = VerifierEngine::new(state.clone(), Box::new(StubBackend::default()));

    let inputs = valid_inputs(&state);
    let commitment = dummy_commitment();

    let result = engine.process_transition(b"valid-proof", inputs, commitment.clone());
    result.unwrap();
    #[cfg(not(feature = "zk-halo2"))]
    {
        let updated = engine.state();

        assert_eq!(updated.nonce, 1);
        assert_eq!(updated.state_root, commitment.0);
    }

    let updated = engine.state();

    assert_eq!(updated.nonce, 1);
    assert_eq!(updated.state_root, commitment.0);
}
