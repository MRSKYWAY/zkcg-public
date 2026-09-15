/// Deterministic proof-byte mutations useful for negative verifier tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalformedProofMutation {
    Truncate(usize),
    FlipBit { offset: usize, bit: u8 },
    Append(Vec<u8>),
    Replace { offset: usize, byte: u8 },
    Empty,
}

pub fn mutate_proof(proof: &[u8], mutation: MalformedProofMutation) -> Vec<u8> {
    match mutation {
        MalformedProofMutation::Truncate(len) => proof[..len.min(proof.len())].to_vec(),
        MalformedProofMutation::FlipBit { offset, bit } => {
            let mut out = proof.to_vec();
            if let Some(byte) = out.get_mut(offset) {
                *byte ^= 1u8.wrapping_shl((bit & 7) as u32);
            }
            out
        }
        MalformedProofMutation::Append(bytes) => {
            let mut out = proof.to_vec();
            out.extend_from_slice(&bytes);
            out
        }
        MalformedProofMutation::Replace { offset, byte } => {
            let mut out = proof.to_vec();
            if let Some(slot) = out.get_mut(offset) {
                *slot = byte;
            }
            out
        }
        MalformedProofMutation::Empty => Vec::new(),
    }
}
