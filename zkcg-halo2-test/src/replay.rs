use crate::{Acceptance, SecurityTestError};

#[derive(Debug, Clone)]
pub struct BindingCase<Input> {
    pub name: String,
    pub original: Input,
    pub mutated: Input,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayOutcome {
    pub name: String,
    pub original_accepted: bool,
    pub mutated_rejected: bool,
}

pub struct ReplayHarness<F> {
    verify: F,
}

impl<F> ReplayHarness<F> {
    pub fn new(verify: F) -> Self {
        Self { verify }
    }

    /// Verifies the original context succeeds and the altered context rejects it.
    pub fn run<Input, E>(
        &self,
        case: BindingCase<Input>,
    ) -> Result<ReplayOutcome, SecurityTestError<E>>
    where
        F: Fn(&Input) -> Result<Acceptance, E>,
    {
        let original = (self.verify)(&case.original).map_err(SecurityTestError::Backend)?;
        let mutated = (self.verify)(&case.mutated).map_err(SecurityTestError::Backend)?;

        if original != Acceptance::Accepted {
            return Err(SecurityTestError::UnexpectedRejection);
        }
        if mutated != Acceptance::Rejected {
            return Err(SecurityTestError::UnexpectedAcceptance);
        }

        Ok(ReplayOutcome {
            name: case.name,
            original_accepted: true,
            mutated_rejected: true,
        })
    }
}
