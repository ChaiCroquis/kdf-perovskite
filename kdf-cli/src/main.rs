//! KDF CLI - Command-line interface for Knowledge Decay Framework
//!
//! Usage:
//!   kdf analyze <input>           # Analyze data and show layer classification
//!   kdf dedupe <input>            # Deduplicate and output selected items
//!   kdf stats <input>             # Show statistics only

use clap::{Parser, Subcommand};
use kdf::{cosine_similarity, levenshtein_similarity, Kdf};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kdf")]
#[command(author = "KDF Team")]
#[command(version = "0.1.0")]
#[command(about = "Knowledge Decay Framework - Reduce redundancy, preserve rare items")]
#[command(long_about = r#"
KDF (Knowledge Decay Framework) is a data curation tool that:
  - Reduces redundant data automatically
  - Preserves rare/isolated items
  - Works without labels (unsupervised)

Examples:
  kdf analyze logs.txt                    # Analyze log file
  kdf analyze data.csv --format csv       # Analyze CSV file
  kdf dedupe docs.txt -o unique.txt       # Deduplicate and save
  kdf stats data.csv --format csv         # Show statistics only
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze data and show layer classification
    Analyze {
        /// Input file path
        input: PathBuf,

        /// Input format: text, csv, json
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Similarity threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.7")]
        threshold: f64,

        /// Similarity function: levenshtein, cosine
        #[arg(short, long, default_value = "levenshtein")]
        similarity: String,

        /// CSV column to use (for csv format)
        #[arg(long, default_value = "0")]
        column: usize,

        /// Show only Rare layer items
        #[arg(long)]
        rare_only: bool,
    },

    /// Deduplicate and output selected items
    Dedupe {
        /// Input file path
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Input format: text, csv, json
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Similarity threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.7")]
        threshold: f64,

        /// Similarity function: levenshtein, cosine
        #[arg(short, long, default_value = "levenshtein")]
        similarity: String,

        /// CSV column to use (for csv format)
        #[arg(long, default_value = "0")]
        column: usize,
    },

    /// Show statistics only
    Stats {
        /// Input file path
        input: PathBuf,

        /// Input format: text, csv, json
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Similarity threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.7")]
        threshold: f64,

        /// Similarity function: levenshtein, cosine
        #[arg(short, long, default_value = "levenshtein")]
        similarity: String,

        /// CSV column to use (for csv format)
        #[arg(long, default_value = "0")]
        column: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            input,
            format,
            threshold,
            similarity,
            column,
            rare_only,
        } => {
            run_analyze(&input, &format, threshold, &similarity, column, rare_only);
        }
        Commands::Dedupe {
            input,
            output,
            format,
            threshold,
            similarity,
            column,
        } => {
            run_dedupe(
                &input,
                output.as_ref(),
                &format,
                threshold,
                &similarity,
                column,
            );
        }
        Commands::Stats {
            input,
            format,
            threshold,
            similarity,
            column,
            json,
        } => {
            run_stats(&input, &format, threshold, &similarity, column, json);
        }
    }
}

fn load_data(path: &PathBuf, format: &str, column: usize) -> Vec<String> {
    match format {
        "csv" => load_csv(path, column),
        "json" => load_json(path),
        _ => load_text(path),
    }
}

fn load_text(path: &PathBuf) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn load_csv(path: &PathBuf, column: usize) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let mut reader = csv::Reader::from_reader(file);
    reader
        .records()
        .filter_map(|record| record.ok())
        .filter_map(|record| record.get(column).map(|s| s.to_string()))
        .collect()
}

fn load_json(path: &PathBuf) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let data: Vec<String> = serde_json::from_reader(reader).expect("Failed to parse JSON");
    data
}

