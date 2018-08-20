#[macro_use]
extern crate criterion;
extern crate tdigest;

use criterion::{Benchmark, Criterion, Throughput};

use tdigest::simple::TDigest;
use tdigest::Estimator;

fn criterion_benchmark(c: &mut Criterion) {
    let mut size = 1;
    for i in 1..=4 {
        size *= 10;
        c.bench(
            "stream-size",
            Benchmark::new(format!("n10^{}", i), move |b| {
                b.iter(|| {
                    let mut estimator = TDigest::new(100.0, 1000);
                    let mut n = 1;
                    for _ in 0..size {
                        estimator.add(n as f64 / size as f64);
                        n = (19 * n) % size;
                    }
                    estimator.estimate(0.99)
                })
            }).throughput(Throughput::Elements(size as u32)),
        );
    }

    for &z in &vec![10, 100, 1_000] {
        for &s in &vec![10, 100, 1_000, 10_000] {
            let mut estimator = TDigest::new(z as f64, s as usize);
            let mut n = 1;
            c.bench(
                "tdigest-params",
                Benchmark::new(format!("z={},s={}", z, s), move |b| {
                    b.iter(|| {
                        estimator.add(n as f64 / size as f64);
                        n = (19 * n) % 1_000_000;
                    })
                }),
            );
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
