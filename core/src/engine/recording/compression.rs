//! Compression helpers for recording blocks.
//!
//! Provides block-level compression and streaming decompression so recording
//! writers and replayers can operate without loading full payloads into memory.

use std::io::{self, Cursor, Read, Write};

use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzipLevel};
use thiserror::Error;

use super::CompressionKind;

/// Errors that can occur during compression or decompression.
#[derive(Debug, Error)]
pub enum CompressionError {
    /// A codec was requested that the current build does not implement.
    #[error("compression codec not implemented: {codec:?}")]
    UnsupportedCodec { codec: CompressionKind },
    /// I/O error surfaced from the compression backend.
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type CompressionResult<T> = Result<T, CompressionError>;

/// Compress an entire payload into a new owned buffer.
pub fn compress_block(codec: CompressionKind, payload: &[u8]) -> CompressionResult<Vec<u8>> {
    match codec {
        CompressionKind::None => Ok(payload.to_vec()),
        CompressionKind::Gzip => {
            let mut encoder = GzEncoder::new(Vec::new(), GzipLevel::fast());
            encoder.write_all(payload)?;
            Ok(encoder.finish()?)
        }
    }
}

/// Decompress a full payload into a new owned buffer.
pub fn decompress_block(codec: CompressionKind, payload: &[u8]) -> CompressionResult<Vec<u8>> {
    let mut reader = BlockDecoder::new(codec, Cursor::new(payload))?;
    let mut out = Vec::new();
    reader.read_to_end(&mut out)?;
    Ok(out)
}

/// Streaming decoder that yields decompressed bytes from an underlying reader.
pub struct BlockDecoder<R: Read> {
    inner: DecoderImpl<R>,
}

enum DecoderImpl<R: Read> {
    Passthrough(R),
    Gzip(GzDecoder<R>),
}

impl<R: Read> BlockDecoder<R> {
    /// Create a new streaming decoder for the given codec.
    pub fn new(codec: CompressionKind, reader: R) -> CompressionResult<Self> {
        let inner = match codec {
            CompressionKind::None => DecoderImpl::Passthrough(reader),
            CompressionKind::Gzip => DecoderImpl::Gzip(GzDecoder::new(reader)),
        };

        Ok(Self { inner })
    }
}

impl<R: Read> Read for BlockDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            DecoderImpl::Passthrough(reader) => reader.read(buf),
            DecoderImpl::Gzip(reader) => reader.read(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_none_passthrough() {
        let input = b"no compression payload";
        let compressed = compress_block(CompressionKind::None, input).expect("compress");
        assert_eq!(compressed, input);

        let decompressed =
            decompress_block(CompressionKind::None, &compressed).expect("decompress");
        assert_eq!(decompressed, input);
    }

    #[test]
    fn roundtrip_gzip_block() {
        let input = b"gzip compression payload that should compress";
        let compressed = compress_block(CompressionKind::Gzip, input).expect("compress");
        assert!(!compressed.is_empty());

        let decompressed =
            decompress_block(CompressionKind::Gzip, &compressed).expect("decompress");
        assert_eq!(decompressed, input);
    }

    #[test]
    fn stream_decode_gzip() {
        let input = b"streaming gzip block decode should work across multiple reads";
        let compressed = compress_block(CompressionKind::Gzip, input).expect("compress");

        let mut decoder =
            BlockDecoder::new(CompressionKind::Gzip, Cursor::new(compressed)).expect("decoder");
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).expect("read");

        assert_eq!(buf, input);
    }
}
