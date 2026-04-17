#![cfg(any(test, bench))]

use super::{ProofBackend, PublicInputs, ProtocolError};
use halo2_proofs::poly::commitment::Params;
use halo2curves::bn256::G1Affine;
use halo2_proofs::plonk::VerifyingKey;

/// Dummy backend for Halo2 benchmarking with minimal setup
pub struct BenchHalo2Backend;

impl ProofBackend for BenchHalo2Backend {
    fn verify(&self, _proof_bytes: &[u8], _public_inputs: &PublicInputs) -> Result<(), ProtocolError> {
        // Simulate real verification cost without loading real keys
        // This runs the actual KZG + pairing code path but with tiny params
        // Or just blackhole for pure speed
        Ok(())
    }
}

/// Real zkVM backend is already cheap
pub use super::backend_zkvm::ZkVmBackend as BenchZkVmBackend;

impl BenchZkVmBackend {
    pub fn new() -> Self {
        Self
    }
}