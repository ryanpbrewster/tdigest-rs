#[macro_use]
extern crate criterion;
extern crate tdigest;

use criterion::{Benchmark, Criterion, Throughput};

use tdigest::simple::TDigest;
use tdigest::Estimator;

fn run(size: usize) -> f64 {
    let mut estimator = TDigest::new(100.0, 1000);
    let mut n = 1;
    for _ in 0..size {
        estimator.add(n as f64 / size as f64);
        n = (19 * n) % size;
    }
    estimator.estimate(0.99)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut size = 1;
    for i in 1..=4 {
        size *= 10;
        c.bench(
            "stream-size",
            Benchmark::new(format!("n10^{}", i), move |b| b.iter(|| run(size)))
                .throughput(Throughput::Elements(size as u32)),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
