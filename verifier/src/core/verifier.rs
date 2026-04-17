use crate::{
    core::{
        proof::Proof,
        registry::{self, VerifierRegistry},
    },
    engine::PublicInputs,
};
use rayon::prelude::*;
use zkcg_common::errors::ProtocolError;

pub trait ProofVerifier: Send + Sync {
    fn verify(&self, proof: &[u8], public_inputs: &PublicInputs) -> Result<(), ProtocolError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationRequest {
    pub proof: Proof,
    pub public_inputs: PublicInputs,
}

impl VerificationRequest {
    pub fn new(proof: Proof, public_inputs: PublicInputs) -> Self {
        Self {
            proof,
            public_inputs,
        }
    }
}

impl From<(Proof, PublicInputs)> for VerificationRequest {
    fn from((proof, public_inputs): (Proof, PublicInputs)) -> Self {
        Self::new(proof, public_inputs)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Verifier;

impl Verifier {
    pub fn default_registry() -> &'static VerifierRegistry {
        registry::default_registry()
    }

    pub fn verify_with_registry(
        verifier_registry: &VerifierRegistry,
        proof: &Proof,
        public_inputs: &PublicInputs,
    ) -> Result<(), ProtocolError> {
        verifier_registry.verify(proof, public_inputs)
    }

    pub fn verify(proof: &Proof, public_inputs: &PublicInputs) -> Result<(), ProtocolError> {
        registry::verify(proof, public_inputs)
    }

    pub fn verify_batch_results_with_registry(
        verifier_registry: &VerifierRegistry,
        requests: &[VerificationRequest],
    ) -> Vec<Result<(), ProtocolError>> {
        requests
            .iter()
            .map(|request| {
                Self::verify_with_registry(
                    verifier_registry,
                    &request.proof,
                    &request.public_inputs,
                )
            })
            .collect()
    }

    pub fn verify_batch_results(
        requests: &[VerificationRequest],
    ) -> Vec<Result<(), ProtocolError>> {
        Self::verify_batch_results_with_registry(Self::default_registry(), requests)
    }

    pub fn verify_batch_with_registry(
        verifier_registry: &VerifierRegistry,
        requests: &[VerificationRequest],
    ) -> Result<(), ProtocolError> {
        for result in Self::verify_batch_results_with_registry(verifier_registry, requests) {
            result?;
        }

        Ok(())
    }

    pub fn verify_batch(requests: &[VerificationRequest]) -> Result<(), ProtocolError> {
        Self::verify_batch_with_registry(Self::default_registry(), requests)
    }

    pub fn verify_batch_parallel_results_with_registry(
        verifier_registry: &VerifierRegistry,
        requests: &[VerificationRequest],
    ) -> Vec<Result<(), ProtocolError>> {
        requests
            .par_iter()
            .map(|request| {
                Self::verify_with_registry(
                    verifier_registry,
                    &request.proof,
                    &request.public_inputs,
                )
            })
            .collect()
    }

    pub fn verify_batch_parallel_results(
        requests: &[VerificationRequest],
    ) -> Vec<Result<(), ProtocolError>> {
        Self::verify_batch_parallel_results_with_registry(Self::default_registry(), requests)
    }

    pub fn verify_batch_parallel_with_registry(
        verifier_registry: &VerifierRegistry,
        requests: &[VerificationRequest],
    ) -> Result<(), ProtocolError> {
        for result in Self::verify_batch_parallel_results_with_registry(verifier_registry, requests)
        {
            result?;
        }

        Ok(())
    }

    pub fn verify_batch_parallel(requests: &[VerificationRequest]) -> Result<(), ProtocolError> {
        Self::verify_batch_parallel_with_registry(Self::default_registry(), requests)
    }
}
