#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz target for rkyv deserialization with CheckBytes validation
    //
    // This target feeds random bytes to check_archived_root to ensure it never panics
    // on ANY input, even malformed or adversarially crafted data.
    //
    // The CheckBytes trait implementation should safely reject invalid data and return
    // errors instead of panicking, crashing, or causing undefined behavior.
    //
    // Success criteria:
    // - No panics on malformed data
    // - No undefined behavior (UB) on invalid archives
    // - Graceful error returns for corrupted structures
    // - No memory safety violations

    // Attempt to validate the archive using CheckBytes
    // This should NEVER panic, even on completely random data
    let _ = rkyv::check_archived_root::<keyrx_core::config::ConfigRoot>(data);

    // If check_archived_root returns Ok, the archive is valid and safe to use
    // If it returns Err, the data is invalid and was safely rejected
    // Either outcome is acceptable - the important part is NO PANICS
});
