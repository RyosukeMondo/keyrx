use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod analyzer;
mod generator;
mod grouper;
mod types;

use analyzer::ModuleAnalyzer;
use generator::ModuleGenerator;
use grouper::FunctionGrouper;

#[derive(Parser)]
#[command(name = "module_splitter")]
#[command(about = "Automatically split large Rust modules into smaller, focused files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a module and suggest split boundaries
    Analyze {
        /// Path to the Rust module file to analyze
        #[arg(value_name = "FILE")]
        module: PathBuf,

        /// Maximum lines per output module (default: 500)
        #[arg(short, long, default_value = "500")]
        max_size: usize,

        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Preview the split plan without applying changes
    Preview {
        /// Path to the Rust module file to split
        #[arg(value_name = "FILE")]
        module: PathBuf,

        /// Maximum lines per output module (default: 500)
        #[arg(short, long, default_value = "500")]
        max_size: usize,
    },

    /// Apply the split to create new module files
    Apply {
        /// Path to the Rust module file to split
        #[arg(value_name = "FILE")]
        module: PathBuf,

        /// Maximum lines per output module (default: 500)
        #[arg(short, long, default_value = "500")]
        max_size: usize,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            module,
            max_size,
            format,
        } => {
            analyze_module(&module, max_size, &format)?;
        }
        Commands::Preview { module, max_size } => {
            preview_split(&module, max_size)?;
        }
        Commands::Apply {
            module,
            max_size,
            yes,
        } => {
            apply_split(&module, max_size, yes)?;
        }
    }

    Ok(())
}

fn analyze_module(module_path: &PathBuf, max_size: usize, format: &str) -> Result<()> {
    println!("{} {}", "Analyzing".cyan().bold(), module_path.display());

    let analyzer = ModuleAnalyzer::new(module_path)?;
    let analysis = analyzer.analyze()?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&analysis)?;
            println!("{}", json);
        }
        _ => {
            println!("\n{}", "Module Statistics:".green().bold());
            println!("  Total lines: {}", analysis.total_lines);
            println!("  Code lines: {}", analysis.code_lines);
            println!("  Functions: {}", analysis.functions.len());
            println!("  Types: {}", analysis.types.len());
            println!("  Imports: {}", analysis.imports.len());

            if analysis.code_lines > max_size {
                println!(
                    "\n{} Module exceeds max size ({} > {})",
                    "⚠".yellow().bold(),
                    analysis.code_lines,
                    max_size
                );
                println!(
                    "  Recommended: Split into {} modules",
                    analysis.code_lines.div_ceil(max_size)
                );
            } else {
                println!(
                    "\n{} Module is within size limit ({} <= {})",
                    "✓".green().bold(),
                    analysis.code_lines,
                    max_size
                );
            }
        }
    }

    Ok(())
}

fn preview_split(module_path: &PathBuf, max_size: usize) -> Result<()> {
    println!("{} {}", "Analyzing".cyan().bold(), module_path.display());

    let analyzer = ModuleAnalyzer::new(module_path)?;
    let analysis = analyzer.analyze()?;

    if analysis.code_lines <= max_size {
        println!(
            "\n{} Module is already within size limit ({} <= {})",
            "✓".green().bold(),
            analysis.code_lines,
            max_size
        );
        println!("No splitting needed.");
        return Ok(());
    }

    println!("\n{} Generating split plan...", "Planning".cyan().bold());

    let grouper = FunctionGrouper::new(max_size);
    let groups = grouper.group_functions(&analysis)?;

    println!("\n{}", "Split Plan:".green().bold());
    println!("  Will create {} new modules:", groups.len());
    println!();

    for (idx, group) in groups.iter().enumerate() {
        println!(
            "  {}. {} ({} lines, {} functions)",
            idx + 1,
            group.name.bright_blue().bold(),
            group.estimated_lines,
            group.functions.len()
        );

        if !group.functions.is_empty() {
            println!("     Functions:");
            for func in &group.functions {
                println!("       - {}", func);
            }
        }
        println!();
    }

    let total_estimated: usize = groups.iter().map(|g| g.estimated_lines).sum();
    println!("  Total estimated lines: {}", total_estimated);
    println!("  Original code lines: {}", analysis.code_lines);

    Ok(())
}

fn apply_split(module_path: &PathBuf, max_size: usize, skip_confirm: bool) -> Result<()> {
    println!("{} {}", "Analyzing".cyan().bold(), module_path.display());

    let analyzer = ModuleAnalyzer::new(module_path)?;
    let analysis = analyzer.analyze()?;

    if analysis.code_lines <= max_size {
        println!(
            "\n{} Module is already within size limit ({} <= {})",
            "✓".green().bold(),
            analysis.code_lines,
            max_size
        );
        println!("No splitting needed.");
        return Ok(());
    }

    println!("\n{} Generating split plan...", "Planning".cyan().bold());

    let grouper = FunctionGrouper::new(max_size);
    let groups = grouper.group_functions(&analysis)?;

    println!("\n{}", "Split Plan:".green().bold());
    println!("  Will create {} new modules\n", groups.len());

    for (idx, group) in groups.iter().enumerate() {
        println!(
            "  {}. {} ({} lines, {} functions)",
            idx + 1,
            group.name.bright_blue().bold(),
            group.estimated_lines,
            group.functions.len()
        );
    }

    if !skip_confirm {
        println!("\n{}", "Proceed with split? [y/N]: ".yellow().bold());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("\n{} Applying split...", "Executing".cyan().bold());

    let generator = ModuleGenerator::new(module_path, &analysis);
    let result = generator.generate_modules(&groups)?;

    println!("\n{}", "Split Complete!".green().bold());
    println!("  Created {} files:", result.created_files.len());
    for file in &result.created_files {
        println!("    ✓ {}", file.display());
    }

    if let Some(mod_file) = &result.mod_file {
        println!("  Created mod.rs: {}", mod_file.display());
    }

    println!("\n{}", "Next Steps:".yellow().bold());
    println!("  1. Review the generated files");
    println!("  2. Run 'cargo check' to verify compilation");
    println!("  3. Run tests to ensure functionality is preserved");
    println!("  4. Update imports in dependent modules");

    Ok(())
}
