use crate::{Acceptance, SecurityTestError};

/// An adversarial input with an explicit expected verifier outcome.
#[derive(Debug, Clone)]
pub struct AdversarialCase<Input> {
    pub name: String,
    pub input: Input,
    pub expected: Acceptance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdversarialOutcome {
    pub name: String,
    pub expected: Acceptance,
    pub observed: Acceptance,
}

pub struct AdversarialHarness<F> {
    verify: F,
}

impl<F> AdversarialHarness<F> {
    pub fn new(verify: F) -> Self {
        Self { verify }
    }

    /// Runs an adversarial case through the supplied real verifier adapter.
    pub fn run<Input, E>(
        &self,
        case: AdversarialCase<Input>,
    ) -> Result<AdversarialOutcome, SecurityTestError<E>>
    where
        F: Fn(&Input) -> Result<Acceptance, E>,
    {
        let observed = (self.verify)(&case.input).map_err(SecurityTestError::Backend)?;
        if observed != case.expected {
            return match observed {
                Acceptance::Accepted => Err(SecurityTestError::UnexpectedAcceptance),
                Acceptance::Rejected => Err(SecurityTestError::UnexpectedRejection),
            };
        }

        Ok(AdversarialOutcome {
            name: case.name,
            expected: case.expected,
            observed,
        })
    }
}
