//! Device pattern matching: THE glob rule for `device_start(pattern)` block
//! selection (daemon) and `when_device(pattern)` conditions (runtime).
//!
//! `*` is a wildcard (`*`, `prefix*`, `*suffix`, `*middle*`, `a*b*c`); matching
//! is ASCII case-insensitive. A device is known by several identities (id,
//! name, path, serial); a pattern matches the device if it matches any of them.

/// Does `pattern` match any of the device's `identities`?
pub fn matches_any(identities: &[&str], pattern: &str) -> bool {
    identities.iter().any(|id| matches(id, pattern))
}

/// Does `pattern` match the single identity `id`?
pub fn matches(id: &str, pattern: &str) -> bool {
    // Device names differ in case across platforms ("USB Keyboard" vs
    // "usb keyboard"); match like the daemon's device selection does.
    let id = id.to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    let (id, pattern) = (id.as_str(), pattern.as_str());

    // Handle glob patterns with *
    if pattern.contains('*') {
        let parts: alloc::vec::Vec<&str> = pattern.split('*').collect();
        match parts.len() {
            1 => {
                // No actual * (shouldn't happen but handle it)
                id == pattern
            }
            2 => {
                // Single * - either prefix, suffix, or empty on one side
                let (prefix, suffix) = (parts[0], parts[1]);
                if prefix.is_empty() && suffix.is_empty() {
                    // Pattern is just "*" - matches everything
                    true
                } else if prefix.is_empty() {
                    // *suffix
                    id.ends_with(suffix)
                } else if suffix.is_empty() {
                    // prefix*
                    id.starts_with(prefix)
                } else {
                    // prefix*suffix
                    id.starts_with(prefix) && id.ends_with(suffix)
                }
            }
            3 => {
                // Two *s - typically *contains*
                let (prefix, middle, suffix) = (parts[0], parts[1], parts[2]);
                if prefix.is_empty() && suffix.is_empty() {
                    // *middle*
                    id.contains(middle)
                } else {
                    // More complex pattern - do simple check
                    id.starts_with(prefix) && id.ends_with(suffix) && id.contains(middle)
                }
            }
            _ => {
                // Complex pattern with multiple * - just check if all parts exist in order
                // This is a simplified implementation
                let mut remaining = id;
                for (i, part) in parts.iter().enumerate() {
                    if part.is_empty() {
                        continue;
                    }
                    if i == 0 {
                        // First part must be prefix
                        if !remaining.starts_with(part) {
                            return false;
                        }
                        remaining = &remaining[part.len()..];
                    } else if i == parts.len() - 1 {
                        // Last part must be suffix
                        if !remaining.ends_with(part) {
                            return false;
                        }
                    } else {
                        // Middle parts must exist somewhere
                        if let Some(pos) = remaining.find(part) {
                            remaining = &remaining[pos + part.len()..];
                        } else {
                            return false;
                        }
                    }
                }
                true
            }
        }
    } else {
        // Exact match
        id == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs() {
        assert!(matches("anything", "*"));
        assert!(matches("USB Keyboard", "usb*"));
        assert!(matches("USB Keyboard", "*KEYBOARD"));
        assert!(matches("my numpad 2", "*NumPad*"));
        assert!(matches("usb-num-kbd", "usb*num*kbd"));
        assert!(matches("Exact", "exact"));
        assert!(!matches("USB Keyboard", "*mouse*"));
        assert!(!matches("abc", "abcd"));
    }

    #[test]
    fn any_identity() {
        assert!(matches_any(
            &["path-/dev/input/event7", "USB NumPad"],
            "*numpad*"
        ));
        assert!(!matches_any(&[], "*"));
    }
}
