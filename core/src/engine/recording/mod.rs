//! Recording subsystem utilities.

pub mod compression;
pub mod format;

pub use compression::{
    compress_block, decompress_block, BlockDecoder, CompressionError, CompressionResult,
};
pub use format::{
    BlockIndex, CompressionKind, FormatError, RecordingHeader, RECORDING_HEADER_SIZE,
    RECORDING_MAGIC, RECORDING_VERSION,
};
