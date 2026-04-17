use crate::{Proof, ProofSystem, Verifier, engine::PublicInputs};
use zkcg_common::errors::ProtocolError;

pub struct ProofInput<'a> {
    pub proof_bytes: &'a [u8],
    pub public_inputs: &'a PublicInputs,
}

pub fn verify(proof: ProofInput<'_>) -> Result<(), ProtocolError> {
    let legacy_proof = Proof::new(ProofSystem::Halo2, proof.proof_bytes);

    Verifier::verify(&legacy_proof, proof.public_inputs)
}
