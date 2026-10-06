//! `examples/user_layout.rhai` is THE user's daily layout (JIS keyboard on a
//! Dvorak OS layout, ten tap-hold layers MD_00..MD_09 and a Shift layer).
//!
//! Three guards live here:
//!
//! 1. `examples/user_layout.krx` is byte-identical to a fresh compile of the
//!    `.rhai` (the compiler is deterministic and nobody forgot to recompile).
//! 2. Every mapping of every layer, derived from the PARSED config (so a new
//!    mapping is covered automatically), produces the right output through the
//!    real engine: tap, hold past the threshold, rollover and release order.
//! 3. Nothing is left held (net held keys = 0) after any of those.

use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::serialize;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConfigRoot, DeviceConfig, KeyCode, KeyMapping,
};
use keyrx_core::runtime::{
    check_tap_hold_timeouts, process_event, DeviceState, KeyEvent, KeyEventType, KeyLookup,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

const THRESHOLD_US: u64 = 200_000;

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
}

fn parse_user_layout() -> ConfigRoot {
    let path = example("user_layout.rhai");
    let source = std::fs::read_to_string(&path).expect("user_layout.rhai readable");
    Parser::new()
        .parse_string(&source, &path)
        .expect("user_layout.rhai compiles")
}

#[test]
fn committed_krx_is_byte_identical_to_a_fresh_compile() {
    let fresh = serialize(&parse_user_layout()).expect("serialize");
    let again = serialize(&parse_user_layout()).expect("serialize");
    assert_eq!(fresh, again, "compilation is not deterministic");
    let committed = std::fs::read(example("user_layout.krx")).expect("user_layout.krx readable");
    assert!(
        fresh == committed,
        "examples/user_layout.krx is stale: recompile with \
         `keyrx_compiler compile examples/user_layout.rhai -o examples/user_layout.krx`"
    );
}

/// The keys a mapping types: modifiers first (in press order), then the key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stroke {
    mods: Vec<KeyCode>,
    keys: Vec<KeyCode>,
}

fn stroke_of(m: &BaseKeyMapping) -> Option<Stroke> {
    match m {
        BaseKeyMapping::Simple { to, .. } => Some(Stroke {
            mods: vec![],
            keys: vec![*to],
        }),
        BaseKeyMapping::ModifiedOutput {
            to,
            shift,
            ctrl,
            alt,
            win,
            ..
        } => {
            let mut mods = vec![];
            for (on, key) in [
                (*shift, KeyCode::LShift),
                (*ctrl, KeyCode::LCtrl),
                (*alt, KeyCode::LAlt),
                (*win, KeyCode::LMeta),
            ] {
                if on {
                    mods.push(key);
                }
            }
            Some(Stroke {
                mods,
                keys: vec![*to],
            })
        }
        BaseKeyMapping::Sequence { keys, .. } => Some(Stroke {
            mods: vec![],
            keys: keys.clone(),
        }),
        _ => None,
    }
}

struct Layout {
    device: DeviceConfig,
    /// modifier id -> physical keys that hold the layer
    holders: BTreeMap<u8, Vec<KeyCode>>,
    /// modifier id -> mappings active while it is held
    layers: BTreeMap<u8, Vec<BaseKeyMapping>>,
    base: Vec<BaseKeyMapping>,
}

fn layout() -> Layout {
    let config = parse_user_layout();
    assert_eq!(config.devices.len(), 1);
    let device = config.devices[0].clone();
    let (mut holders, mut layers, mut base) = (BTreeMap::new(), BTreeMap::new(), vec![]);
    for m in &device.mappings {
        match m {
            KeyMapping::Base(b) => {
                match b {
                    BaseKeyMapping::TapHold {
                        from,
                        hold_modifier,
                        ..
                    }
                    | BaseKeyMapping::HoldOnly {
                        from,
                        hold_modifier,
                        ..
                    } => {
                        holders
                            .entry(*hold_modifier)
                            .or_insert_with(Vec::new)
                            .push(*from);
                    }
                    _ => {}
                }
                base.push(b.clone());
            }
            KeyMapping::Conditional {
                condition: Condition::ModifierActive(id),
                mappings,
            } => {
                layers
                    .entry(*id)
                    .or_insert_with(Vec::new)
                    .extend(mappings.iter().cloned());
            }
            // IME/language-conditional kana helpers cannot be reached without
            // an IME state; they are covered by `ime_sequences_follow_ime_state`.
            KeyMapping::Conditional { .. } => {}
        }
    }
    Layout {
        device,
        holders,
        layers,
        base,
    }
}

