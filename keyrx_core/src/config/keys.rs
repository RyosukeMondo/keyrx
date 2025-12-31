// CheckBytes is used by #[archive(check_bytes)] derive macro
#[allow(unused_imports)]
use rkyv::{Archive, CheckBytes, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Platform-agnostic keyboard key codes
///
/// All variants have explicit discriminants to prevent reordering issues.
/// Keys are organized by category for maintainability.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Debug,
)]
#[archive(check_bytes)]
#[repr(u16)]
pub enum KeyCode {
    // Letters: A-Z (0x00-0x19)
    A = 0x00,
    B = 0x01,
    C = 0x02,
    D = 0x03,
    E = 0x04,
    F = 0x05,
    G = 0x06,
    H = 0x07,
    I = 0x08,
    J = 0x09,
    K = 0x0A,
    L = 0x0B,
    M = 0x0C,
    N = 0x0D,
    O = 0x0E,
    P = 0x0F,
    Q = 0x10,
    R = 0x11,
    S = 0x12,
    T = 0x13,
    U = 0x14,
    V = 0x15,
    W = 0x16,
    X = 0x17,
    Y = 0x18,
    Z = 0x19,

    // Numbers: 0-9 (0x20-0x29)
    Num0 = 0x20,
    Num1 = 0x21,
    Num2 = 0x22,
    Num3 = 0x23,
    Num4 = 0x24,
    Num5 = 0x25,
    Num6 = 0x26,
    Num7 = 0x27,
    Num8 = 0x28,
    Num9 = 0x29,

    // Function keys: F1-F12 (0x30-0x3B)
    F1 = 0x30,
    F2 = 0x31,
    F3 = 0x32,
    F4 = 0x33,
    F5 = 0x34,
    F6 = 0x35,
    F7 = 0x36,
    F8 = 0x37,
    F9 = 0x38,
    F10 = 0x39,
    F11 = 0x3A,
    F12 = 0x3B,

    // Physical modifier keys (0x100-0x107)
    LShift = 0x100,
    RShift = 0x101,
    LCtrl = 0x102,
    RCtrl = 0x103,
    LAlt = 0x104,
    RAlt = 0x105,
    LMeta = 0x106,
    RMeta = 0x107,

    // Special keys (0x200+)
    Escape = 0x200,
    Enter = 0x201,
    Backspace = 0x202,
    Tab = 0x203,
    Space = 0x204,
    CapsLock = 0x205,
    NumLock = 0x206,
    ScrollLock = 0x207,
    PrintScreen = 0x208,
    Pause = 0x209,
    Insert = 0x20A,
    Delete = 0x20B,
    Home = 0x20C,
    End = 0x20D,
    PageUp = 0x20E,
    PageDown = 0x20F,

    // Arrow keys (0x210-0x213)
    Left = 0x210,
    Right = 0x211,
    Up = 0x212,
    Down = 0x213,

    // Additional special keys
    LeftBracket = 0x220,
    RightBracket = 0x221,
    Backslash = 0x222,
    Semicolon = 0x223,
    Quote = 0x224,
    Comma = 0x225,
    Period = 0x226,
    Slash = 0x227,
    Grave = 0x228,
    Minus = 0x229,
    Equal = 0x22A,

    // Numpad keys (0x230+)
    Numpad0 = 0x230,
    Numpad1 = 0x231,
    Numpad2 = 0x232,
    Numpad3 = 0x233,
    Numpad4 = 0x234,
    Numpad5 = 0x235,
    Numpad6 = 0x236,
    Numpad7 = 0x237,
    Numpad8 = 0x238,
    Numpad9 = 0x239,
    NumpadDivide = 0x23A,
    NumpadMultiply = 0x23B,
    NumpadSubtract = 0x23C,
    NumpadAdd = 0x23D,
    NumpadEnter = 0x23E,
    NumpadDecimal = 0x23F,

    // Extended function keys (0x240+)
    F13 = 0x240,
    F14 = 0x241,
    F15 = 0x242,
    F16 = 0x243,
    F17 = 0x244,
    F18 = 0x245,
    F19 = 0x246,
    F20 = 0x247,
    F21 = 0x248,
    F22 = 0x249,
    F23 = 0x24A,
    F24 = 0x24B,

    // Media keys (0x250+)
    Mute = 0x250,
    VolumeDown = 0x251,
    VolumeUp = 0x252,
    MediaPlayPause = 0x253,
    MediaStop = 0x254,
    MediaPrevious = 0x255,
    MediaNext = 0x256,

    // System keys (0x260+)
    Power = 0x260,
    Sleep = 0x261,
    Wake = 0x262,

    // Browser keys (0x270+)
    BrowserBack = 0x270,
    BrowserForward = 0x271,
    BrowserRefresh = 0x272,
    BrowserStop = 0x273,
    BrowserSearch = 0x274,
    BrowserFavorites = 0x275,
    BrowserHome = 0x276,

    // Application keys (0x280+)
    AppMail = 0x280,
    AppCalculator = 0x281,
    AppMyComputer = 0x282,

    // Additional keys
    Menu = 0x290,
    Help = 0x291,
    Select = 0x292,
    Execute = 0x293,
    Undo = 0x294,
    Redo = 0x295,
    Cut = 0x296,
    Copy = 0x297,
    Paste = 0x298,
    Find = 0x299,

    // Japanese JIS keyboard keys (0x300+)
    // 全角/半角 (Zenkaku/Hankaku) - IME toggle
    Zenkaku = 0x300,
    // カタカナ (Katakana mode)
    Katakana = 0x301,
    // ひらがな (Hiragana mode)
    Hiragana = 0x302,
    // 変換 (Henkan) - IME conversion
    Henkan = 0x303,
    // 無変換 (Muhenkan) - IME non-conversion
    Muhenkan = 0x304,
    // ¥ (Yen key)
    Yen = 0x305,
    // ろ (Ro key) - JIS backslash position
    Ro = 0x306,
    // カタカナ/ひらがな toggle
    KatakanaHiragana = 0x307,

    // Korean keyboard keys (0x310+)
    // 한글 (Hangeul/Hangul) - Korean input toggle
    Hangeul = 0x310,
    // 한자 (Hanja) - Chinese character input
    Hanja = 0x311,

    // ISO/European keyboard keys (0x320+)
    // Extra key between left shift and Z on ISO keyboards
    Iso102nd = 0x320,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_has_all_expected_variants() {
        // Test letters
        assert_eq!(KeyCode::A as u16, 0x00);
        assert_eq!(KeyCode::Z as u16, 0x19);

        // Test numbers
        assert_eq!(KeyCode::Num0 as u16, 0x20);
        assert_eq!(KeyCode::Num9 as u16, 0x29);

        // Test function keys
        assert_eq!(KeyCode::F1 as u16, 0x30);
        assert_eq!(KeyCode::F12 as u16, 0x3B);

        // Test modifiers
        assert_eq!(KeyCode::LShift as u16, 0x100);
        assert_eq!(KeyCode::RMeta as u16, 0x107);

        // Test special keys
        assert_eq!(KeyCode::Escape as u16, 0x200);
        assert_eq!(KeyCode::Enter as u16, 0x201);

        // Test arrow keys
        assert_eq!(KeyCode::Left as u16, 0x210);
        assert_eq!(KeyCode::Down as u16, 0x213);
    }
}
