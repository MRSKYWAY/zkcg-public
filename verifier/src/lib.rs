pub mod adapters;
pub mod backend;
pub mod backend_stub;
pub mod core;
pub mod engine;
pub mod policy;
pub mod proof;
pub mod storage;

pub use core::proof::{Proof, ProofSystem};
pub use core::{
    registry::VerifierRegistry,
    verifier::{ProofVerifier, VerificationRequest, Verifier},
};

#[cfg(feature = "zk-halo2")]
pub mod backend_halo2;

#[cfg(feature = "zk-halo2")]
pub use backend_halo2::Halo2Backend;

#[cfg(feature = "zk-vm")]
pub mod backend_zkvm;

#[cfg(feature = "zk-vm")]
pub use backend_zkvm::ZkVmBackend;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "zk-halo2"))]
mod tests_halo2;

#[cfg(all(test, feature = "zk-halo2-kzg"))]
mod tests_halo2_kzg;

#[cfg(all(test, feature = "zk-vm"))]
mod tests_zkvm;

#[cfg(all(test, feature = "zk-halo2", feature = "zk-vm"))]
mod tests_equivalence;

#[cfg(all(test, feature = "zk-halo2"))]
mod tests_rwa;

#[cfg(all(test, feature = "zk-halo2"))]
mod tests_payout;

#[cfg(test)]
mod tests_universal;
