//! Single-shot benchmark summary runner.
//!
//! Generates fixtures, runs each phase once, and prints a markdown table
//! to stdout. Intended for inclusion in `docs/benchmarks.md`.
//!
//! Run with:
//!   cargo run --release --example bench_summary

#[path = "../benches/fixtures.rs"]
mod fixtures;

use fixtures::*;

fn main() {
    let sizes: &[usize] = &[100, 1000, 5000];

    // Environment header.
    println!("# CodeCompass Benchmark Results");
    println!();
    println!("Generated: {}", current_iso());
    println!();
    println!("- OS: {}", std::env::consts::OS);
    println!("- Arch: {}", std::env::consts::ARCH);
    println!("- Profile: release");
    println!();
    println!("| Files | Scan (ms) | Analyze (ms) | Graph (ms) | Unchanged rescan (ms) | Modified rescan (ms) | Imports | Symbols |");
    println!("|------:|----------:|-------------:|-----------:|----------------------:|---------------------:|--------:|--------:|");

    for &n in sizes {
        let fx = make_fixture(n);
        let scan_us = bench_scan(&fx);
        let analyze_us = bench_analyze(&fx);
        let graph_us = bench_graph(&fx);
        let unchanged_us = bench_scan(&fx);
        let modified_us = bench_modified_rescan(&fx, 10);
        let counts = count_all(&fx);
        println!(
            "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {} | {} |",
            n,
            scan_us as f64 / 1000.0,
            analyze_us as f64 / 1000.0,
            graph_us as f64 / 1000.0,
            unchanged_us as f64 / 1000.0,
            modified_us as f64 / 1000.0,
            counts.imports,
            counts.symbols,
        );
    }
}

fn current_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{secs}")
}
