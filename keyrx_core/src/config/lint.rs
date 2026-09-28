//! Static checks on a compiled configuration that are not errors but almost
//! always are mistakes.
//!
//! The only check today finds **dead mappings**: `KeyLookup` tries a key's
//! mappings in a fixed order and the first whose condition holds wins, so a
//! mapping is unreachable when an earlier-tried mapping of the same key holds
//! whenever it does - in practice the same key mapped twice in the same scope
//! (`map("A", ..)` twice, or the same `when` layer declared twice). The order
//! is read from `KeyLookup` itself so the lint cannot drift from the runtime.

use alloc::string::String;
use alloc::vec::Vec;

use crate::config::{Condition, ConditionItem, DeviceConfig, KeyCode};
use crate::runtime::KeyLookup;

/// A mapping that can never fire, and the earlier one that always wins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadMapping {
    /// Pattern of the `device_start` block.
    pub device_pattern: String,
    /// The input key both mappings are for.
    pub key: KeyCode,
    /// Condition of the dead mapping (`None` = unconditional).
    pub dead_condition: Option<Condition>,
    /// Condition of the mapping that shadows it.
    pub winning_condition: Option<Condition>,
}

/// One requirement of an AND-ed condition. `NotActive` is kept whole: two
/// negations only compare equal when they are identical.
#[derive(PartialEq)]
enum Requirement<'a> {
    Item(ConditionItem),
    Device(&'a str),
    Not(&'a [ConditionItem]),
}

fn requirements(condition: Option<&Condition>) -> Vec<Requirement<'_>> {
    let Some(condition) = condition else {
        return Vec::new();
    };
    match condition {
        Condition::ModifierActive(id) => {
            alloc::vec![Requirement::Item(ConditionItem::ModifierActive(*id))]
        }
        Condition::LockActive(id) => alloc::vec![Requirement::Item(ConditionItem::LockActive(*id))],
        Condition::AllActive(items) => items.iter().cloned().map(Requirement::Item).collect(),
        Condition::NotActive(items) => alloc::vec![Requirement::Not(items)],
        Condition::DeviceMatches(pattern) => alloc::vec![Requirement::Device(pattern)],
        Condition::ImeActive => alloc::vec![Requirement::Item(ConditionItem::ImeActive)],
        Condition::InputLanguage(lang) => {
            alloc::vec![Requirement::Item(ConditionItem::InputLanguage(
                lang.clone()
            ))]
        }
    }
}

/// `true` when condition `a` holds whenever `b` holds (`None` = always):
/// every AND-ed requirement of `a` is also one of `b`'s.
pub(crate) fn holds_whenever(a: Option<&Condition>, b: Option<&Condition>) -> bool {
    let b = requirements(b);
    requirements(a).iter().all(|req| b.contains(req))
}

/// Mappings of `config` that can never fire (see the module docs), sorted
/// by key for stable output.
pub fn dead_mappings(config: &DeviceConfig) -> Vec<DeadMapping> {
    let lookup = KeyLookup::from_device_config(config);
    let mut dead = Vec::new();
    for (key, order) in lookup.conditions_in_lookup_order() {
        for (j, condition) in order.iter().enumerate() {
            let earlier = order.get(..j).unwrap_or_default();
            if let Some(winning) = earlier.iter().find(|c| holds_whenever(**c, *condition)) {
                dead.push(DeadMapping {
                    device_pattern: config.identifier.pattern.clone(),
                    key,
                    dead_condition: condition.cloned(),
                    winning_condition: winning.cloned(),
                });
            }
        }
    }
    dead.sort_by_key(|d| d.key as u16);
    dead
}

impl core::fmt::Display for DeadMapping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let scope = |c: &Option<Condition>| match c {
            None => String::from("base layer"),
            Some(c) => alloc::format!("when {c:?}"),
        };
        write!(
            f,
            "device \"{}\": mapping of {:?} ({}) never fires - an earlier mapping of the same key ({}) always wins",
            self.device_pattern,
            self.key,
            scope(&self.dead_condition),
            scope(&self.winning_condition),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BaseKeyMapping, DeviceIdentifier, KeyMapping};
    use alloc::vec;

    fn block(mappings: Vec<KeyMapping>) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".into(),
            },
            mappings,
        }
    }

    fn when(condition: Condition, from: KeyCode, to: KeyCode) -> KeyMapping {
        KeyMapping::Conditional {
            condition,
            mappings: vec![BaseKeyMapping::Simple { from, to }],
        }
    }

    #[test]
    fn duplicate_base_mapping_is_dead() {
        let config = block(vec![
            KeyMapping::simple(KeyCode::A, KeyCode::B),
            KeyMapping::simple(KeyCode::A, KeyCode::C),
        ]);
        let dead = dead_mappings(&config);
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].key, KeyCode::A);
        assert_eq!(dead[0].dead_condition, None);
    }

    #[test]
    fn more_specific_layer_after_general_one_is_live() {
        let both = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::ModifierActive(1),
        ]);
        let config = block(vec![
            when(Condition::ModifierActive(0), KeyCode::A, KeyCode::B),
            when(both, KeyCode::A, KeyCode::C),
        ]);
        assert!(dead_mappings(&config).is_empty());
    }

    #[test]
    fn specific_before_general_and_base_after_layer_are_live() {
        let both = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::ModifierActive(1),
        ]);
        let config = block(vec![
            KeyMapping::simple(KeyCode::A, KeyCode::D),
            when(both, KeyCode::A, KeyCode::C),
            when(Condition::ModifierActive(0), KeyCode::A, KeyCode::B),
            when(Condition::LockActive(0), KeyCode::A, KeyCode::E),
        ]);
        assert!(dead_mappings(&config).is_empty());
    }

    #[test]
    fn same_layer_declared_twice_is_dead() {
        let config = block(vec![
            when(Condition::ModifierActive(3), KeyCode::H, KeyCode::Left),
            when(Condition::ModifierActive(3), KeyCode::H, KeyCode::Right),
        ]);
        assert_eq!(dead_mappings(&config).len(), 1);
    }

    #[test]
    fn different_negations_do_not_shadow() {
        let config = block(vec![
            when(
                Condition::NotActive(vec![ConditionItem::ModifierActive(0)]),
                KeyCode::A,
                KeyCode::B,
            ),
            when(
                Condition::NotActive(vec![ConditionItem::ModifierActive(1)]),
                KeyCode::A,
                KeyCode::C,
            ),
        ]);
        assert!(dead_mappings(&config).is_empty());
    }
}
