use crate::{Acceptance, SecurityTestError};

/// Independently computes expected security behavior for a test input.
pub trait SecurityOracle<Input> {
    type Expected: Clone + PartialEq;
    fn expected(&self, input: &Input) -> Self::Expected;
}

/// One deliberately mutated security-relevant public result.
#[derive(Debug, Clone)]
pub struct MutationCase<Input, Expected> {
    pub name: String,
    pub input: Input,
    pub expected: Expected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationOutcome {
    pub name: String,
    pub accepted: bool,
    pub expected_rejection: bool,
}

pub struct MutationHarness<F> {
    verify: F,
}

impl<F> MutationHarness<F> {
    pub fn new(verify: F) -> Self {
        Self { verify }
    }

    /// Runs a semantic mutation and requires the real verifier to reject it.
    pub fn run<Input, Expected, E>(
        &self,
        case: MutationCase<Input, Expected>,
    ) -> Result<MutationOutcome, SecurityTestError<E>>
    where
        F: Fn(&Input, &Expected) -> Result<Acceptance, E>,
    {
        match (self.verify)(&case.input, &case.expected).map_err(SecurityTestError::Backend)? {
            Acceptance::Rejected => Ok(MutationOutcome {
                name: case.name,
                accepted: false,
                expected_rejection: true,
            }),
            Acceptance::Accepted => Err(SecurityTestError::UnexpectedAcceptance),
        }
    }
}
