#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Consumers should decode their public-input type and pass it through their
    // real verifier adapter here. The template deliberately avoids inventing a
    // ZKCG-specific wire format.
    let _ = data.len();
});
