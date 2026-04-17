#![cfg(feature = "zk-vm")]

use crate::{core::verifier::ProofVerifier, engine::PublicInputs};
use risc0_zkvm::Receipt;
use zkcg_common::{errors::ProtocolError, types::ZkVmJournal};
use zkcg_zkvm_host::method_id;

#[derive(Clone, Copy, Debug, Default)]
pub struct ZkVmVerifier;

impl ProofVerifier for ZkVmVerifier {
    fn verify(
        &self,
        proof_bytes: &[u8],
        public_inputs: &PublicInputs,
    ) -> Result<(), ProtocolError> {
        let receipt: Receipt =
            bincode::deserialize(proof_bytes).map_err(|_| ProtocolError::InvalidProof)?;

        receipt
            .verify(method_id())
            .map_err(|_| ProtocolError::InvalidProof)?;

        let journal: ZkVmJournal = receipt
            .journal
            .decode()
            .map_err(|_| ProtocolError::InvalidProof)?;

        if &journal != public_inputs {
            return Err(ProtocolError::InvalidProof);
        }

        Ok(())
    }
}
