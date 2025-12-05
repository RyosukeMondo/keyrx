//! YAML implementation of the CLI output formatter.
//!
//! Mirrors the JSON formatter structure while emitting valid YAML without
//! trailing blank lines to avoid double newlines when printed.

use super::formatter::OutputFormatter;
use serde::Serialize;

/// Formatter that renders CLI output as YAML.
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlFormatter;

impl YamlFormatter {}

fn trim_trailing_newline(mut text: String) -> String {
    if text.ends_with('\n') {
        text.pop();
    }
    text
}

#[derive(Serialize)]
struct StatusMessage<'a> {
    status: &'a str,
    message: &'a str,
}

#[derive(Serialize)]
struct ErrorMessage<'a> {
    status: &'a str,
    message: &'a str,
    detail: Option<String>,
}

fn render_status(status: &str, message: &str) -> String {
    let payload = StatusMessage { status, message };
    serde_yaml::to_string(&payload)
        .map(trim_trailing_newline)
        .unwrap_or_else(fallback_error)
}

fn fallback_error(err: serde_yaml::Error) -> String {
    let payload = ErrorMessage {
        status: "error",
        message: "failed to serialize formatter output",
        detail: Some(err.to_string()),
    };

    serde_yaml::to_string(&payload)
        .map(trim_trailing_newline)
        .unwrap_or_else(|_| {
            format!(
                "status: error\nmessage: failed to serialize formatter output\ndetail: {}",
                err
            )
        })
}

impl OutputFormatter for YamlFormatter {
    fn format_success(&self, message: &str) -> String {
        render_status("success", message)
    }

    fn format_error(&self, message: &str) -> String {
        render_status("error", message)
    }

    fn format_warning(&self, message: &str) -> String {
        render_status("warning", message)
    }

    fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String> {
        serde_yaml::to_string(data)
            .map(trim_trailing_newline)
            .map_err(|err| serde_json::Error::io(std::io::Error::other(err.to_string())))
    }
}
