//! Runtime error definitions.
//!
//! This module defines all errors related to runtime engine operations,
//! script execution, event processing, and state management.
//! Runtime errors use the KRX-R2xxx range.

use crate::define_errors;

define_errors! {
    category: Runtime,
    base: 2000,

    errors: {
        ENGINE_NOT_INITIALIZED = 1 => {
            message: "Engine not initialized",
            hint: "Call engine initialization before starting event processing",
            severity: Error,
        },

        ENGINE_ALREADY_RUNNING = 2 => {
            message: "Engine is already running",
            hint: "Stop the current engine instance before starting a new one",
            severity: Error,
        },

        ENGINE_START_FAILED = 3 => {
            message: "Failed to start engine: {reason}",
            hint: "Check system resources and driver availability. See logs for details",
            severity: Error,
        },

        ENGINE_STOP_FAILED = 4 => {
            message: "Failed to stop engine cleanly: {reason}",
            hint: "Some resources may not have been released. Consider restarting the application",
            severity: Warning,
        },

        EVENT_PROCESSING_FAILED = 5 => {
            message: "Event processing failed: {reason}",
            hint: "Check your configuration and script for errors. Event may be dropped",
            severity: Error,
        },

        EVENT_QUEUE_FULL = 6 => {
            message: "Event queue is full (capacity: {capacity})",
            hint: "Events are arriving faster than they can be processed. Consider reducing input rate or optimizing processing",
            severity: Warning,
        },

        INVALID_EVENT_DATA = 7 => {
            message: "Invalid event data: {reason}",
            hint: "Event data is corrupted or malformed. This may indicate a driver issue",
            severity: Error,
        },

        SCRIPT_EXECUTION_FAILED = 8 => {
            message: "Script execution failed: {error}",
            hint: "Check your script for syntax errors or runtime issues. See error details above",
            severity: Error,
        },

        SCRIPT_COMPILATION_FAILED = 9 => {
            message: "Script compilation failed: {error}",
            hint: "Fix syntax errors in your script. Check line and column numbers in the error message",
            severity: Error,
        },

        SCRIPT_TIMEOUT = 10 => {
            message: "Script execution timed out after {timeout_ms}ms",
            hint: "Optimize your script or increase the timeout limit. Avoid infinite loops",
            severity: Error,
        },

        SCRIPT_HOOK_NOT_FOUND = 11 => {
            message: "Script hook '{hook}' not defined",
            hint: "Define the required hook function in your script or remove the hook reference",
            severity: Warning,
        },

        SCRIPT_INVALID_RETURN_TYPE = 12 => {
            message: "Script function '{function}' returned invalid type: expected {expected}, got {actual}",
            hint: "Fix the return type of the function to match the expected type",
            severity: Error,
        },

        SCRIPT_OPERATION_LIMIT_EXCEEDED = 13 => {
            message: "Script exceeded operation limit of {limit} operations",
            hint: "Simplify your script or reduce the number of operations. This is a safety limit",
            severity: Error,
        },

        LAYER_NOT_FOUND = 14 => {
            message: "Layer {layer_id} not found",
            hint: "Check that the layer is defined in your configuration before referencing it",
            severity: Error,
        },

        LAYER_STACK_OVERFLOW = 15 => {
            message: "Layer stack overflow: maximum depth {max_depth} exceeded",
            hint: "Too many nested layer activations. Check for layer activation loops",
            severity: Error,
        },

        LAYER_ALREADY_ACTIVE = 16 => {
            message: "Layer {layer_id} is already active",
            hint: "This is informational. Layer was not activated again",
            severity: Info,
        },

        INVALID_LAYER_ID = 17 => {
            message: "Invalid layer ID: {layer_id}",
            hint: "Use a valid layer ID from your configuration",
            severity: Error,
        },

        MODIFIER_STATE_INCONSISTENT = 18 => {
            message: "Modifier state inconsistent: {reason}",
            hint: "This may indicate a bug or driver issue. Try restarting the engine",
            severity: Warning,
        },

        STATE_CORRUPTION_DETECTED = 19 => {
            message: "Engine state corruption detected: {reason}",
            hint: "This is a serious error. Save your work and restart the application",
            severity: Fatal,
        },

        STATE_SAVE_FAILED = 20 => {
            message: "Failed to save engine state: {reason}",
            hint: "Check file permissions and disk space",
            severity: Error,
        },

        STATE_LOAD_FAILED = 21 => {
            message: "Failed to load engine state: {reason}",
            hint: "State file may be corrupted or incompatible. Starting with default state",
            severity: Warning,
        },

        SESSION_RECORDING_FAILED = 22 => {
            message: "Session recording failed: {reason}",
            hint: "Check write permissions and disk space. Recording will be incomplete",
            severity: Error,
        },

        SESSION_REPLAY_FAILED = 23 => {
            message: "Session replay failed: {reason}",
            hint: "Session file may be corrupted or incompatible with this version",
            severity: Error,
        },

        SESSION_FILE_CORRUPT = 24 => {
            message: "Session file is corrupted: {path}. Parse error: {error}",
            hint: "The session file cannot be read. It may be incomplete or damaged or unparsable",
            severity: Error,
        },

        SESSION_VERSION_MISMATCH = 25 => {
            message: "Session file version mismatch: found {found}, expected {expected}",
            hint: "This session was recorded with a different version. It may not replay correctly",
            severity: Warning,
        },

        SESSION_NOT_RECORDING = 26 => {
            message: "Session is not currently recording",
            hint: "Start recording before attempting to stop or save",
            severity: Error,
        },

        SESSION_ALREADY_RECORDING = 27 => {
            message: "Session is already recording",
            hint: "Stop the current recording before starting a new one",
            severity: Error,
        },

        REPLAY_COMPLETED = 28 => {
            message: "Replay session completed",
            hint: "All recorded events have been replayed",
            severity: Info,
        },

        REPLAY_NOT_STARTED = 29 => {
            message: "Replay has not been started",
            hint: "Call start() before attempting to replay events",
            severity: Error,
        },

        REMAP_NOT_FOUND = 30 => {
            message: "Remap not found for key {key}",
            hint: "This key has no remap defined. It will pass through unchanged",
            severity: Info,
        },

        COMBO_TIMEOUT = 31 => {
            message: "Combo sequence timed out",
            hint: "Complete the combo sequence faster or increase the timeout setting",
            severity: Info,
        },

        COMBO_BUFFER_OVERFLOW = 32 => {
            message: "Combo buffer overflow: maximum {max_keys} keys exceeded",
            hint: "Simplify your combo definitions or increase the buffer size",
            severity: Warning,
        },

        TAP_HOLD_TIMEOUT_EXCEEDED = 33 => {
            message: "Tap-hold timeout exceeded for key {key}",
            hint: "Key was held longer than tap_timeout. Treating as hold",
            severity: Info,
        },

        INVALID_OUTPUT_ACTION = 34 => {
            message: "Invalid output action: {reason}",
            hint: "Check your configuration for invalid key codes or actions",
            severity: Error,
        },

        OUTPUT_INJECTION_FAILED = 35 => {
            message: "Failed to inject output event: {reason}",
            hint: "Driver may not be available or permissions may be insufficient",
            severity: Error,
        },

        DEADLOCK_DETECTED = 36 => {
            message: "Potential deadlock detected in {component}",
            hint: "This is a critical error. Save your work and restart immediately",
            severity: Fatal,
        },

        THREAD_PANIC = 37 => {
            message: "Worker thread panicked: {thread}",
            hint: "An internal error occurred. Check logs for panic details and report this bug",
            severity: Fatal,
        },

        RESOURCE_EXHAUSTED = 38 => {
            message: "Resource exhausted: {resource}",
            hint: "System resources are depleted. Close other applications or increase limits",
            severity: Error,
        },

        OPERATION_CANCELLED = 39 => {
            message: "Operation was cancelled: {operation}",
            hint: "This is expected if you cancelled the operation manually",
            severity: Info,
        },

        OPERATION_TIMEOUT = 40 => {
            message: "Operation timed out after {timeout_ms}ms: {operation}",
            hint: "The operation took too long. Try again or increase the timeout",
            severity: Error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCategory;
    use crate::keyrx_err;

    #[test]
    fn runtime_error_codes_in_range() {
        assert_eq!(ENGINE_NOT_INITIALIZED.code().number(), 2001);
        assert_eq!(SCRIPT_EXECUTION_FAILED.code().number(), 2008);
        assert_eq!(OPERATION_TIMEOUT.code().number(), 2040);

        // Verify all are in Runtime category range
        assert!(ErrorCategory::Runtime.contains(ENGINE_NOT_INITIALIZED.code().number()));
        assert!(ErrorCategory::Runtime.contains(OPERATION_TIMEOUT.code().number()));
    }

    #[test]
    fn runtime_error_categories() {
        assert_eq!(
            ENGINE_NOT_INITIALIZED.code().category(),
            ErrorCategory::Runtime
        );
        assert_eq!(
            SCRIPT_EXECUTION_FAILED.code().category(),
            ErrorCategory::Runtime
        );
        assert_eq!(
            SESSION_REPLAY_FAILED.code().category(),
            ErrorCategory::Runtime
        );
    }

    #[test]
    fn runtime_error_messages() {
        let err = keyrx_err!(ENGINE_START_FAILED, reason = "driver not available");
        assert_eq!(err.code(), "KRX-R2003");
        assert!(err.message().contains("driver not available"));
    }

    #[test]
    fn runtime_error_hints() {
        assert!(ENGINE_NOT_INITIALIZED.hint().is_some());
        assert!(SCRIPT_EXECUTION_FAILED
            .hint()
            .unwrap()
            .contains("syntax errors"));
        assert!(LAYER_STACK_OVERFLOW.hint().unwrap().contains("loops"));
    }

    #[test]
    fn runtime_error_severities() {
        use crate::errors::ErrorSeverity;

        assert_eq!(ENGINE_NOT_INITIALIZED.severity(), ErrorSeverity::Error);
        assert_eq!(LAYER_ALREADY_ACTIVE.severity(), ErrorSeverity::Info);
        assert_eq!(ENGINE_STOP_FAILED.severity(), ErrorSeverity::Warning);
        assert_eq!(STATE_CORRUPTION_DETECTED.severity(), ErrorSeverity::Fatal);
    }

    #[test]
    fn runtime_error_formatting() {
        let err = keyrx_err!(SESSION_VERSION_MISMATCH, found = "2", expected = "1");
        assert!(err.message().contains("2"));
        assert!(err.message().contains("1"));
        assert!(err.message().contains("version mismatch"));
    }

    #[test]
    fn runtime_error_context_substitution() {
        let err = keyrx_err!(
            SCRIPT_INVALID_RETURN_TYPE,
            function = "on_key",
            expected = "bool",
            actual = "string"
        );
        assert_eq!(err.code(), "KRX-R2012");
        assert!(err.message().contains("on_key"));
        assert!(err.message().contains("bool"));
        assert!(err.message().contains("string"));
    }

    #[test]
    fn runtime_script_errors() {
        let timeout_err = keyrx_err!(SCRIPT_TIMEOUT, timeout_ms = "1000");
        assert!(timeout_err.message().contains("1000"));

        let hook_err = keyrx_err!(SCRIPT_HOOK_NOT_FOUND, hook = "on_custom_event");
        assert!(hook_err.message().contains("on_custom_event"));
    }

    #[test]
    fn runtime_layer_errors() {
        let layer_err = keyrx_err!(LAYER_NOT_FOUND, layer_id = "42");
        assert!(layer_err.message().contains("42"));

        let overflow_err = keyrx_err!(LAYER_STACK_OVERFLOW, max_depth = "8");
        assert!(overflow_err.message().contains("8"));
    }

    #[test]
    fn runtime_session_errors() {
        let replay_err = keyrx_err!(SESSION_REPLAY_FAILED, reason = "unexpected EOF");
        assert!(replay_err.message().contains("unexpected EOF"));

        let corrupt_err = keyrx_err!(
            SESSION_FILE_CORRUPT,
            path = "/tmp/session.krx",
            error = "test error"
        );
        assert!(corrupt_err.message().contains("/tmp/session.krx"));
        assert!(corrupt_err.message().contains("test error"));
    }
}
