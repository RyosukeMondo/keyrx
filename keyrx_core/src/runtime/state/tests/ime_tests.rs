use super::*;
use alloc::string::String;

use crate::config::ImeState;

#[test]
fn test_ime_state_default_inactive() {
    let state = DeviceState::new();
    assert!(!state.is_ime_active());
    assert_eq!(state.input_language(), "");
}

#[test]
fn test_set_ime_state() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });
    assert!(state.is_ime_active());
    assert_eq!(state.input_language(), "ja");
}

#[test]
fn test_evaluate_condition_ime_active() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });
    assert!(state.evaluate_condition(&Condition::ImeActive));
}

#[test]
fn test_evaluate_condition_ime_inactive() {
    let state = DeviceState::new();
    assert!(!state.evaluate_condition(&Condition::ImeActive));
}

#[test]
fn test_evaluate_condition_input_language_exact() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });
    assert!(state.evaluate_condition(&Condition::InputLanguage(String::from("ja"))));
    assert!(!state.evaluate_condition(&Condition::InputLanguage(String::from("ko"))));
}

#[test]
fn test_evaluate_condition_input_language_prefix() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja-JP"),
    });
    assert!(state.evaluate_condition(&Condition::InputLanguage(String::from("ja"))));
    assert!(!state.evaluate_condition(&Condition::InputLanguage(String::from("ko"))));
}

#[test]
fn test_evaluate_condition_input_language_case_insensitive() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });
    assert!(state.evaluate_condition(&Condition::InputLanguage(String::from("JA"))));
    assert!(state.evaluate_condition(&Condition::InputLanguage(String::from("Ja"))));
}

#[test]
fn test_all_active_with_ime() {
    let mut state = DeviceState::new();
    state.set_modifier(0);
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });

    let cond = Condition::AllActive(vec![
        ConditionItem::ModifierActive(0),
        ConditionItem::ImeActive,
        ConditionItem::InputLanguage(String::from("ja")),
    ]);
    assert!(state.evaluate_condition(&cond));
}

#[test]
fn test_all_active_with_ime_fails_when_off() {
    let mut state = DeviceState::new();
    state.set_modifier(0);
    // IME is off (default)

    let cond = Condition::AllActive(vec![
        ConditionItem::ModifierActive(0),
        ConditionItem::ImeActive,
    ]);
    assert!(!state.evaluate_condition(&cond));
}

#[test]
fn test_not_active_with_ime() {
    let state = DeviceState::new(); // IME inactive
    let cond = Condition::NotActive(vec![ConditionItem::ImeActive]);
    assert!(state.evaluate_condition(&cond));
}

#[test]
fn test_not_active_ime_when_on() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::from("ja"),
    });
    let cond = Condition::NotActive(vec![ConditionItem::ImeActive]);
    assert!(!state.evaluate_condition(&cond));
}

#[test]
fn test_language_empty_never_matches() {
    let mut state = DeviceState::new();
    state.set_ime_state(ImeState {
        active: true,
        language: String::new(),
    });
    assert!(!state.evaluate_condition(&Condition::InputLanguage(String::from("ja"))));
}
