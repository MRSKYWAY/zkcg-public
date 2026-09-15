use crate::{Acceptance, SecurityTestError};

/// A controlled mutation pair for a circuit-soundness benchmark.
///
/// The consumer owns the actual Halo2 circuit construction. The harness only
/// checks the required security property: the intact circuit rejects the same
/// malicious statement that the deliberately weakened variant accepts.
#[derive(Debug, Clone)]
pub struct CircuitMutationCase<Input> {
    pub name: String,
    pub malicious_input: Input,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitMutationOutcome {
    pub name: String,
    pub intact_rejected: bool,
    pub mutated_accepted: bool,
}

pub struct CircuitMutationRunner<F> {
    verify: F,
}

impl<F> CircuitMutationRunner<F> {
    /// `verify` receives `(input, mutated)` where `mutated=false` selects the
    /// intact circuit and `mutated=true` selects the deliberately weakened one.
    pub fn new(verify: F) -> Self {
        Self { verify }
    }

    pub fn run<Input, E>(
        &self,
        case: CircuitMutationCase<Input>,
    ) -> Result<CircuitMutationOutcome, SecurityTestError<E>>
    where
        F: Fn(&Input, bool) -> Result<Acceptance, E>,
    {
        let intact =
            (self.verify)(&case.malicious_input, false).map_err(SecurityTestError::Backend)?;
        let mutated =
            (self.verify)(&case.malicious_input, true).map_err(SecurityTestError::Backend)?;

        if intact != Acceptance::Rejected {
            return Err(SecurityTestError::UnexpectedAcceptance);
        }
        if mutated != Acceptance::Accepted {
            return Err(SecurityTestError::UnexpectedRejection);
        }

        Ok(CircuitMutationOutcome {
            name: case.name,
            intact_rejected: true,
            mutated_accepted: true,
        })
    }
}

#[cfg(feature = "halo2")]
pub mod halo2 {
    //! Integration markers and helpers for consumers building real Halo2
    //! proving/verifying mutation cases. The generic runner stays independent
    //! of a particular circuit, transcript, curve, or backend configuration.

    pub const SUPPORTED_HALO2_SERIES: &str = "0.3.x";
}
