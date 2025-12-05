//! Binary recording file format description.
//!
//! Layout (little-endian):
//! - 0..4   : magic bytes ("KRR1")
//! - 4..8   : format version
//! - 8      : compression codec (see [`CompressionKind`])
//! - 9..12  : reserved
//! - 12..16 : flags (reserved for future use)
//! - 16..24 : created_at epoch ms
//! - 24..32 : metadata offset (relative to start of file)
//! - 32..36 : metadata length in bytes
//! - 36..44 : index offset (relative to start of file)
//! - 44..48 : index length in bytes
//! - 48..52 : block count
//! - 52..64 : reserved
//! - Blocks : compressed blocks containing serialized events
//! - Index  : table of [`BlockIndex`] entries located at `index_offset`
//!
//! The header uses a fixed 64-byte footprint to support forward-compatible
//! extensions without breaking older readers.

use thiserror::Error;

/// Magic bytes identifying a KeyRx recording stream.
pub const RECORDING_MAGIC: [u8; 4] = *b"KRR1";

/// Current binary format version.
pub const RECORDING_VERSION: u32 = 1;

/// Fixed header size in bytes.
pub const RECORDING_HEADER_SIZE: usize = 64;

/// Size in bytes of a single index entry.
pub const BLOCK_INDEX_SIZE: usize = 56;

/// Compression algorithms supported for blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionKind {
    /// No compression applied.
    None = 0,
    /// Gzip compression (RFC 1952).
    Gzip = 1,
}

impl CompressionKind {
    /// Parse a codec identifier from the on-disk byte.
    pub fn from_byte(byte: u8) -> Result<Self, FormatError> {
        match byte {
            0 => Ok(Self::None),
            1 => Ok(Self::Gzip),
            codec => Err(FormatError::UnsupportedCompression { codec }),
        }
    }

    /// Serialize the codec to its on-disk byte representation.
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Errors encountered while parsing or validating the format.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FormatError {
    /// Header slice shorter than the fixed header size.
    #[error("header too short: found {found} bytes, expected {expected}")]
    HeaderTooShort { found: usize, expected: usize },
    /// Magic bytes do not match the expected signature.
    #[error("invalid magic bytes: {found:?}")]
    InvalidMagic { found: [u8; 4] },
    /// Version newer than the current reader supports.
    #[error("unsupported version {found} (max supported {supported})")]
    UnsupportedVersion { found: u32, supported: u32 },
    /// Compression codec is not recognized.
    #[error("unsupported compression codec: {codec}")]
    UnsupportedCompression { codec: u8 },
    /// Metadata section points before the header or has zero length.
    #[error("invalid metadata section offset {offset} (len {len})")]
    InvalidMetadata { offset: u64, len: u32 },
    /// Index section points before the header or has zero length.
    #[error("invalid index section offset {offset} (len {len})")]
    InvalidIndex { offset: u64, len: u32 },
    /// Index length is not aligned to the entry size.
    #[error("index length {len} is not a multiple of entry size {entry_size}")]
    InvalidIndexLength { len: u32, entry_size: usize },
    /// Declared block count does not match the index payload.
    #[error("declared block count {declared} does not match index entries {actual}")]
    BlockCountMismatch { declared: u32, actual: u32 },
    /// Index entry slice shorter than expected.
    #[error("index entry too short: found {found} bytes, expected {expected}")]
    IndexEntryTooShort { found: usize, expected: usize },
}

/// Fixed-size header describing where metadata, blocks, and index live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingHeader {
    /// Binary format version for forward/backward compatibility.
    pub version: u32,
    /// Compression codec used for all blocks.
    pub compression: CompressionKind,
    /// Reserved for future format flags.
    pub flags: u32,
    /// Creation time in milliseconds since Unix epoch.
    pub created_at_ms: u64,
    /// Offset of the metadata segment from the start of the file.
    pub metadata_offset: u64,
    /// Length of the metadata segment in bytes.
    pub metadata_len: u32,
    /// Offset of the index table from the start of the file.
    pub index_offset: u64,
    /// Length of the index table in bytes.
    pub index_len: u32,
    /// Number of compressed blocks described by the index.
    pub block_count: u32,
}

impl RecordingHeader {
    /// Build a new header with the required fields.
    pub fn new(
        compression: CompressionKind,
        created_at_ms: u64,
        metadata_offset: u64,
        metadata_len: u32,
        index_offset: u64,
        index_len: u32,
        block_count: u32,
    ) -> Self {
        Self {
            version: RECORDING_VERSION,
            compression,
            flags: 0,
            created_at_ms,
            metadata_offset,
            metadata_len,
            index_offset,
            index_len,
            block_count,
        }
    }

