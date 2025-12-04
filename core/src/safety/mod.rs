//! Safety utilities for panic recovery and circuit breaking.
//!
//! This module provides infrastructure for making KeyRx resilient to panics
//! and failures in critical paths. It includes:
//!
//! - `panic_guard`: Panic catching and backtrace logging
//! - `circuit_breaker`: Circuit breakers for preventing cascading failures
//! - `extensions`: Extension traits for safe unwrapping and error conversion

pub mod circuit_breaker;
pub mod extensions;
pub mod panic_guard;
