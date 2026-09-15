#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Consumers should route bytes through the same proof-system dispatch used
    // by production verification and assert that malformed input never panics.
    let _ = data.len();
});
