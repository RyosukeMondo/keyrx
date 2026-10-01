//! Every code sample in `docs/user-guide/dsl-manual.md` must compile, so the
//! manual cannot rot again. The fence info string decides how a block is
//! checked:
//!
//! - ```` ```rhai ````            a complete script; must compile.
//! - ```` ```rhai,fragment ````   statements only; wrapped in
//!   `device_start("*"); ... device_end();` and must compile.
//! - ```` ```rhai,error ````      intentionally invalid (a documented
//!   mistake); must FAIL to compile.
//!
//! Blocks with any other info string (plain fences, `text`, ...) are not
//! checked. `rhai` blocks that do not declare a `device_start` are an
//! authoring error: mark them `fragment`.

use keyrx_compiler::parser::Parser;
use std::path::{Path, PathBuf};

struct Block {
    line: usize,
    kind: String,
    code: String,
}

fn manual_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/user-guide/dsl-manual.md")
}

fn extract_blocks(markdown: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current: Option<Block> = None;
    for (idx, line) in markdown.lines().enumerate() {
        match (&mut current, line.strip_prefix("```")) {
            (None, Some(info)) => {
                let kind = info.trim().to_string();
                if kind.starts_with("rhai") {
                    current = Some(Block {
                        line: idx + 1,
                        kind,
                        code: String::new(),
                    });
                }
            }
            (Some(_), Some(_)) => blocks.extend(current.take()),
            (Some(block), None) => {
                block.code.push_str(line);
                block.code.push('\n');
            }
            (None, None) => {}
        }
    }
    blocks
}

fn compiles(script: &str) -> Result<(), String> {
    Parser::new()
        .parse_string(script, Path::new("manual.rhai"))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn every_rhai_sample_in_the_manual_compiles_as_documented() {
    let markdown = std::fs::read_to_string(manual_path()).expect("manual readable");
    let blocks = extract_blocks(&markdown);
    assert!(blocks.len() > 20, "found only {} rhai blocks", blocks.len());

    let mut failures = Vec::new();
    for block in &blocks {
        let (script, expect_ok) = match block.kind.as_str() {
            "rhai" => (block.code.clone(), true),
            "rhai,fragment" => (
                format!("device_start(\"*\");\n{}\ndevice_end();\n", block.code),
                true,
            ),
            "rhai,error" => (block.code.clone(), false),
            other => {
                failures.push(format!(
                    "line {}: unknown fence `{other}` (use rhai, rhai,fragment or rhai,error)",
                    block.line
                ));
                continue;
            }
        };
        match (compiles(&script), expect_ok) {
            (Ok(()), true) | (Err(_), false) => {}
            (Err(e), true) => failures.push(format!("line {}: does not compile: {e}", block.line)),
            (Ok(()), false) => failures.push(format!(
                "line {}: marked rhai,error but compiles",
                block.line
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "{} manual sample(s) are wrong:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The scripts shipped in `examples/` and the daemon's profile templates
/// must compile too.
#[test]
fn every_shipped_example_and_template_compiles() {
    let mut failures = Vec::new();
    let mut seen = 0;
    for dir in ["../examples", "../keyrx_daemon/templates"] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        for entry in std::fs::read_dir(&dir).expect("script dir readable") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rhai") {
                continue;
            }
            seen += 1;
            if let Err(e) = Parser::new().parse_script(&path) {
                failures.push(format!("{}: {e}", path.display()));
            }
        }
    }
    assert!(seen >= 12, "found only {seen} scripts");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
