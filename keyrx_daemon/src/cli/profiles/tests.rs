use super::args::parse_template;
use super::handlers_io::{format_time, truncate};
use crate::config::profile_manager::ProfileTemplate;

#[test]
fn test_parse_template() {
    assert!(matches!(
        parse_template("blank").expect("blank template should parse"),
        ProfileTemplate::Blank
    ));
    assert!(matches!(
        parse_template("simple_remap").expect("simple_remap template should parse"),
        ProfileTemplate::SimpleRemap
    ));
    assert!(matches!(
        parse_template("capslock_escape").expect("capslock_escape template should parse"),
        ProfileTemplate::CapslockEscape
    ));
    assert!(matches!(
        parse_template("vim_navigation").expect("vim_navigation template should parse"),
        ProfileTemplate::VimNavigation
    ));
    assert!(matches!(
        parse_template("gaming").expect("gaming template should parse"),
        ProfileTemplate::Gaming
    ));
    assert!(parse_template("invalid").is_err());
}

#[test]
fn test_truncate() {
    assert_eq!(truncate("short", 10), "short");
    assert_eq!(truncate("verylongstring", 8), "veryl...");
    assert_eq!(truncate("abc", 2), "ab");
}

#[test]
fn test_format_time() {
    use std::time::{Duration, SystemTime};

    let now = SystemTime::now();
    assert_eq!(format_time(&now), "just now");

    let five_min_ago = now - Duration::from_secs(300);
    assert_eq!(format_time(&five_min_ago), "5m ago");

    let two_hours_ago = now - Duration::from_secs(7200);
    assert_eq!(format_time(&two_hours_ago), "2h ago");

    let three_days_ago = now - Duration::from_secs(259200);
    assert_eq!(format_time(&three_days_ago), "3d ago");
}