fn run_analyze(
    input: &PathBuf,
    format: &str,
    threshold: f64,
    similarity: &str,
    column: usize,
    rare_only: bool,
) {
    let data = load_data(input, format, column);
    if data.is_empty() {
        eprintln!("Error: No data found in input file");
        return;
    }

    println!("# KDF Analysis\n");
    println!("Input: {} ({} items)", input.display(), data.len());
    println!("Threshold: {}", threshold);
    println!("Similarity: {}\n", similarity);

    let kdf = Kdf::with_defaults();
    let result = match similarity {
        "cosine" => {
            let vectors: Vec<Vec<f64>> = data.iter().map(|s| text_to_vector(s)).collect();
            kdf.process(&vectors, threshold, |a, b| cosine_similarity(a, b))
        }
        _ => kdf.process(&data, threshold, |a, b| levenshtein_similarity(a, b)),
    };

    let core_items = result.core_items();
    let edge_items = result.edge_items();
    let rare_items = result.rare_items();

    println!("## Layer Distribution\n");
    println!("| Layer | Count | Percentage |");
    println!("|-------|-------|------------|");
    println!(
        "| Core  | {:>5} | {:>9.1}% |",
        core_items.len(),
        100.0 * core_items.len() as f64 / data.len() as f64
    );
    println!(
        "| Edge  | {:>5} | {:>9.1}% |",
        edge_items.len(),
        100.0 * edge_items.len() as f64 / data.len() as f64
    );
    println!(
        "| Rare  | {:>5} | {:>9.1}% |",
        rare_items.len(),
        100.0 * rare_items.len() as f64 / data.len() as f64
    );

    let compression = 100.0 * (1.0 - result.selected.len() as f64 / data.len() as f64);
    println!("\nCompression: {:.1}%", compression);
    println!("Selected: {} / {}", result.selected.len(), data.len());

    if rare_only {
        println!("\n## Rare Layer Items\n");
        for &i in rare_items.iter() {
            let truncated: String = data[i].chars().take(80).collect();
            println!("  {}", truncated);
        }
    } else {
        println!("\n## Selected Items by Layer\n");

        if !rare_items.is_empty() {
            println!("### Rare (preserved due to isolation)\n");
            for &i in rare_items.iter().take(10) {
                let truncated: String = data[i].chars().take(60).collect();
                println!("  → {}", truncated);
            }
            if rare_items.len() > 10 {
                println!("  ... and {} more", rare_items.len() - 10);
            }
        }

        if !edge_items.is_empty() {
            println!("\n### Edge (moderate connectivity)\n");
            for &i in edge_items.iter().take(5) {
                let truncated: String = data[i].chars().take(60).collect();
                println!("    {}", truncated);
            }
            if edge_items.len() > 5 {
                println!("  ... and {} more", edge_items.len() - 5);
            }
        }

        if !core_items.is_empty() {
            println!("\n### Core (high redundancy)\n");
            for &i in core_items.iter().take(5) {
                let truncated: String = data[i].chars().take(60).collect();
                println!("    {}", truncated);
            }
            if core_items.len() > 5 {
                println!("  ... and {} more", core_items.len() - 5);
            }
        }
    }
}

fn run_dedupe(
    input: &PathBuf,
    output: Option<&PathBuf>,
    format: &str,
    threshold: f64,
    similarity: &str,
    column: usize,
) {
    let data = load_data(input, format, column);
    if data.is_empty() {
        eprintln!("Error: No data found in input file");
        return;
    }

    let kdf = Kdf::with_defaults();
    let result = match similarity {
        "cosine" => {
            let vectors: Vec<Vec<f64>> = data.iter().map(|s| text_to_vector(s)).collect();
            kdf.process(&vectors, threshold, |a, b| cosine_similarity(a, b))
        }
        _ => kdf.process(&data, threshold, |a, b| levenshtein_similarity(a, b)),
    };

    let selected_data: Vec<&String> = result.selected.iter().map(|&i| &data[i]).collect();

    if let Some(output_path) = output {
        let mut file = File::create(output_path).expect("Failed to create output file");
        for item in &selected_data {
            writeln!(file, "{}", item).expect("Failed to write");
        }
        println!(
            "Deduplicated: {} → {} items (saved to {})",
            data.len(),
            selected_data.len(),
            output_path.display()
        );
    } else {
        for item in &selected_data {
            println!("{}", item);
        }
        eprintln!(
            "\n# Deduplicated: {} → {} items",
            data.len(),
            selected_data.len()
        );
    }
}

fn run_stats(
    input: &PathBuf,
    format: &str,
    threshold: f64,
    similarity: &str,
    column: usize,
    json_output: bool,
) {
    let data = load_data(input, format, column);
    if data.is_empty() {
        eprintln!("Error: No data found in input file");
        return;
    }

    let kdf = Kdf::with_defaults();
    let result = match similarity {
        "cosine" => {
            let vectors: Vec<Vec<f64>> = data.iter().map(|s| text_to_vector(s)).collect();
            kdf.process(&vectors, threshold, |a, b| cosine_similarity(a, b))
        }
        _ => kdf.process(&data, threshold, |a, b| levenshtein_similarity(a, b)),
    };

    let stats = serde_json::json!({
        "input": input.to_string_lossy(),
        "total": data.len(),
        "selected": result.selected.len(),
        "compression_rate": 1.0 - result.selected.len() as f64 / data.len() as f64,
        "layers": {
            "core": result.core_items().len(),
            "edge": result.edge_items().len(),
            "rare": result.rare_items().len()
        },
        "threshold": threshold,
        "similarity": similarity
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&stats).unwrap());
    } else {
        println!("Input:       {}", input.display());
        println!("Total:       {}", data.len());
        println!("Selected:    {}", result.selected.len());
        println!(
            "Compression: {:.1}%",
            100.0 * (1.0 - result.selected.len() as f64 / data.len() as f64)
        );
        println!("---");
        println!("Core:        {}", result.core_items().len());
        println!("Edge:        {}", result.edge_items().len());
        println!("Rare:        {}", result.rare_items().len());
    }
}

/// Simple text to vector conversion (bag of words)
fn text_to_vector(text: &str) -> Vec<f64> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut vector = vec![0.0; 256];
    for word in words {
        let hash = word
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize))
            % 256;
        vector[hash] += 1.0;
    }
    // Normalize
    let norm: f64 = vector.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 0.0 {
        for v in &mut vector {
            *v /= norm;
        }
    }
    vector
}
