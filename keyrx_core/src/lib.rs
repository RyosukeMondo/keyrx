#![no_std]

//! keyrx_core - Platform-agnostic keyboard remapping logic
//!
//! This crate contains the core remapping engine that is OS-agnostic and WASM-compatible.
//! It uses no_std to ensure it can be compiled to any target, including browser WASM.

extern crate alloc;

pub mod config;
pub mod dfa;
pub mod lookup;
pub mod simulator;
pub mod state;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
