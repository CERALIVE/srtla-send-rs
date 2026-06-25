//! Allocator micro-benchmark: an SRT packet-churn workload measured under the
//! global allocator selected at build time.
//!
//! Run both configurations to compare:
//!   cargo bench --bench alloc                       # mimalloc (default)
//!   cargo bench --bench alloc --no-default-features # system allocator
//!
//! Each run emits a greppable `allocator=<name> <ops/s>` line for the evidence
//! ledger in addition to the criterion report.

use std::hint::black_box;
use std::time::Instant;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

#[cfg(all(not(windows), feature = "mimalloc-allocator"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

const ALLOCATOR: &str = if cfg!(feature = "mimalloc-allocator") {
    "mimalloc"
} else {
    "system"
};

const SRT_PKT_SIZE: usize = 1316;
const BATCH: usize = 256;

fn packet_churn() {
    let mut bufs: Vec<Vec<u8>> = Vec::with_capacity(BATCH);
    for i in 0..BATCH {
        let mut buf = vec![0u8; SRT_PKT_SIZE];
        buf[0] = i as u8;
        buf[SRT_PKT_SIZE - 1] = i as u8;
        bufs.push(buf);
    }
    black_box(&bufs);
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc");
    group.throughput(Throughput::Elements(BATCH as u64));
    group.bench_function(format!("packet_churn allocator={ALLOCATOR}"), |b| {
        b.iter(packet_churn);
    });
    group.finish();

    let iters = 50_000u64;
    let start = Instant::now();
    for _ in 0..iters {
        packet_churn();
    }
    let ops = (iters * BATCH as u64) as f64 / start.elapsed().as_secs_f64();
    println!("allocator={ALLOCATOR} {ops:.0}");
}

criterion_group!(benches, bench);
criterion_main!(benches);
