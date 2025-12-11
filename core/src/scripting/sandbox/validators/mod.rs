//! Common validators for script function inputs.
//!
//! This module provides reusable validators that implement the `InputValidator`
//! trait. These validators can be composed using the `and()` and `or()` methods.

mod keycode;
mod range;
mod string;

pub use keycode::KeyCodeValidator;
pub use range::{NonNegativeValidator, PositiveValidator, RangeValidator};
pub use string::{LengthValidator, NonEmptyValidator, PatternValidator};
