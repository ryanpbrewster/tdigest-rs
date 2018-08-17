extern crate rand;
extern crate tdigest;

use rand::distributions::{Distribution, Normal, Uniform};
use rand::{Rng, SeedableRng, XorShiftRng};
use tdigest::Estimator;

fn main() {
    println!("U(0, 100)");
    check_accuracy(Uniform::new(0.0, 100.0), 100_000);

    println!("N(10^3; 10^2)");
    check_accuracy(Normal::new(1000.0, 100.0), 100_000);
}

fn check_accuracy<D: Distribution<f64>>(dist: D, size: usize) {
    let mut prng = XorShiftRng::from_seed([42; 16]);

    let mut e = tdigest::TDigest::new(128);
    let mut buf = Vec::new();
    for _ in 0..size {
        let x = prng.sample(&dist);
        e.add(x);
        buf.push(x);
    }

    buf.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let quantiles = vec![
        0.0001, 0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99, 0.9999,
    ];
    for q in quantiles {
        let expected = buf[(q * buf.len() as f64) as usize];
        let actual = e.estimate(q);
        let err = (actual - expected) / expected;
        println!(
            "{:6.4} --- {:6.4} ({:6.4} {:6.4})",
            q,
            100.0 * err,
            expected,
            actual
        );
    }
}
