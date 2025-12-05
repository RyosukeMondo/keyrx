//! Output formatting for CLI commands.

pub mod formatter;

pub use formatter::{OutputFormat, OutputFormatter};
use serde::Serialize;
use std::io::{self, Write};

/// Writer for formatted CLI output.
#[derive(Debug)]
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
        let json = OutputFormatter::format_data(self, data)?;
        println!("{json}");
        io::stdout().flush()
    }
}

impl OutputFormatter for OutputWriter {
    fn format_success(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[OK] {}", message),
            OutputFormat::Json => format!(r#"{{"status":"success","message":"{}"}}"#, message),
        }
    }

    fn format_error(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[ERROR] {}", message),
            OutputFormat::Json => format!(r#"{{"status":"error","message":"{}"}}"#, message),
        }
    }

    fn format_warning(&self, message: &str) -> String {
        match self.format {
            OutputFormat::Human => format!("[WARN] {}", message),
            OutputFormat::Json => format!(r#"{{"status":"warning","message":"{}"}}"#, message),
        }
    }

    fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String> {
        match self.format {
            OutputFormat::Human => serde_json::to_string_pretty(data),
            OutputFormat::Json => serde_json::to_string(data),
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
            r#"{"status":"success","message":"ok"}"#
        );
        assert_eq!(
            writer.format_warning("watch"),
            r#"{"status":"warning","message":"watch"}"#
        );
        assert_eq!(
            writer.format_error("fail"),
            r#"{"status":"error","message":"fail"}"#
        );
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
        assert_eq!(compact, "{\"name\":\"alpha\",\"count\":3}");
    }
}
