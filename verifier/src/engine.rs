use crate::backend::ProofBackend;
use crate::policy;
use zkcg_common::{
    errors::ProtocolError,
    state::ProtocolState,
    types::{Commitment, ProofClaims},
};

pub struct VerifierEngine {
    state: ProtocolState,
    backend: Box<dyn ProofBackend>,
}

impl VerifierEngine {
    pub fn new(state: ProtocolState, backend: Box<dyn ProofBackend>) -> Self {
        Self { state, backend }
    }

    pub fn state(&self) -> &ProtocolState {
        &self.state
    }

    pub fn process_transition(
        &mut self,
        proof_bytes: &[u8],
        public_inputs: PublicInputs,
        commitment: Commitment,
    ) -> Result<(), ProtocolError> {
        let phase1 = public_inputs.phase1().ok_or(ProtocolError::InvalidFormat)?;

        // 1. Check state root
        if phase1.old_state_root != self.state.state_root {
            return Err(ProtocolError::StateMismatch);
        }

        // 2. Check nonce
        if phase1.nonce != self.state.nonce + 1 {
            return Err(ProtocolError::InvalidNonce);
        }

        // 3. Verify proof
        self.backend.verify(proof_bytes, &public_inputs)?;

        // 4. Enforce policy
        policy::enforce(&public_inputs)?;

        // 5. Update state
        self.state.state_root = commitment.0;
        self.state.nonce += 1;

        Ok(())
    }
}

pub type PublicInputs = ProofClaims;
