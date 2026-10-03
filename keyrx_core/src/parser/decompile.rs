//! Turn a compiled [`ConfigRoot`] back into Rhai source.
//!
//! A `.krx` carries no source, so loading one into the editor means writing
//! the DSL calls that compile back to the same mappings. This module is the
//! inverse of [`super::builders`]: every `BaseKeyMapping` has one spelling.
//!
//! What is NOT recovered: comments, blank lines and the original ordering
//! style (a `.krx` never stored them), and the compile-time [`Metadata`]
//! (timestamp, source hash). Mappings are exact; a mapping the DSL cannot
//! spell yields [`DecompileError`] instead of an approximation, so the caller
//! (which re-parses the output and compares) never imports a different layout
//! than the file contained.

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{self, Write};

use crate::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, KeyCode, KeyMapping,
};

/// A compiled mapping that has no Rhai spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompileError(pub String);

impl fmt::Display for DecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Rhai source that compiles to `config.devices` (see the module docs).
///
/// # Errors
///
/// [`DecompileError`] when a mapping or condition cannot be written in the DSL.
pub fn decompile(config: &ConfigRoot) -> Result<String, DecompileError> {
    let mut out = String::new();
    out.push_str(
        "// Decompiled from a compiled .krx layout.\n\
         // A .krx stores no source, so comments and original formatting are not\n\
         // available; the key mappings are exact.\n\n",
    );
    for device in &config.devices {
        write_device(&mut out, device)?;
    }
    Ok(out)
}

fn write_device(out: &mut String, device: &DeviceConfig) -> Result<(), DecompileError> {
    let _ = writeln!(out, "device_start({});", quote(&device.identifier.pattern));
    for mapping in &device.mappings {
        match mapping {
            KeyMapping::Base(base) => write_base(out, "  ", base)?,
            KeyMapping::Conditional {
                condition,
                mappings,
            } => {
                let (open, close) = block_calls(condition)?;
                let _ = writeln!(out, "  {open};");
                for base in mappings {
                    write_base(out, "    ", base)?;
                }
                let _ = writeln!(out, "  {close}();");
            }
        }
    }
    out.push_str("device_end();\n\n");
    Ok(())
}

fn quote(s: &str) -> String {
    let mut q = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => q.push_str("\\\""),
            '\\' => q.push_str("\\\\"),
            '\n' => q.push_str("\\n"),
            '\r' => q.push_str("\\r"),
            '\t' => q.push_str("\\t"),
            c => q.push(c),
        }
    }
    q.push('"');
    q
}

/// Physical (input) spelling: the bare variant name.
fn input(key: KeyCode) -> String {
    quote(&format!("{key:?}"))
}

/// Virtual (output) spelling: `VK_` + variant name.
fn output(key: KeyCode) -> String {
    quote(&format!("VK_{key:?}"))
}

fn md(id: u8) -> String {
    format!("MD_{id:02X}")
}

fn lk(id: u8) -> String {
    format!("LK_{id:02X}")
}

fn item_name(item: &ConditionItem) -> String {
    match item {
        ConditionItem::ModifierActive(id) => md(*id),
        ConditionItem::LockActive(id) => lk(*id),
        ConditionItem::ImeActive => "IME".to_string(),
        ConditionItem::InputLanguage(lang) => format!("LANG_{lang}"),
    }
}

/// The `(open call, close function)` pair that writes `condition`.
fn block_calls(condition: &Condition) -> Result<(String, &'static str), DecompileError> {
    Ok(match condition {
        Condition::ModifierActive(id) => (format!("when_start({})", quote(&md(*id))), "when_end"),
        Condition::LockActive(id) => (format!("when_start({})", quote(&lk(*id))), "when_end"),
        Condition::ImeActive => ("when_start(\"IME\")".to_string(), "when_end"),
        Condition::InputLanguage(lang) => (
            format!("when_start({})", quote(&format!("LANG_{lang}"))),
            "when_end",
        ),
        Condition::AllActive(items) => {
            let names: alloc::vec::Vec<String> =
                items.iter().map(|i| quote(&item_name(i))).collect();
            (format!("when_start([{}])", names.join(", ")), "when_end")
        }
        Condition::NotActive(items) => match items.as_slice() {
            [item] => (
                format!("when_not_start({})", quote(&item_name(item))),
                "when_not_end",
            ),
            _ => {
                return Err(DecompileError(format!(
                    "a negated condition over {} items has no Rhai spelling (when_not takes one)",
                    items.len()
                )))
            }
        },
        Condition::DeviceMatches(pattern) => (
            format!("when_device_start({})", quote(pattern)),
            "when_device_end",
        ),
    })
}

