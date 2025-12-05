//! Recording subsystem utilities.

pub mod compression;
pub mod format;
pub mod recorder;

pub use compression::{
    compress_block, decompress_block, BlockDecoder, CompressionError, CompressionResult,
};
pub use format::{
    BlockIndex, CompressionKind, FormatError, RecordingHeader, BLOCK_INDEX_SIZE,
    RECORDING_HEADER_SIZE, RECORDING_MAGIC, RECORDING_VERSION,
};
pub use recorder::{
    RecorderError, RecordingMetadata, RecordingSummary, SessionRecorder, MAX_BLOCK_UNCOMPRESSED,
};
