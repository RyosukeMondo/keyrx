//! Test example for rhai_doc macro

use keyrx_core_macros::rhai_doc;
use rhai::EvalAltResult;

/// Test function to verify rhai_doc macro works
///
/// This is a simple test function that demonstrates the rhai_doc macro.
///
/// # Parameters
/// * `name` - The name to greet
/// * `count` - Number of times to repeat the greeting
///
/// # Returns
/// Returns a greeting string
///
/// # Examples
/// ```rhai
/// let msg = test_greet("World", 1);
/// print(msg);
/// ```
///
/// # Notes
/// This is just a test function
#[rhai_doc(module = "test", since = "0.1.0")]
#[allow(dead_code)]
pub fn test_greet(name: &str, count: i64) -> Result<String, Box<EvalAltResult>> {
    Ok(format!("Hello, {}! (x{})", name, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::registry;

    #[test]
    #[serial_test::serial]
    fn test_macro_generates_doc() {
        // Initialize registry
        registry::initialize();
        registry::clear(); // Clear any previous state

        // Call the generated registration function
        __register_doc_test_greet();

        // Verify the documentation was registered
        let doc = registry::get_function("test", "test_greet");
        assert!(doc.is_some(), "Documentation should be registered");

        let doc = doc.unwrap();
        assert_eq!(doc.name, "test_greet");
        assert_eq!(doc.module, "test");
        assert!(doc.description.contains("test function"));
        assert_eq!(doc.parameters.len(), 2);
        assert_eq!(doc.parameters[0].name, "name");
        assert_eq!(doc.parameters[1].name, "count");
        assert!(doc.returns.is_some());
        assert_eq!(doc.examples.len(), 1);
        assert!(doc.notes.is_some());
        assert_eq!(doc.since, Some("0.1.0".to_string()));

        registry::clear();
    }
}