fn write_base(out: &mut String, indent: &str, m: &BaseKeyMapping) -> Result<(), DecompileError> {
    let line = match m {
        BaseKeyMapping::Simple { from, to } => format!("map({}, {})", input(*from), output(*to)),
        BaseKeyMapping::Modifier { from, modifier_id } => {
            format!("map({}, {})", input(*from), quote(&md(*modifier_id)))
        }
        BaseKeyMapping::Lock { from, lock_id } => {
            format!("map({}, {})", input(*from), quote(&lk(*lock_id)))
        }
        BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } => format!(
            "tap_hold({}, {}, {}, {threshold_ms})",
            input(*from),
            output(*tap),
            quote(&md(*hold_modifier))
        ),
        BaseKeyMapping::HoldOnly {
            from,
            hold_modifier,
            threshold_ms,
        } => format!(
            "hold_only({}, {}, {threshold_ms})",
            input(*from),
            quote(&md(*hold_modifier))
        ),
        BaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        } => format!(
            "map({}, with_mods({}, {shift}, {ctrl}, {alt}, {win}))",
            input(*from),
            output(*to)
        ),
        BaseKeyMapping::Sequence { from, keys } => {
            let keys: alloc::vec::Vec<String> = keys.iter().map(|k| output(*k)).collect();
            format!("sequence({}, [{}])", input(*from), keys.join(", "))
        }
        BaseKeyMapping::TapHoldKey {
            from,
            tap: Some(tap),
            hold,
            threshold_ms,
        } => format!(
            "tap_hold({}, {}, {}, {threshold_ms})",
            input(*from),
            output(*tap),
            output(*hold)
        ),
        BaseKeyMapping::TapHoldKey {
            from,
            tap: None,
            hold,
            threshold_ms,
        } => format!(
            "hold_only({}, {}, {threshold_ms})",
            input(*from),
            output(*hold)
        ),
        BaseKeyMapping::OneShot {
            from,
            modifier,
            timeout_ms,
        } => format!(
            "one_shot({}, {}, {timeout_ms})",
            input(*from),
            output(*modifier)
        ),
        BaseKeyMapping::TapHoldKeyTimeoutOnly {
            from,
            tap: Some(tap),
            hold,
            threshold_ms,
        } => format!(
            "tap_hold_timeout_only({}, {}, {}, {threshold_ms})",
            input(*from),
            output(*tap),
            output(*hold)
        ),
        BaseKeyMapping::TapHoldKeyTimeoutOnly {
            from, tap: None, ..
        } => {
            return Err(DecompileError(format!(
                "{from:?}: a timeout-only hold without a tap key has no Rhai spelling"
            )))
        }
    };
    let _ = writeln!(out, "{indent}{line};");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DeviceIdentifier, Metadata, Version};
    use alloc::vec;

    fn root(mappings: alloc::vec::Vec<KeyMapping>) -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: "a \"quoted\" \\ pattern".to_string(),
                },
                mappings,
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: String::new(),
                source_hash: String::new(),
            },
        }
    }

    #[test]
    fn writes_every_base_mapping_and_escapes_the_pattern() {
        let src = decompile(&root(vec![
            KeyMapping::simple(KeyCode::A, KeyCode::B),
            KeyMapping::modifier(KeyCode::CapsLock, 1),
            KeyMapping::lock(KeyCode::ScrollLock, 0xFE),
            KeyMapping::modified_output(KeyCode::Num2, KeyCode::Num3, true, false, true, false),
            KeyMapping::sequence(KeyCode::F1, vec![KeyCode::H, KeyCode::I]),
        ]))
        .unwrap();
        assert!(src.contains(r#"device_start("a \"quoted\" \\ pattern");"#));
        assert!(src.contains(r#"map("A", "VK_B");"#));
        assert!(src.contains(r#"map("CapsLock", "MD_01");"#));
        assert!(src.contains(r#"map("ScrollLock", "LK_FE");"#));
        assert!(src.contains(r#"map("Num2", with_mods("VK_Num3", true, false, true, false));"#));
        assert!(src.contains(r#"sequence("F1", ["VK_H", "VK_I"]);"#));
    }

    #[test]
    fn writes_conditional_blocks() {
        let src = decompile(&root(vec![
            KeyMapping::conditional(
                Condition::NotActive(vec![ConditionItem::LockActive(2)]),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::B,
                }],
            ),
            KeyMapping::conditional(Condition::DeviceMatches("*Logi*".to_string()), vec![]),
        ]))
        .unwrap();
        assert!(src.contains("when_not_start(\"LK_02\");"));
        assert!(src.contains("when_not_end();"));
        assert!(src.contains("when_device_start(\"*Logi*\");"));
    }

    #[test]
    fn refuses_what_the_dsl_cannot_spell() {
        let not_two = KeyMapping::conditional(
            Condition::NotActive(vec![
                ConditionItem::ModifierActive(1),
                ConditionItem::LockActive(2),
            ]),
            vec![],
        );
        assert!(decompile(&root(vec![not_two])).is_err());
        let timeout_only = KeyMapping::Base(BaseKeyMapping::TapHoldKeyTimeoutOnly {
            from: KeyCode::F,
            tap: None,
            hold: KeyCode::LCtrl,
            threshold_ms: 200,
        });
        assert!(decompile(&root(vec![timeout_only])).is_err());
    }
}
