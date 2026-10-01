//! Scope bookkeeping shared by both DSL parsers (keyrx_core's WASM parser and
//! keyrx_compiler's std parser): where each source key was first mapped, and
//! where each `when_start`-style block was opened.
//!
//! Two lints live here so they cannot drift between the parsers:
//! - a source key mapped twice in the SAME scope (the device's top level, or
//!   one conditional block) is an error - the second mapping would be dead;
//! - a block opened and never closed is reported at the line it was opened.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::config::KeyCode;

/// `Some(line)` when the host engine could tell us the source line.
pub type Line = Option<usize>;

#[derive(Debug, Clone, Default)]
struct Block {
    opened_at: Line,
    sources: HashMap<KeyCode, Line>,
}

/// Tracks the scopes of one parse. See the module docs.
#[derive(Debug, Clone, Default)]
pub struct MappingScopes {
    device: HashMap<KeyCode, Line>,
    blocks: Vec<Block>,
}

fn at(line: Line) -> String {
    line.map_or(String::new(), |l| format!(" (line {})", l))
}

impl MappingScopes {
    /// A new `device_start`: the device's top-level scope is empty again.
    pub fn start_device(&mut self) {
        self.device.clear();
    }

    /// A conditional block was opened at `line`.
    pub fn open_block(&mut self, line: Line) {
        self.blocks.push(Block {
            opened_at: line,
            sources: HashMap::new(),
        });
    }

    /// The innermost conditional block was closed.
    pub fn close_block(&mut self) {
        self.blocks.pop();
    }

    /// The outermost block that was never closed, if any: `Some(line)` where
    /// `line` is where it was opened (when known).
    pub fn unclosed_block(&self) -> Option<Line> {
        self.blocks.first().map(|b| b.opened_at)
    }

    /// An error naming the line of the outermost block still open, for
    /// `device_end()` / end of script.
    pub fn check_all_closed(&self) -> Result<(), String> {
        match self.unclosed_block() {
            None => Ok(()),
            Some(line) => Err(format!(
                "when_start()/when_not_start()/when_device_start() block opened{} is never \
                 closed: add the matching when_end()/when_not_end()/when_device_end()",
                at(line)
            )),
        }
    }

    /// Records that `key` is mapped at `line` in the current scope; an error
    /// naming both lines if the scope already maps it.
    pub fn record(&mut self, key: KeyCode, line: Line) -> Result<(), String> {
        let scope = match self.blocks.last_mut() {
            Some(block) => &mut block.sources,
            None => &mut self.device,
        };
        if let Some(first) = scope.get(&key) {
            return Err(format!(
                "Duplicate mapping for key {:?}: it is already mapped{} in this scope, so \
                 this mapping{} would never run. Remove one of them.",
                key,
                at(*first),
                at(line),
            ));
        }
        scope.insert(key, line);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_in_same_scope_is_an_error_naming_both_lines() {
        let mut s = MappingScopes::default();
        s.record(KeyCode::A, Some(3)).unwrap();
        let err = s.record(KeyCode::A, Some(9)).unwrap_err();
        assert!(err.contains("line 3") && err.contains("line 9"), "{err}");
    }

    #[test]
    fn same_key_in_different_scopes_is_fine() {
        let mut s = MappingScopes::default();
        s.record(KeyCode::A, Some(1)).unwrap();
        s.open_block(Some(2));
        s.record(KeyCode::A, Some(3)).unwrap();
        s.close_block();
        s.open_block(Some(5));
        s.record(KeyCode::A, Some(6)).unwrap();
        s.close_block();
        s.start_device();
        s.record(KeyCode::A, Some(8)).unwrap();
    }

    #[test]
    fn unclosed_block_reports_where_it_opened() {
        let mut s = MappingScopes::default();
        assert_eq!(s.unclosed_block(), None);
        s.open_block(Some(4));
        assert_eq!(s.unclosed_block(), Some(Some(4)));
        s.close_block();
        assert_eq!(s.unclosed_block(), None);
    }
}
