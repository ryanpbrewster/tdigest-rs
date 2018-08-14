extern crate rand;
extern crate tdigest;

use rand::{Rng, SeedableRng};
use tdigest::Estimator;

fn main() {
    let mut e = tdigest::TDigest::new(16);

    let mut prng = rand::thread_rng();
    for _ in 0..1_000_000 {
        e.add(prng.gen_range(0.0, 100.0));
    }

    println!("{:#?}", e);
}