    /// Encode the header into a fixed 64-byte array.
    pub fn encode(&self) -> [u8; RECORDING_HEADER_SIZE] {
        let mut buf = [0u8; RECORDING_HEADER_SIZE];
        buf[..4].copy_from_slice(&RECORDING_MAGIC);
        buf[4..8].copy_from_slice(&self.version.to_le_bytes());
        buf[8] = self.compression.as_byte();
        // 9..12 reserved
        buf[12..16].copy_from_slice(&self.flags.to_le_bytes());
        buf[16..24].copy_from_slice(&self.created_at_ms.to_le_bytes());
        buf[24..32].copy_from_slice(&self.metadata_offset.to_le_bytes());
        buf[32..36].copy_from_slice(&self.metadata_len.to_le_bytes());
        buf[36..44].copy_from_slice(&self.index_offset.to_le_bytes());
        buf[44..48].copy_from_slice(&self.index_len.to_le_bytes());
        buf[48..52].copy_from_slice(&self.block_count.to_le_bytes());
        // 52..64 reserved
        buf
    }

    /// Decode a header from a byte slice and validate layout.
    pub fn decode(bytes: &[u8]) -> Result<Self, FormatError> {
        if bytes.len() < RECORDING_HEADER_SIZE {
            return Err(FormatError::HeaderTooShort {
                found: bytes.len(),
                expected: RECORDING_HEADER_SIZE,
            });
        }

        let magic = slice_to_array::<4>(&bytes[0..4]);
        if magic != RECORDING_MAGIC {
            return Err(FormatError::InvalidMagic { found: magic });
        }

        let version = read_u32(&bytes[4..8]);
        if version > RECORDING_VERSION {
            return Err(FormatError::UnsupportedVersion {
                found: version,
                supported: RECORDING_VERSION,
            });
        }

        let compression = CompressionKind::from_byte(bytes[8])?;
        let flags = read_u32(&bytes[12..16]);
        let created_at_ms = read_u64(&bytes[16..24]);
        let metadata_offset = read_u64(&bytes[24..32]);
        let metadata_len = read_u32(&bytes[32..36]);
        let index_offset = read_u64(&bytes[36..44]);
        let index_len = read_u32(&bytes[44..48]);
        let block_count = read_u32(&bytes[48..52]);

        validate_section(
            metadata_offset,
            metadata_len,
            FormatError::InvalidMetadata {
                offset: metadata_offset,
                len: metadata_len,
            },
        )?;
        validate_section(
            index_offset,
            index_len,
            FormatError::InvalidIndex {
                offset: index_offset,
                len: index_len,
            },
        )?;

        if !index_len.is_multiple_of(BLOCK_INDEX_SIZE as u32) {
            return Err(FormatError::InvalidIndexLength {
                len: index_len,
                entry_size: BLOCK_INDEX_SIZE,
            });
        }

        let entry_count = index_len / (BLOCK_INDEX_SIZE as u32);
        if block_count == 0 || block_count != entry_count {
            return Err(FormatError::BlockCountMismatch {
                declared: block_count,
                actual: entry_count,
            });
        }

        Ok(Self {
            version,
            compression,
            flags,
            created_at_ms,
            metadata_offset,
            metadata_len,
            index_offset,
            index_len,
            block_count,
        })
    }
}

/// Metadata about a compressed block within the recording file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockIndex {
    /// Sequential identifier for the block (0-based).
    pub block_id: u32,
    /// Sequence number of the first event in the block.
    pub start_seq: u64,
    /// Sequence number of the last event in the block (inclusive).
    pub end_seq: u64,
    /// Timestamp (µs since session start) of the first event.
    pub start_timestamp_us: u64,
    /// Timestamp (µs since session start) of the last event.
    pub end_timestamp_us: u64,
    /// Byte offset where the block payload begins.
    pub offset: u64,
    /// Compressed payload size in bytes.
    pub compressed_len: u32,
    /// Uncompressed payload size in bytes.
    pub uncompressed_len: u32,
    /// CRC32 of the compressed payload (optional validation).
    pub checksum: u32,
}

