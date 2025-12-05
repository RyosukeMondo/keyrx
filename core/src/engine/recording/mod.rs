//! Recording subsystem utilities.

pub mod format;

pub use format::{
    BlockIndex, CompressionKind, FormatError, RecordingHeader, RECORDING_HEADER_SIZE,
    RECORDING_MAGIC, RECORDING_VERSION,
};
