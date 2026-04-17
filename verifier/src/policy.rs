use crate::engine::PublicInputs;
use zkcg_common::errors::ProtocolError;

pub fn enforce(inputs: &PublicInputs) -> Result<(), ProtocolError> {
    if let Some(phase1) = inputs.phase1() {
        if phase1.threshold == 0 {
            return Err(ProtocolError::PolicyViolation);
        }
    }

    Ok(())
}
