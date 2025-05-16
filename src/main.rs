//! CLI executable

use clap::{Parser, Subcommand};
use std::{fs, path::Path, process::Command as ProcessCommand};
use tree_pir::tree::generate_and_save_tree;

#[derive(Parser)]
#[command(about = "Tree generation and benchmarking CLI")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a tree with 2^exponent leaves
    GenTree { exponent: u32 },
    /// Run benchmarks
    Bench {
        #[command(subcommand)]
        bench: Bench,
    },
}

#[derive(Subcommand)]
enum Bench {
    Single { exponent: String },
    Batch { exponent: String },
    All,
}

const EXPONENTS: &[u32] = &[12, 16, 20, 24];

fn main() -> Result<(), String> {
    match Cli::parse().command {
        Cmd::GenTree { exponent } => {
            validate_exponent(exponent)?;
            generate_and_save_tree(exponent);
            println!("Generated tree with 2^{exponent} leaves");
        }
        Cmd::Bench { bench } => run_benchmarks(&bench)?,
    }
    Ok(())
}

fn validate_exponent(exp: u32) -> Result<(), String> {
    EXPONENTS.contains(&exp).then_some(()).ok_or(format!(
        "Invalid exponent {exp}. Use: {}",
        EXPONENTS
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn ensure_tree(exp: u32) {
    let path = format!("./trees/tree_{exp}.json");
    if !Path::new(&path).exists() {
        println!("Generating tree for 2^{exp} leaves...");
        fs::create_dir_all("./trees").expect("Failed to create ./trees");
        generate_and_save_tree(exp);
    }
}

fn run_benchmarks(bench: &Bench) -> Result<(), String> {
    let benches = match bench {
        Bench::Single { exponent } => select_benches("single", exponent)?,
        Bench::Batch { exponent } => select_benches("batch", exponent)?,
        Bench::All => EXPONENTS
            .iter()
            .flat_map(|&exp| [format!("single_{exp}"), format!("batch_{exp}")])
            .collect(),
    };

    if benches.is_empty() {
        return Err("No benchmarks selected".to_string());
    }

    for bench in benches {
        let exp = bench.split('_').next_back().unwrap().parse().unwrap();
        println!("Running {bench}...");
        ensure_tree(exp);
        let status = ProcessCommand::new("cargo")
            .args(["bench", "--bench", &bench])
            .status()
            .map_err(|e| format!("Failed to run {bench}: {e}"))?;
        if !status.success() {
            eprintln!("Benchmark {bench} failed: {status}");
        }
    }
    println!("Benchmarks completed");
    Ok(())
}

fn select_benches(prefix: &str, exponent: &str) -> Result<Vec<String>, String> {
    if exponent == "all" {
        Ok(EXPONENTS
            .iter()
            .map(|&exp| format!("{prefix}_{exp}"))
            .collect())
    } else {
        let exp: u32 = exponent.parse().map_err(|_| {
            format!(
                "Invalid exponent {exponent}. Use: all, {}",
                EXPONENTS
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        validate_exponent(exp)?;
        Ok(vec![format!("{prefix}_{exp}")])
    }
}