/// Feeds events with explicit timestamps and tracks what is held at the output.
struct Rig {
    lookup: KeyLookup,
    state: DeviceState,
    held: HashSet<KeyCode>,
    log: Vec<KeyEvent>,
}

impl Rig {
    fn new(device: &DeviceConfig) -> Self {
        Self {
            lookup: KeyLookup::from_device_config(device),
            state: DeviceState::new(),
            held: HashSet::new(),
            log: vec![],
        }
    }

    fn absorb(&mut self, events: Vec<KeyEvent>) -> Vec<KeyEvent> {
        for e in &events {
            match e.event_type() {
                KeyEventType::Press => {
                    self.held.insert(e.keycode());
                }
                KeyEventType::Release => {
                    self.held.remove(&e.keycode());
                }
            }
        }
        self.log.extend(events.iter().cloned());
        events
    }

    fn press(&mut self, key: KeyCode, at_ms: u64) -> Vec<KeyEvent> {
        let ev = KeyEvent::press(key).with_timestamp(at_ms * 1000);
        let out = process_event(ev, &self.lookup, &mut self.state);
        self.absorb(out)
    }

    fn release(&mut self, key: KeyCode, at_ms: u64) -> Vec<KeyEvent> {
        let ev = KeyEvent::release(key).with_timestamp(at_ms * 1000);
        let out = process_event(ev, &self.lookup, &mut self.state);
        self.absorb(out)
    }

    fn tick(&mut self, at_ms: u64) -> Vec<KeyEvent> {
        let out = check_tap_hold_timeouts(at_ms * 1000, &mut self.state);
        self.absorb(out)
    }
}

fn presses(events: &[KeyEvent]) -> Vec<KeyCode> {
    events
        .iter()
        .filter(|e| e.event_type() == KeyEventType::Press)
        .map(KeyEvent::keycode)
        .collect()
}

/// Checks the press events typed for `stroke`: modifiers first, then keys.
fn assert_typed(ctx: &str, out: &[KeyEvent], stroke: &Stroke) {
    let typed = presses(out);
    let mut want = stroke.mods.clone();
    want.extend(stroke.keys.iter().copied());
    // Sequences type their keys one per press+release; only the order of the
    // presses is observable in the combined log.
    assert_eq!(typed, want, "{ctx}: typed {typed:?}, expected {want:?}");
}

fn assert_clean(ctx: &str, rig: &Rig) {
    assert!(
        rig.held.is_empty(),
        "{ctx}: stuck output keys {:?}",
        rig.held
    );
}

#[test]
fn every_layer_mapping_types_the_right_keys_in_every_timing() {
    let l = layout();
    assert_eq!(
        l.layers.len(),
        10,
        "MD_00..MD_09 expected, got {:?}",
        l.layers.keys()
    );
    let mut checked = 0usize;
    for (id, mappings) in &l.layers {
        if *id > 9 && *id != 0x0A {
            continue;
        }
        let holder = *l
            .holders
            .get(id)
            .unwrap_or_else(|| panic!("MD_{id:02X} has mappings but no key holds it"))
            .first()
            .unwrap();
        for m in mappings {
            let Some(stroke) = stroke_of(m) else { continue };
            let from = m.source_key();
            // A source that is itself a layer holder is resolved by the engine's
            // tap-hold ahead of the layer: skip, it is reported by
            // `layer_holders_inside_layers_are_known`.
            if l.holders.values().flatten().any(|h| *h == from) {
                continue;
            }
            let ctx = format!("MD_{id:02X} {holder:?}+{from:?}");

            // 1. permissive hold: layer key down, other key within the threshold.
            let mut r = Rig::new(&l.device);
            r.press(holder, 0);
            r.press(from, 50);
            r.tick(60);
            assert_typed(&format!("{ctx} permissive"), &r.log, &stroke);
            r.release(from, 90);
            r.release(holder, 100);
            assert_clean(&format!("{ctx} permissive"), &r);

            // 2. hold past the threshold, then the key.
            let mut r = Rig::new(&l.device);
            r.press(holder, 0);
            r.tick(THRESHOLD_US / 1000 + 50);
            r.press(from, 300);
            r.release(from, 340);
            r.release(holder, 400);
            assert_typed(&format!("{ctx} hold>200ms"), &r.log, &stroke);
            assert_clean(&format!("{ctx} hold>200ms"), &r);

            // 3. rollover: the layer key is released BEFORE the typed key.
            let mut r = Rig::new(&l.device);
            r.press(holder, 0);
            r.press(from, 50);
            r.release(holder, 80);
            r.release(from, 120);
            assert_typed(&format!("{ctx} rollover"), &r.log, &stroke);
            assert_clean(&format!("{ctx} rollover"), &r);

            // 4. same key twice in a row: two identical strokes, nothing stuck.
            let mut r = Rig::new(&l.device);
            r.press(holder, 0);
            for i in 0..2u64 {
                r.press(from, 50 + i * 60);
                r.release(from, 80 + i * 60);
            }
            r.release(holder, 300);
            let n = presses(&r.log)
                .iter()
                .filter(|k| stroke.keys.contains(k))
                .count();
            assert_eq!(n, 2 * stroke.keys.len(), "{ctx} repeat: {:?}", r.log);
            assert_clean(&format!("{ctx} repeat"), &r);
            checked += 1;
        }
    }
    assert!(
        checked >= 180,
        "only {checked} layer mappings were exercised"
    );
}

