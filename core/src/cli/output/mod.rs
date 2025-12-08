//! Output formatting for CLI commands.

pub mod formatter;
pub mod json;
pub mod table;
pub mod yaml;

pub use formatter::{OutputFormat, OutputFormatter};
use json::JsonFormatter;
use serde::Serialize;
use std::io::{self, Write};
use table::TableFormatter;
use yaml::YamlFormatter;

/// Writer for formatted CLI output.
#[derive(Debug, Clone)]
pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    /// Create a new output writer.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Get the output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Write a success message.
    pub fn success(&self, message: &str) {
        let line = OutputFormatter::format_success(self, message);
        println!("{line}");
        let _ = io::stdout().flush();
    }

    /// Write an error message.
    pub fn error(&self, message: &str) {
        let line = OutputFormatter::format_error(self, message);
        eprintln!("{line}");
        let _ = io::stderr().flush();
    }

    /// Write a warning message.
    pub fn warning(&self, message: &str) {
        let line = OutputFormatter::format_warning(self, message);
        println!("{line}");
        let _ = io::stdout().flush();
    }

    /// Write structured data.
    pub fn data<T: Serialize + ?Sized>(&self, data: &T) -> io::Result<()> {
        let rendered = OutputFormatter::format_data(self, data).map_err(io::Error::other)?;
        println!("{rendered}");
        io::stdout().flush()
    }
}

impl OutputFormatter for OutputWriter {
    fn format_success(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[OK] {}", message),
            OutputFormat::Json => JsonFormatter.format_success(message),
            OutputFormat::Table => TableFormatter.format_success(message),
            OutputFormat::Yaml => YamlFormatter.format_success(message),
        }
    }

    fn format_error(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[ERROR] {}", message),
            OutputFormat::Json => JsonFormatter.format_error(message),
            OutputFormat::Table => TableFormatter.format_error(message),
            OutputFormat::Yaml => YamlFormatter.format_error(message),
        }
    }

    fn format_warning(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[WARN] {}", message),
            OutputFormat::Json => JsonFormatter.format_warning(message),
            OutputFormat::Table => TableFormatter.format_warning(message),
            OutputFormat::Yaml => YamlFormatter.format_warning(message),
        }
    }

    fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String> {
        match self.format {
            OutputFormat::Human => serde_json::to_string_pretty(data),
            OutputFormat::Json => JsonFormatter.format_data(data),
            OutputFormat::Table => TableFormatter.format_data(data),
            OutputFormat::Yaml => YamlFormatter.format_data(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn formats_human_readable_messages() {
        let writer = OutputWriter::new(OutputFormat::Human);

        assert_eq!(writer.format_success("done"), "[OK] done");
        assert_eq!(writer.format_warning("check"), "[WARN] check");
        assert_eq!(writer.format_error("fail"), "[ERROR] fail");
    }

    #[test]
    fn formats_json_messages() {
        let writer = OutputWriter::new(OutputFormat::Json);

        assert_eq!(
            writer.format_success("ok"),
            "{\n  \"message\": \"ok\",\n  \"status\": \"success\"\n}"
        );
        assert_eq!(
            writer.format_warning("watch"),
            "{\n  \"message\": \"watch\",\n  \"status\": \"warning\"\n}"
        );
        assert_eq!(
            writer.format_error("fail"),
            "{\n  \"message\": \"fail\",\n  \"status\": \"error\"\n}"
        );
    }

    #[test]
    fn formats_yaml_messages() {
        let writer = OutputWriter::new(OutputFormat::Yaml);

        assert_eq!(writer.format_success("ok"), "status: success\nmessage: ok");
        assert_eq!(
            writer.format_warning("watch"),
            "status: warning\nmessage: watch"
        );
        assert_eq!(writer.format_error("fail"), "status: error\nmessage: fail");
    }

    #[derive(Serialize)]
    struct SampleData<'a> {
        name: &'a str,
        count: u8,
    }

    #[test]
    fn formats_data_for_each_mode() {
        let sample = SampleData {
            name: "alpha",
            count: 3,
        };

        let human = OutputWriter::new(OutputFormat::Human)
            .format_data(&sample)
            .expect("human data write should succeed");
        assert_eq!(human, "{\n  \"name\": \"alpha\",\n  \"count\": 3\n}");

        let compact = OutputWriter::new(OutputFormat::Json)
            .format_data(&sample)
            .expect("json data write should succeed");
        assert_eq!(compact, "{\n  \"name\": \"alpha\",\n  \"count\": 3\n}");

        let yaml = OutputWriter::new(OutputFormat::Yaml)
            .format_data(&sample)
            .expect("yaml data write should succeed");
        assert_eq!(yaml, "name: alpha\ncount: 3");

        let table = OutputWriter::new(OutputFormat::Table)
            .format_data(&sample)
            .expect("table data write should succeed");
        assert_eq!(
            table,
            "key   | value\n------+------\ncount | 3    \nname  | alpha"
        );
    }
}
