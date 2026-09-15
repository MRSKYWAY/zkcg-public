//! Reusable security-testing primitives for Halo2-based proof systems.
//!
//! The crate is intentionally independent of ZKCG application types. A consuming
//! project supplies its own proof, public-input, oracle, and backend adapters.

pub mod adversarial;
pub mod circuit;
pub mod differential;
pub mod malformed;
pub mod mutation;
pub mod replay;
pub mod reporting;

pub use adversarial::{AdversarialCase, AdversarialHarness, AdversarialOutcome};
pub use circuit::{CircuitMutationCase, CircuitMutationOutcome, CircuitMutationRunner};
pub use differential::{
    DifferentialCase, DifferentialHarness, DifferentialOutcome, VerificationBackend,
};
pub use malformed::{MalformedProofMutation, mutate_proof};
pub use mutation::{MutationCase, MutationHarness, MutationOutcome, SecurityOracle};
pub use replay::{BindingCase, ReplayHarness, ReplayOutcome};
pub use reporting::{EvidenceRecord, EvidenceSummary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acceptance {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityTestError<E> {
    Backend(E),
    UnexpectedAcceptance,
    UnexpectedRejection,
}
