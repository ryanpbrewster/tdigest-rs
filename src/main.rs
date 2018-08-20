extern crate rand;
extern crate tdigest;

use rand::distributions::{Distribution, Normal, Uniform};
use rand::{Rng, SeedableRng, XorShiftRng};
use tdigest::{Estimator, Oracle};

fn main() {
    println!("Uniform");
    check_accuracy(Uniform::new(0.0, 100.0), 1_000_000);

    println!("Normal");
    check_accuracy(Normal::new(0.0, 100.0), 1_000_000);
}

fn check_accuracy<D: Distribution<f64>>(dist: D, size: usize) {
    let mut prng = XorShiftRng::from_seed([42; 16]);

    let mut e = tdigest::simple::TDigest::new(100.0, 1000);
    let mut buf = Vec::new();
    for _ in 0..size {
        let x = prng.sample(&dist);
        e.add(x);
        buf.push(x);
    }

    let oracle = Oracle::new(buf);

    let quantiles = vec![
        0.00001, 0.0001, 0.001, 0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99, 0.999,
        0.9999, 0.99999,
    ];
    for q in quantiles {
        let expected = oracle.quantile(q);
        let actual = e.estimate(q);
        let rank = oracle.rank(actual);
        println!(
            "{:7.5} --- {:11.4} {:11.4} ({:11.8})",
            q,
            expected,
            actual,
            rank - q,
        );
    }
}
