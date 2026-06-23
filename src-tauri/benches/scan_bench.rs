//! Reproducible scan/analysis/graph benchmark.
//!
//! Run via `cargo bench` to get Criterion statistical reports under
//! `target/criterion/`. Fixtures are generated at runtime in a temp dir,
//! so no large data is committed.
//!
//! For a single-shot markdown summary, run:
//!   `cargo run --release --example bench_summary`

mod fixtures;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fixtures::*;

const SIZES: &[usize] = &[100, 1000];

fn bench_scan_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_with_setup(|| make_fixture(n), |fx| bench_scan(&fx));
        });
    }
    group.finish();
}

fn bench_analyze_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_with_setup(
                || {
                    let fx = make_fixture(n);
                    bench_scan(&fx);
                    fx
                },
                |fx| bench_analyze(&fx),
            );
        });
    }
    group.finish();
}

fn bench_graph_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph");
    for &n in SIZES {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_with_setup(
                || {
                    let fx = make_fixture(n);
                    bench_scan(&fx);
                    bench_analyze(&fx);
                    fx
                },
                |fx| bench_graph(&fx),
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_scan_sizes,
    bench_analyze_sizes,
    bench_graph_sizes
);
criterion_main!(benches);
