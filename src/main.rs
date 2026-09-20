use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use headercheck::scan::{find_files, has_header, insert_header};
use headercheck::template::Template;

#[derive(Parser)]
#[command(
    name = "headercheck",
    about = "Ensures every source file has the correct license/copyright header"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report every file missing the expected header.
    Check(Common),
    /// Insert the expected header into every file missing it.
    Fix(Common),
}

#[derive(Args)]
struct Common {
    /// Directory to scan.
    #[arg(default_value = ".")]
    dir: PathBuf,
    /// File extension to check (no leading dot), e.g. `rs`. Repeatable.
    #[arg(long = "ext", required = true)]
    extensions: Vec<String>,
    /// Read the exact header text from this file (supports a `{year}`
    /// placeholder). Mutually exclusive with --holder.
    #[arg(long, conflicts_with = "holder")]
    template: Option<PathBuf>,
    /// Build a one-line `<prefix> Copyright (c) <year> <holder>` header.
    /// Mutually exclusive with --template.
    #[arg(long, conflicts_with = "template")]
    holder: Option<String>,
    /// Comment prefix used with --holder.
    #[arg(long, default_value = "//")]
    prefix: String,
}

fn build_template(common: &Common) -> anyhow::Result<Template> {
    if let Some(path) = &common.template {
        let raw = fs::read_to_string(path)?;
        return Ok(Template::new(raw.trim_end_matches('\n').to_string()));
    }
    if let Some(holder) = &common.holder {
        return Ok(Template::new(format!(
            "{} Copyright (c) {{year}} {holder}",
            common.prefix
        )));
    }
    anyhow::bail!("give either --template <file> or --holder <name>");
}

fn current_year() -> i32 {
    chrono::Utc::now()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap_or(1970)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(exit_nonzero) => {
            if exit_nonzero {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("headercheck: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<bool> {
    match cli.command {
        Command::Check(common) => {
            let template = build_template(&common)?;
            let files = find_files(&common.dir, &common.extensions)?;
            let mut missing = 0;
            for path in &files {
                let content = fs::read_to_string(path)?;
                if !has_header(&content, &template) {
                    println!("{}: missing header", path.display());
                    missing += 1;
                }
            }
            if missing == 0 {
                println!(
                    "headercheck: all {} file(s) have the expected header",
                    files.len()
                );
                Ok(false)
            } else {
                eprintln!("\nheadercheck: {missing} file(s) missing the expected header");
                Ok(true)
            }
        }
        Command::Fix(common) => {
            let template = build_template(&common)?;
            let year = current_year();
            let files = find_files(&common.dir, &common.extensions)?;
            let mut fixed = 0;
            for path in &files {
                let content = fs::read_to_string(path)?;
                if !has_header(&content, &template) {
                    let updated = insert_header(&content, &template, year);
                    fs::write(path, updated)?;
                    println!("{}: header inserted", path.display());
                    fixed += 1;
                }
            }
            println!("headercheck: inserted header into {fixed} file(s)");
            Ok(false)
        }
    }
}
