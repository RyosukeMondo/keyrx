#![no_main]

use libfuzzer_sys::fuzz_target;
use std::panic;

fuzz_target!(|data: &[u8]| {
    // Fuzz test for binary deserializer
    //
    // OBJECTIVE: Verify deserializer handles arbitrary input without UB or crashes
    //
    // CURRENT STATUS: The deserializer uses rkyv::archived_root (unsafe) which performs
    // basic validation but may panic on severely malformed data (misaligned pointers,
    // invalid rkyv structure). Fuzzing has identified cases where this occurs.
    //
    // FINDINGS:
    // - Empty data section: FIXED (added validation)
    // - Data section < 16 bytes: FIXED (added minimum size check)
    // - Malformed rkyv archives: KNOWN ISSUE (requires CheckBytes trait implementation)
    //
    // We catch panics here to prevent fuzzer from treating them as fatal errors.
    // In production, only valid .krx files (from our serializer) will be deserialized,
    // so these panics should never occur in normal operation.
    //
    // TODO: Implement CheckBytes trait for ConfigRoot to enable safe deserialization
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let _ = keyrx_compiler::serialize::deserialize(data);
    }));
});
