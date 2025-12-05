//! Table formatter for human-readable CLI output.
//!
//! This formatter renders structured data into aligned tables while
//! reusing the same textual prefixes for status messages as the
//! human-readable formatter.

use serde::Serialize;
use serde_json::{self, Map, Value};

/// Formatter that renders rows and columns as an aligned text table.
#[derive(Debug, Default, Clone, Copy)]
pub struct TableFormatter;

impl TableFormatter {
    /// Format a success message with a stable prefix.
    pub fn format_success(&self, message: &str) -> String {
        format!("[OK] {message}")
    }

    /// Format an error message with a stable prefix.
    pub fn format_error(&self, message: &str) -> String {
        format!("[ERROR] {message}")
    }

    /// Format a warning message with a stable prefix.
    pub fn format_warning(&self, message: &str) -> String {
        format!("[WARN] {message}")
    }

    /// Format structured data as an aligned table.
    pub fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String> {
        let value = serde_json::to_value(data)?;
        let (headers, rows) = normalize_table(&value);
        Ok(render_table(&headers, &rows))
    }
}

fn normalize_table(value: &Value) -> (Vec<String>, Vec<Vec<String>>) {
    match value {
        Value::Array(items) => normalize_array(items),
        Value::Object(map) => normalize_object(map),
        other => (
            vec!["value".to_string()],
            vec![vec![stringify_value(other)]],
        ),
    }
}

fn normalize_array(items: &[Value]) -> (Vec<String>, Vec<Vec<String>>) {
    if items.is_empty() {
        return (vec!["value".to_string()], Vec::new());
    }

    if items.iter().all(|v| v.is_object()) {
        let mut headers: Vec<String> = Vec::new();
        for item in items {
            if let Value::Object(map) = item {
                for key in map.keys() {
                    if !headers.contains(key) {
                        headers.push(key.clone());
                    }
                }
            }
        }

        let rows = items
            .iter()
            .map(|item| match item {
                Value::Object(map) => headers
                    .iter()
                    .map(|h| map.get(h).map(stringify_value).unwrap_or_default())
                    .collect(),
                _ => headers.iter().map(|_| String::new()).collect(),
            })
            .collect();

        return (headers, rows);
    }

    if items.iter().all(|v| matches!(v, Value::Array(_))) {
        let max_len = items
            .iter()
            .map(|v| match v {
                Value::Array(arr) => arr.len(),
                _ => 0,
            })
            .max()
            .unwrap_or(0);

        let headers = (1..=max_len).map(|i| format!("col{i}")).collect::<Vec<_>>();

        let rows = items
            .iter()
            .map(|item| match item {
                Value::Array(arr) => (0..max_len)
                    .map(|i| arr.get(i).map(stringify_value).unwrap_or_default())
                    .collect(),
                _ => vec![String::new(); max_len],
            })
            .collect();

        return (headers, rows);
    }

    (
        vec!["value".to_string()],
        items.iter().map(|v| vec![stringify_value(v)]).collect(),
    )
}

fn normalize_object(map: &Map<String, Value>) -> (Vec<String>, Vec<Vec<String>>) {
    let headers = vec!["key".to_string(), "value".to_string()];
    let rows = map
        .iter()
        .map(|(k, v)| vec![k.clone(), stringify_value(v)])
        .collect();
    (headers, rows)
}

fn render_table(headers: &[String], rows: &[Vec<String>]) -> String {
    let widths = column_widths(headers, rows);

    let mut lines = Vec::with_capacity(rows.len() + 2);
    lines.push(render_row(headers, &widths));
    lines.push(render_separator(&widths));

    for row in rows {
        lines.push(render_row(row, &widths));
    }

    lines.join("\n")
}

fn column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(idx) {
                *width = (*width).max(cell.len());
            } else {
                widths.push(cell.len());
            }
        }
    }

    widths
}

fn render_row(row: &[String], widths: &[usize]) -> String {
    let padded = widths
        .iter()
        .enumerate()
        .map(|(i, width)| {
            let cell = row.get(i).cloned().unwrap_or_default();
            format!("{cell:<width$}")
        })
        .collect::<Vec<_>>();

    padded.join(" | ")
}

fn render_separator(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|w| "-".repeat(*w))
        .collect::<Vec<_>>()
        .join("-+-")
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::Null => "".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                serde_json::to_string(value).unwrap_or_else(|_| "<array>".to_string())
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                serde_json::to_string(value).unwrap_or_else(|_| "<object>".to_string())
            }
        }
    }
}
