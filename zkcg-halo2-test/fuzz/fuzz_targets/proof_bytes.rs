#![no_main]

use libfuzzer_sys::fuzz_target;
use zkcg_halo2_test::{mutate_proof, MalformedProofMutation};

fuzz_target!(|data: &[u8]| {
    let _ = mutate_proof(data, MalformedProofMutation::Empty);
    let _ = mutate_proof(data, MalformedProofMutation::Truncate(data.len() / 2));
    let _ = mutate_proof(data, MalformedProofMutation::FlipBit { offset: 0, bit: 0 });
    let _ = mutate_proof(data, MalformedProofMutation::Append(vec![0x00]));
});
