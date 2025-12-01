//! Keycode definitions table.
//!
//! This module contains all keycode definitions in a single data table.
//! Format: Variant => "DisplayName", evdev_code, vk_code, ["ALIAS1", "ALIAS2", ...]
//!
//! Evdev codes are from linux/input-event-codes.h
//! VK codes are from Windows WinUser.h VK_* constants

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::macro_def::define_keycodes;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct KeyDefinition {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub evdev: u16,
    pub vk: u16,
}

define_keycodes! {
    // Letters A-Z (evdev: 30-38, 44-50, 16-25; VK: 0x41-0x5A)
    A => "A", 30, 0x41, ["A"],
    B => "B", 48, 0x42, ["B"],
    C => "C", 46, 0x43, ["C"],
    D => "D", 32, 0x44, ["D"],
    E => "E", 18, 0x45, ["E"],
    F => "F", 33, 0x46, ["F"],
    G => "G", 34, 0x47, ["G"],
    H => "H", 35, 0x48, ["H"],
    I => "I", 23, 0x49, ["I"],
    J => "J", 36, 0x4A, ["J"],
    K => "K", 37, 0x4B, ["K"],
    L => "L", 38, 0x4C, ["L"],
    M => "M", 50, 0x4D, ["M"],
    N => "N", 49, 0x4E, ["N"],
    O => "O", 24, 0x4F, ["O"],
    P => "P", 25, 0x50, ["P"],
    Q => "Q", 16, 0x51, ["Q"],
    R => "R", 19, 0x52, ["R"],
    S => "S", 31, 0x53, ["S"],
    T => "T", 20, 0x54, ["T"],
    U => "U", 22, 0x55, ["U"],
    V => "V", 47, 0x56, ["V"],
    W => "W", 17, 0x57, ["W"],
    X => "X", 45, 0x58, ["X"],
    Y => "Y", 21, 0x59, ["Y"],
    Z => "Z", 44, 0x5A, ["Z"],

    // Numbers 0-9 (evdev: 2-11; VK: 0x30-0x39)
    Key0 => "0", 11, 0x30, ["0", "KEY0"],
    Key1 => "1", 2, 0x31, ["1", "KEY1"],
    Key2 => "2", 3, 0x32, ["2", "KEY2"],
    Key3 => "3", 4, 0x33, ["3", "KEY3"],
    Key4 => "4", 5, 0x34, ["4", "KEY4"],
    Key5 => "5", 6, 0x35, ["5", "KEY5"],
    Key6 => "6", 7, 0x36, ["6", "KEY6"],
    Key7 => "7", 8, 0x37, ["7", "KEY7"],
    Key8 => "8", 9, 0x38, ["8", "KEY8"],
    Key9 => "9", 10, 0x39, ["9", "KEY9"],

    // Function keys F1-F12 (evdev: 59-68, 87-88; VK: 0x70-0x7B)
    F1 => "F1", 59, 0x70, ["F1"],
    F2 => "F2", 60, 0x71, ["F2"],
    F3 => "F3", 61, 0x72, ["F3"],
    F4 => "F4", 62, 0x73, ["F4"],
    F5 => "F5", 63, 0x74, ["F5"],
    F6 => "F6", 64, 0x75, ["F6"],
    F7 => "F7", 65, 0x76, ["F7"],
    F8 => "F8", 66, 0x77, ["F8"],
    F9 => "F9", 67, 0x78, ["F9"],
    F10 => "F10", 68, 0x79, ["F10"],
    F11 => "F11", 87, 0x7A, ["F11"],
    F12 => "F12", 88, 0x7B, ["F12"],

    // Modifier keys
    LeftShift => "LeftShift", 42, 0xA0, ["LEFTSHIFT", "LSHIFT", "SHIFT"],
    RightShift => "RightShift", 54, 0xA1, ["RIGHTSHIFT", "RSHIFT"],
    LeftCtrl => "LeftCtrl", 29, 0xA2, ["LEFTCTRL", "LCTRL", "CTRL", "CONTROL"],
    RightCtrl => "RightCtrl", 97, 0xA3, ["RIGHTCTRL", "RCTRL"],
    LeftAlt => "LeftAlt", 56, 0xA4, ["LEFTALT", "LALT", "ALT"],
    RightAlt => "RightAlt", 100, 0xA5, ["RIGHTALT", "RALT", "ALTGR"],
    LeftMeta => "LeftMeta", 125, 0x5B, ["LEFTMETA", "LMETA", "META", "WIN", "SUPER", "CMD"],
    RightMeta => "RightMeta", 126, 0x5C, ["RIGHTMETA", "RMETA", "RWIN", "RSUPER", "RCMD"],

    // Navigation keys
    Up => "Up", 103, 0x26, ["UP", "UPARROW"],
    Down => "Down", 108, 0x28, ["DOWN", "DOWNARROW"],
    Left => "Left", 105, 0x25, ["LEFT", "LEFTARROW"],
    Right => "Right", 106, 0x27, ["RIGHT", "RIGHTARROW"],
    Home => "Home", 102, 0x24, ["HOME"],
    End => "End", 107, 0x23, ["END"],
    PageUp => "PageUp", 104, 0x21, ["PAGEUP", "PGUP"],
    PageDown => "PageDown", 109, 0x22, ["PAGEDOWN", "PGDN"],

    // Editing keys
    Insert => "Insert", 110, 0x2D, ["INSERT", "INS"],
    Delete => "Delete", 111, 0x2E, ["DELETE", "DEL"],
    Backspace => "Backspace", 14, 0x08, ["BACKSPACE", "BACK", "BS"],

    // Whitespace keys
    Space => "Space", 57, 0x20, ["SPACE", "SPACEBAR"],
    Tab => "Tab", 15, 0x09, ["TAB"],
    Enter => "Enter", 28, 0x0D, ["ENTER", "RETURN"],

    // Lock keys
    CapsLock => "CapsLock", 58, 0x14, ["CAPSLOCK", "CAPS"],
    NumLock => "NumLock", 69, 0x90, ["NUMLOCK", "NUM"],
    ScrollLock => "ScrollLock", 70, 0x91, ["SCROLLLOCK", "SCROLL"],

    // Escape and Print Screen area
    Escape => "Escape", 1, 0x1B, ["ESCAPE", "ESC"],
    PrintScreen => "PrintScreen", 99, 0x2C, ["PRINTSCREEN", "PRINT", "PRTSC"],
    Pause => "Pause", 119, 0x13, ["PAUSE", "BREAK"],

    // Punctuation and symbols
    Grave => "Grave", 41, 0xC0, ["GRAVE", "BACKTICK", "TILDE"],
    Minus => "Minus", 12, 0xBD, ["MINUS", "DASH"],
    Equal => "Equal", 13, 0xBB, ["EQUAL", "EQUALS"],
    LeftBracket => "LeftBracket", 26, 0xDB, ["LEFTBRACKET", "LBRACKET"],
    RightBracket => "RightBracket", 27, 0xDD, ["RIGHTBRACKET", "RBRACKET"],
    Backslash => "Backslash", 43, 0xDC, ["BACKSLASH"],
    Semicolon => "Semicolon", 39, 0xBA, ["SEMICOLON"],
    Apostrophe => "Apostrophe", 40, 0xDE, ["APOSTROPHE", "QUOTE"],
    Comma => "Comma", 51, 0xBC, ["COMMA"],
    Period => "Period", 52, 0xBE, ["PERIOD", "DOT"],
    Slash => "Slash", 53, 0xBF, ["SLASH"],

    // Numpad keys
    Numpad0 => "Numpad0", 82, 0x60, ["NUMPAD0", "KP0"],
    Numpad1 => "Numpad1", 79, 0x61, ["NUMPAD1", "KP1"],
    Numpad2 => "Numpad2", 80, 0x62, ["NUMPAD2", "KP2"],
    Numpad3 => "Numpad3", 81, 0x63, ["NUMPAD3", "KP3"],
    Numpad4 => "Numpad4", 75, 0x64, ["NUMPAD4", "KP4"],
    Numpad5 => "Numpad5", 76, 0x65, ["NUMPAD5", "KP5"],
    Numpad6 => "Numpad6", 77, 0x66, ["NUMPAD6", "KP6"],
    Numpad7 => "Numpad7", 71, 0x67, ["NUMPAD7", "KP7"],
    Numpad8 => "Numpad8", 72, 0x68, ["NUMPAD8", "KP8"],
    Numpad9 => "Numpad9", 73, 0x69, ["NUMPAD9", "KP9"],
    NumpadAdd => "NumpadAdd", 78, 0x6B, ["NUMPADADD", "KPADD", "KPPLUS"],
    NumpadSubtract => "NumpadSubtract", 74, 0x6D, ["NUMPADSUBTRACT", "KPSUB", "KPMINUS"],
    NumpadMultiply => "NumpadMultiply", 55, 0x6A, ["NUMPADMULTIPLY", "KPMUL", "KPASTERISK"],
    NumpadDivide => "NumpadDivide", 98, 0x6F, ["NUMPADDIVIDE", "KPDIV", "KPSLASH"],
    NumpadEnter => "NumpadEnter", 96, 0x0D, ["NUMPADENTER", "KPENTER"],
    NumpadDecimal => "NumpadDecimal", 83, 0x6E, ["NUMPADDECIMAL", "KPDECIMAL", "KPDOT"],

    // Media keys
    VolumeUp => "VolumeUp", 115, 0xAF, ["VOLUMEUP", "VOLUP"],
    VolumeDown => "VolumeDown", 114, 0xAE, ["VOLUMEDOWN", "VOLDOWN"],
    VolumeMute => "VolumeMute", 113, 0xAD, ["VOLUMEMUTE", "MUTE"],
    MediaPlayPause => "MediaPlayPause", 164, 0xB3, ["MEDIAPLAYPAUSE", "PLAYPAUSE", "PLAY"],
    MediaStop => "MediaStop", 166, 0xB2, ["MEDIASTOP", "STOP"],
    MediaNext => "MediaNext", 163, 0xB0, ["MEDIANEXT", "NEXT", "NEXTTRACK"],
    MediaPrev => "MediaPrev", 165, 0xB1, ["MEDIAPREV", "PREV", "PREVTRACK"],
}
