use crate::{Acceptance, SecurityTestError};

/// A backend that returns only the security-relevant verification decision.
pub trait VerificationBackend<Input> {
    type Error;
    fn verify(&self, input: &Input) -> Result<Acceptance, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct DifferentialCase<Input> {
    pub name: String,
    pub input: Input,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferentialOutcome {
    pub name: String,
    pub oracle: Acceptance,
    pub backend_a: Acceptance,
    pub backend_b: Acceptance,
}

pub struct DifferentialHarness<A, B, O> {
    backend_a: A,
    backend_b: B,
    oracle: O,
}

impl<A, B, O> DifferentialHarness<A, B, O> {
    pub fn new(backend_a: A, backend_b: B, oracle: O) -> Self {
        Self {
            backend_a,
            backend_b,
            oracle,
        }
    }

    pub fn run<Input, EA, EB, EO>(
        &self,
        case: DifferentialCase<Input>,
    ) -> Result<DifferentialOutcome, SecurityTestError<String>>
    where
        A: VerificationBackend<Input, Error = EA>,
        B: VerificationBackend<Input, Error = EB>,
        O: Fn(&Input) -> Result<Acceptance, EO>,
        EA: core::fmt::Display,
        EB: core::fmt::Display,
        EO: core::fmt::Display,
    {
        let oracle =
            (self.oracle)(&case.input).map_err(|e| SecurityTestError::Backend(e.to_string()))?;
        let backend_a = self
            .backend_a
            .verify(&case.input)
            .map_err(|e| SecurityTestError::Backend(e.to_string()))?;
        let backend_b = self
            .backend_b
            .verify(&case.input)
            .map_err(|e| SecurityTestError::Backend(e.to_string()))?;

        if backend_a != oracle || backend_b != oracle || backend_a != backend_b {
            return Err(SecurityTestError::UnexpectedAcceptance);
        }

        Ok(DifferentialOutcome {
            name: case.name,
            oracle,
            backend_a,
            backend_b,
        })
    }
}
