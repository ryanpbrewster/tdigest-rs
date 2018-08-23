extern crate rand;
extern crate tdigest;

use rand::distributions::Uniform;
use rand::{Rng, SeedableRng, XorShiftRng};
use tdigest::{simple::TDigest, Estimator};

fn main() {
    let dist = Uniform::new(0.0, 100.0);
    let mut prng = XorShiftRng::from_seed([42; 16]);
    let mut e = TDigest::new(100.0, 1000);
    loop {
        e.add(prng.sample(&dist));
    }
}
