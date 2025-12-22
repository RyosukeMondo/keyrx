use std::path::PathBuf;

/// Represents a single step in the import chain.
/// Each step records the file and the line number where the import occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStep {
    pub file: PathBuf,
    pub line: usize,
}

/// Errors that can occur during Rhai script parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by parser module
pub enum ParseError {
    /// Syntax error in Rhai script.
    SyntaxError {
        file: PathBuf,
        line: usize,
        column: usize,
        message: String,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Invalid prefix used (expected VK_/MD_/LK_, got something else).
    InvalidPrefix {
        expected: String,
        got: String,
        context: String,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Custom modifier ID out of valid range (00-FE).
    ModifierIdOutOfRange {
        got: u16,
        max: u8,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Custom lock ID out of valid range (00-FE).
    LockIdOutOfRange {
        got: u16,
        max: u8,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Physical modifier name used in MD_ prefix (e.g., MD_LShift).
    PhysicalModifierInMD {
        name: String,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Required prefix missing from key name.
    MissingPrefix {
        key: String,
        context: String,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Import file not found.
    ImportNotFound {
        path: PathBuf,
        searched_paths: Vec<PathBuf>,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },

    /// Circular import detected.
    CircularImport { chain: Vec<PathBuf> },

    /// Resource limit exceeded (operations, depth, etc.).
    ResourceLimitExceeded {
        limit_type: String,
        /// Import chain leading to this error (empty if error is in main file)
        import_chain: Vec<ImportStep>,
    },
}

/// Errors that can occur during serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by serialize module
pub enum SerializeError {
    /// rkyv serialization error.
    RkyvError(String),

    /// I/O error during file operations.
    IoError(String),
}

/// Errors that can occur during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Will be used by serialize module
pub enum DeserializeError {
    /// Invalid magic bytes (expected "KRX\n").
    InvalidMagic { expected: [u8; 4], got: [u8; 4] },

    /// Version mismatch.
    VersionMismatch { expected: u32, got: u32 },

    /// Hash mismatch (data corruption detected).
    HashMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
    },

    /// rkyv deserialization error.
    RkyvError(String),

    /// I/O error during file operations.
    IoError(String),
}
