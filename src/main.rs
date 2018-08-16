extern crate rand;
extern crate tdigest;

use rand::distributions::Uniform;
use rand::{Rng, SeedableRng, XorShiftRng};
use tdigest::Estimator;

fn main() {
    let mut e = tdigest::TDigest::new(128);
    let mut buf = Vec::new();

    let range = Uniform::new(0.0, 100.0);
    let mut prng = XorShiftRng::from_seed([42; 16]);
    for _ in 0..100_000 {
        let x = prng.sample(range);
        e.add(x);
        buf.push(x);
    }

    buf.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for q in vec![
        0.0001, 0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99, 0.9999,
    ] {
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
