//! Usage text for the DSL functions, shared by both parsers.
//!
//! Rhai reports a call with the wrong number or type of arguments as
//! `Function not found: tap_hold (&str | ImmutableString | String, ...)`,
//! which tells a user nothing about what the function wants.
//! [`explain_function_not_found`] turns that into `tap_hold needs 4 arguments
//! (key, tap, hold, threshold_ms), got 3`.

use alloc::format;
use alloc::string::{String, ToString};

/// One DSL function: its name and the accepted argument lists (one entry
/// per overload).
struct Usage {
    name: &'static str,
    overloads: &'static [&'static [&'static str]],
}

const USAGES: &[Usage] = &[
    Usage {
        name: "device_start",
        overloads: &[&["pattern"]],
    },
    Usage {
        name: "device_end",
        overloads: &[&[]],
    },
    Usage {
        name: "map",
        overloads: &[&["from", "to"]],
    },
    Usage {
        name: "tap_hold",
        overloads: &[&["key", "tap", "hold", "threshold_ms"]],
    },
    Usage {
        name: "tap_hold_timeout_only",
        overloads: &[&["key", "tap", "hold", "threshold_ms"]],
    },
    Usage {
        name: "hold_only",
        overloads: &[&["key", "hold"], &["key", "hold", "threshold_ms"]],
    },
    Usage {
        name: "one_shot",
        overloads: &[&["key", "modifier"], &["key", "modifier", "timeout_ms"]],
    },
    Usage {
        name: "sequence",
        overloads: &[&["key", "[output keys]"]],
    },
    Usage {
        name: "when_start",
        overloads: &[&["condition"], &["[conditions]"]],
    },
    Usage {
        name: "when_not_start",
        overloads: &[&["condition"]],
    },
    Usage {
        name: "when_device_start",
        overloads: &[&["pattern"]],
    },
];

/// "`name(a, b)` or `name(a, b, c)`" for `name`, if it is a DSL function.
pub fn usage_of(name: &str) -> Option<String> {
    let usage = USAGES.iter().find(|u| u.name == name)?;
    let forms: alloc::vec::Vec<String> = usage
        .overloads
        .iter()
        .map(|args| format!("{}({})", name, args.join(", ")))
        .collect();
    Some(forms.join(" or "))
}

/// Rewrites Rhai's `Function not found: <signature>` for a DSL function
/// into a message naming the expected arguments. `signature` is the text
/// after `Function not found: `, e.g. `tap_hold (&str | String, &str)`.
/// `None` when it is not one of our functions (leave Rhai's message alone).
pub fn explain_function_not_found(signature: &str) -> Option<String> {
    let name = signature.split(" (").next()?.trim();
    let usage = USAGES.iter().find(|u| u.name == name)?;
    let args = signature
        .split_once(" (")
        .map(|(_, rest)| rest.trim_end_matches(')'))
        .unwrap_or("");
    let got = if args.trim().is_empty() {
        0
    } else {
        args.split(',').count()
    };
    let expected = usage_of(name)?;
    let arities_match = usage.overloads.iter().any(|o| o.len() == got);
    Some(if arities_match {
        format!(
            "{name}: an argument has the wrong type (keys and patterns are quoted \"strings\", \
             times are whole numbers). Usage: {expected}"
        )
    } else {
        let counts: alloc::vec::Vec<String> = usage
            .overloads
            .iter()
            .map(|o| o.len().to_string())
            .collect();
        format!(
            "{name} needs {} argument(s), got {got}. Usage: {expected}",
            counts.join(" or ")
        )
    })
}

/// Strips Rhai's own decoration from an error message: the `Syntax error: `
/// / `Runtime error: ` prefixes (the compiler adds its own label) and the
/// trailing `(line N, position M)` (the location is reported separately).
pub fn clean_rhai_message(message: &str) -> String {
    let mut text = message.trim();
    while let Some(rest) = ["Syntax error: ", "Runtime error: ", "Error: "]
        .iter()
        .find_map(|p| text.strip_prefix(p))
    {
        text = rest;
    }
    if let Some(idx) = text.rfind(" (line ") {
        if text.ends_with(')') && text[idx..].contains("position") {
            text = &text[..idx];
        }
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_arity_names_the_expected_arguments() {
        let msg = explain_function_not_found("tap_hold (&str | ImmutableString | String, &str | ImmutableString | String, &str | ImmutableString | String)").unwrap();
        assert!(msg.contains("tap_hold needs 4 argument(s), got 3"), "{msg}");
        assert!(
            msg.contains("tap_hold(key, tap, hold, threshold_ms)"),
            "{msg}"
        );
    }

    #[test]
    fn right_arity_wrong_type_says_so() {
        let msg = explain_function_not_found("map (i64, &str)").unwrap();
        assert!(msg.contains("wrong type"), "{msg}");
    }

    #[test]
    fn overloads_are_listed() {
        let msg = explain_function_not_found("hold_only (&str)").unwrap();
        assert!(msg.contains("2 or 3"), "{msg}");
    }

    #[test]
    fn rhai_decoration_is_stripped() {
        assert_eq!(
            clean_rhai_message(
                "Syntax error: Expecting ';' to terminate this statement (line 3, position 1)"
            ),
            "Expecting ';' to terminate this statement"
        );
        assert_eq!(
            clean_rhai_message("Runtime error: Invalid 'from' key (line 2, position 1)"),
            "Invalid 'from' key"
        );
    }

    #[test]
    fn unknown_functions_are_left_alone() {
        assert!(explain_function_not_found("frobnicate (&str)").is_none());
    }
}
