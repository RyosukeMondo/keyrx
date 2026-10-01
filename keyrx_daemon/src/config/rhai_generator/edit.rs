//! Line-level helpers for [`super::RhaiGenerator`]: building a mapping line,
//! finding the lines that map a key, and choosing where a new one goes.

use super::keys::{canonical_output, same_key};
use super::scan::{first_quoted, Layout, Scope};
use super::{GeneratorError, KeyAction, MacroStep};

/// Calls that are structure, not mappings.
const STRUCTURE: [&str; 6] = [
    "device_start",
    "device_end",
    "when_start",
    "when_end",
    "when_device",
    "import",
];

/// The statement (without indentation) that maps `key` (already canonical).
pub fn mapping_body(key: &str, action: &KeyAction) -> Result<String, GeneratorError> {
    match action {
        KeyAction::SimpleRemap { output } => Ok(format!(
            "map(\"{key}\", \"{}\");",
            canonical_output(output)?
        )),
        KeyAction::TapHold {
            tap,
            hold,
            threshold_ms,
        } => Ok(format!(
            "tap_hold(\"{key}\", \"{}\", \"{}\", {threshold_ms});",
            canonical_output(tap)?,
            canonical_output(hold)?
        )),
        KeyAction::Macro { sequence } => {
            let steps = sequence
                .iter()
                .map(macro_step)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("macro(\"{key}\", [{}]);", steps.join(", ")))
        }
        KeyAction::Conditional { .. } => Err(GeneratorError::SyntaxError(
            "Conditional actions should use when blocks, not direct mappings".to_string(),
        )),
    }
}

fn macro_step(step: &MacroStep) -> Result<String, GeneratorError> {
    Ok(match step {
        MacroStep::Press(k) => format!("press(\"{}\")", canonical_output(k)?),
        MacroStep::Release(k) => format!("release(\"{}\")", canonical_output(k)?),
        MacroStep::Wait(ms) => format!("wait({ms})"),
    })
}

/// `(call name, first string argument)` of a mapping-like statement.
fn call_and_key(line: &str) -> Option<(&str, String)> {
    let line = line.trim();
    let open = line.find('(')?;
    let name = &line[..open];
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    if STRUCTURE.contains(&name) || !line[open + 1..].trim_start().starts_with('"') {
        return None;
    }
    Some((name, first_quoted(&line[open..])?))
}

/// Indexes of the lines in `scope` that map `key`, in order.
pub fn matching_lines(lines: &[String], scope: &Scope, layout: &Layout, key: &str) -> Vec<usize> {
    scope
        .lines()
        .filter(|&i| !layout.in_other_scope(scope, i))
        .filter(|&i| call_and_key(&lines[i]).is_some_and(|(_, k)| same_key(&k, key)))
        .collect()
}

/// Number of statements in a block body (blank lines and comments excluded).
pub fn count_mappings(body: &[String]) -> usize {
    body.iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .count()
}

pub fn indent_of(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

/// The indentation siblings in `scope` use (the last mapping's), else two spaces.
pub fn scope_indent(lines: &[String], scope: &Scope) -> String {
    (scope.range.start..scope.insert_limit)
        .rev()
        .find(|&i| call_and_key(&lines[i]).is_some())
        .map_or_else(|| "  ".to_string(), |i| indent_of(&lines[i]).to_string())
}

/// Where a new mapping line goes: right after the scope's last non-blank line.
pub fn insertion_point(lines: &[String], scope: &Scope) -> usize {
    let first = scope.range.start;
    (first..scope.insert_limit)
        .rev()
        .find(|&i| !lines[i].trim().is_empty())
        .map_or(first, |i| i + 1)
}

/// Removes the lines at `indexes` (any order).
pub fn remove_lines(lines: &mut Vec<String>, indexes: &[usize]) {
    let mut sorted = indexes.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    for i in sorted {
        lines.remove(i);
    }
}