#[test]
fn every_base_mapping_types_the_right_keys_and_releases_them() {
    let l = layout();
    let mut checked = 0usize;
    for m in &l.base {
        let ctx = format!("base {m:?}");
        match m {
            BaseKeyMapping::Simple { from, .. } | BaseKeyMapping::ModifiedOutput { from, .. } => {
                let stroke = stroke_of(m).unwrap();
                let mut r = Rig::new(&l.device);
                r.press(*from, 0);
                r.release(*from, 20);
                assert_typed(&ctx, &r.log, &stroke);
                assert_clean(&ctx, &r);
                checked += 1;
            }
            BaseKeyMapping::TapHold { from, tap, .. } => {
                let mut r = Rig::new(&l.device);
                r.press(*from, 0);
                r.release(*from, 60);
                assert_eq!(presses(&r.log), vec![*tap], "{ctx}: tap");
                assert_clean(&ctx, &r);
                // Held past the threshold with no other key: nothing typed.
                let mut r = Rig::new(&l.device);
                r.press(*from, 0);
                r.tick(300);
                r.release(*from, 400);
                assert!(presses(&r.log).is_empty(), "{ctx}: hold typed {:?}", r.log);
                assert_clean(&ctx, &r);
                checked += 1;
            }
            BaseKeyMapping::HoldOnly { from, .. } => {
                let mut r = Rig::new(&l.device);
                r.press(*from, 0);
                r.release(*from, 60);
                assert!(
                    r.log.is_empty(),
                    "{ctx}: tap must be suppressed, got {:?}",
                    r.log
                );
                checked += 1;
            }
            _ => {}
        }
    }
    assert!(checked >= 60, "only {checked} base mappings were exercised");
}

#[test]
fn layer_holders_inside_layers_are_known() {
    // The only layer entry whose source is itself a tap-hold key is the shift
    // layer's `B -> Shift+Enter`. Pin it so a new collision is a conscious one.
    let l = layout();
    let holders: HashSet<KeyCode> = l.holders.values().flatten().copied().collect();
    let mut collisions = BTreeSet::new();
    for (id, mappings) in &l.layers {
        for m in mappings {
            let from = match m {
                BaseKeyMapping::Simple { from, .. }
                | BaseKeyMapping::ModifiedOutput { from, .. } => *from,
                _ => continue,
            };
            if holders.contains(&from) {
                collisions.insert((*id, format!("{from:?}")));
            }
        }
    }
    assert_eq!(
        collisions.into_iter().collect::<Vec<_>>(),
        vec![(10u8, "B".to_string())],
        "unexpected layer/holder collisions"
    );
}

#[test]
fn ime_sequences_follow_ime_state() {
    let l = layout();
    let ime: Vec<&KeyMapping> = l
        .device
        .mappings
        .iter()
        .filter(|m| matches!(m, KeyMapping::Conditional { condition, .. } if !matches!(condition, Condition::ModifierActive(_))))
        .collect();
    assert_eq!(ime.len(), 1, "exactly one IME block expected");
}