impl BlockIndex {
    /// Serialize the index entry to its fixed-size byte representation.
    pub fn encode(&self) -> [u8; BLOCK_INDEX_SIZE] {
        let mut buf = [0u8; BLOCK_INDEX_SIZE];
        buf[0..4].copy_from_slice(&self.block_id.to_le_bytes());
        buf[4..12].copy_from_slice(&self.start_seq.to_le_bytes());
        buf[12..20].copy_from_slice(&self.end_seq.to_le_bytes());
        buf[20..28].copy_from_slice(&self.start_timestamp_us.to_le_bytes());
        buf[28..36].copy_from_slice(&self.end_timestamp_us.to_le_bytes());
        buf[36..44].copy_from_slice(&self.offset.to_le_bytes());
        buf[44..48].copy_from_slice(&self.compressed_len.to_le_bytes());
        buf[48..52].copy_from_slice(&self.uncompressed_len.to_le_bytes());
        buf[52..56].copy_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    /// Deserialize an index entry from bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, FormatError> {
        if bytes.len() < BLOCK_INDEX_SIZE {
            return Err(FormatError::IndexEntryTooShort {
                found: bytes.len(),
                expected: BLOCK_INDEX_SIZE,
            });
        }

        Ok(Self {
            block_id: read_u32(&bytes[0..4]),
            start_seq: read_u64(&bytes[4..12]),
            end_seq: read_u64(&bytes[12..20]),
            start_timestamp_us: read_u64(&bytes[20..28]),
            end_timestamp_us: read_u64(&bytes[28..36]),
            offset: read_u64(&bytes[36..44]),
            compressed_len: read_u32(&bytes[44..48]),
            uncompressed_len: read_u32(&bytes[48..52]),
            checksum: read_u32(&bytes[52..56]),
        })
    }
}

fn read_u32(bytes: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(bytes);
    u32::from_le_bytes(buf)
}

fn read_u64(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

fn slice_to_array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut array = [0u8; N];
    array.copy_from_slice(bytes);
    array
}

fn validate_section(offset: u64, len: u32, err: FormatError) -> Result<(), FormatError> {
    if len == 0 || offset < RECORDING_HEADER_SIZE as u64 {
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let header = RecordingHeader::new(
            CompressionKind::Gzip,
            1_700_000_000_000,
            RECORDING_HEADER_SIZE as u64,
            512,
            2048,
            (BLOCK_INDEX_SIZE as u32) * 2,
            2,
        );

        let encoded = header.encode();
        let decoded = RecordingHeader::decode(&encoded).expect("decode");

        assert_eq!(decoded.version, RECORDING_VERSION);
        assert_eq!(decoded.compression, CompressionKind::Gzip);
        assert_eq!(decoded.metadata_offset, RECORDING_HEADER_SIZE as u64);
        assert_eq!(decoded.index_offset, 2048);
        assert_eq!(decoded.block_count, 2);
    }

    #[test]
    fn rejects_bad_magic_and_version() {
        let mut encoded = RecordingHeader::new(
            CompressionKind::None,
            0,
            RECORDING_HEADER_SIZE as u64,
            64,
            128,
            BLOCK_INDEX_SIZE as u32,
            1,
        )
        .encode();

        // Corrupt magic
        encoded[0] = 0x00;
        let err = RecordingHeader::decode(&encoded).unwrap_err();
        assert!(matches!(err, FormatError::InvalidMagic { .. }));

        // Restore and bump version
        encoded[..4].copy_from_slice(&RECORDING_MAGIC);
        encoded[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        let err = RecordingHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            FormatError::UnsupportedVersion { found, .. } if found == u32::MAX
        ));
    }

    #[test]
    fn detects_misaligned_index() {
        let mut encoded = RecordingHeader::new(
            CompressionKind::None,
            0,
            RECORDING_HEADER_SIZE as u64,
            64,
            256,
            10, // not a multiple of BLOCK_INDEX_SIZE
            1,
        )
        .encode();

        let err = RecordingHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            FormatError::InvalidIndexLength {
                len: 10,
                entry_size: BLOCK_INDEX_SIZE
            }
        ));

        // Fix length but mismatch block count
        encoded[44..48].copy_from_slice(&(BLOCK_INDEX_SIZE as u32 * 2).to_le_bytes());
        let err = RecordingHeader::decode(&encoded).unwrap_err();
        assert!(matches!(
            err,
            FormatError::BlockCountMismatch {
                declared: 1,
                actual: 2
            }
        ));
    }

    #[test]
    fn block_index_roundtrip() {
        let entry = BlockIndex {
            block_id: 7,
            start_seq: 100,
            end_seq: 149,
            start_timestamp_us: 1_000_000,
            end_timestamp_us: 1_250_000,
            offset: 4096,
            compressed_len: 8192,
            uncompressed_len: 16_384,
            checksum: 0xDEADBEEF,
        };

        let encoded = entry.encode();
        let decoded = BlockIndex::decode(&encoded).expect("decode entry");

        assert_eq!(decoded, entry);
    }

    #[test]
    fn rejects_short_index_entry() {
        let buf = vec![0u8; BLOCK_INDEX_SIZE - 1];
        let err = BlockIndex::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            FormatError::IndexEntryTooShort {
                found,
                expected: BLOCK_INDEX_SIZE
            } if found == BLOCK_INDEX_SIZE - 1
        ));
    }
}
