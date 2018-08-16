extern crate itertools;

use itertools::Itertools;

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};

pub trait Estimator {
    fn add(&mut self, x: f64);
    fn estimate(&mut self, q: f64) -> f64;
}

#[derive(Debug)]
pub struct TDigest {
    max_size: usize,
    centroids: Vec<Centroid>,
    buffer: Vec<f64>,
}

struct Centroid {
    sum: f64,
    weight: f64,
}

impl Debug for Centroid {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Centroid({:9.6}; {})",
            self.sum / self.weight,
            self.weight
        )
    }
}

impl Centroid {
    fn new() -> Centroid {
        Centroid {
            sum: 0.0,
            weight: 0.0,
        }
    }
    fn single(x: f64) -> Centroid {
        Centroid {
            sum: x,
            weight: 1.0,
        }
    }
    fn add(&mut self, rhs: &Centroid) {
        self.sum += rhs.sum;
        self.weight += rhs.weight;
    }
}

impl PartialEq for Centroid {
    fn eq(&self, other: &Centroid) -> bool {
        self.sum / self.weight == other.sum / other.weight
    }
}

impl PartialOrd for Centroid {
    fn partial_cmp(&self, other: &Centroid) -> Option<Ordering> {
        let m1 = self.sum / self.weight;
        let m2 = other.sum / other.weight;
        m1.partial_cmp(&m2)
    }
}

fn q(k: f64) -> f64 {
    assert!(0.0 <= k && k <= 1.0, "{} expected to be in [0,1]", k);
    if k <= 0.5 {
        2.0 * k * k
    } else {
        1.0 - q(1.0 - k)
    }
}

impl TDigest {
    pub fn new(max_size: usize) -> TDigest {
        TDigest {
            max_size,
            centroids: Vec::new(),
            buffer: Vec::new(),
        }
    }
    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        self.buffer.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let new_centroids: Vec<Centroid> =
            self.buffer.iter().map(|&x| Centroid::single(x)).collect();
        self.buffer.clear();

        self.centroids = TDigest::merge(&self.centroids, &new_centroids, self.max_size);
    }

    fn merge(a: &[Centroid], b: &[Centroid], max_size: usize) -> Vec<Centroid> {
        let mut result = Vec::with_capacity(max_size);

        let total_weight: f64 = a.iter().chain(b.iter()).map(|c| c.weight).sum();

        let mut k1 = 1;
        let mut weight_so_far = 0.0;
        let mut weight_to_break = q(k1 as f64 / max_size as f64) * total_weight;

        let mut acc = Centroid::new();
        for cur in a.iter().merge(b.iter()) {
            if weight_so_far >= weight_to_break {
                result.push(acc);
                acc = Centroid::new();
                k1 += 1;
                weight_to_break = q(k1 as f64 / max_size as f64) * total_weight;
            }

            weight_so_far += cur.weight;
            acc.add(cur);
        }
        result.push(acc);

        // TODO(rpb): see if calling `result.sort()` is necessary due to floating point arithmetic
        result
    }
}

impl Estimator for TDigest {
    fn add(&mut self, x: f64) {
        self.buffer.push(x);
        if self.buffer.len() >= self.max_size {
            self.flush_buffer();
        }
    }

    fn estimate(&mut self, q: f64) -> f64 {
        self.flush_buffer();
        if self.centroids.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = self.centroids.iter().map(|c| c.weight).sum();
        let target_weight = q * total_weight;

        let mut acc_weight = 0.0;
        for i in 0..self.centroids.len() - 1 {
            if acc_weight + self.centroids[i].weight > target_weight {
                // The target is somewhere between this and the next centroid, so we interpolate.
                let cur = self.centroids[i].sum / self.centroids[i].weight;
                let next = self.centroids[i + 1].sum / self.centroids[i + 1].weight;
                return cur + (next - cur) * (target_weight - acc_weight) / self.centroids[i].weight;
            }
            acc_weight += self.centroids[i].weight;
        }

        let last = self.centroids.last().expect("centroids is non-empty");
        last.sum / last.weight
    }
}
