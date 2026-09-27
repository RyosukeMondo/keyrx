//! Macro recording operation error types.

use thiserror::Error;

/// Macro recording operation errors.
///
/// This error type covers failures in macro recording and playback operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RecorderError {
    /// Attempted to stop recording when not currently recording.
    #[error("Not currently recording")]
    NotRecording,

    /// Attempted to start recording when already recording.
    #[error("Already recording")]
    AlreadyRecording,

    /// Playback failed at the specified frame.
    #[error("Playback failed at frame {0}")]
    PlaybackFailed(usize),

    /// Recording buffer is full.
    #[error("Recording buffer full (max {0} events)")]
    BufferFull(usize),

    /// Mutex poisoned during recorder operation.
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),
}
