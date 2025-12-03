//! FFI domain implementations.
//!
//! Each module in this directory implements the FfiExportable trait for a specific
//! domain of functionality (discovery, validation, engine, etc.).

pub mod analysis;
pub mod device;
pub mod diagnostics;
pub mod discovery;
pub mod engine;
pub mod recording;
pub mod script;
pub mod testing;
pub mod validation;

pub use analysis::AnalysisFfi;
pub use device::DeviceFfi;
pub use diagnostics::DiagnosticsFfi;
pub use discovery::DiscoveryFfi;
pub use engine::EngineFfi;
pub use recording::RecordingFfi;
pub use script::ScriptFfi;
pub use testing::TestingFfi;
pub use validation::ValidationFfi;
