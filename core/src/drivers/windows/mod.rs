mod device;
mod hook;
mod hook_thread;
mod injector;
mod input;
mod input_source;
mod keymap;
pub mod raw_input;

/// Safety wrappers for Windows driver operations.
///
/// Contains safe abstractions over unsafe Windows API calls.
pub mod safety;

pub use device::list_keyboards;
pub use injector::SendInputInjector;
pub use input::WindowsInput;
pub use raw_input::WindowsRawInput;

#[cfg(test)]
mod mod_tests;
