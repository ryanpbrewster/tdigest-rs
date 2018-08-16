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
    min: f64,
    max: f64,
    buffer: Vec<f64>,
}

#[derive(Clone)]
struct Centroid {
    sum: f64,
    weight: f64,
    mean: f64,
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
            mean: 0.0,
        }
    }
    fn single(x: f64) -> Centroid {
        Centroid {
            sum: x,
            weight: 1.0,
            mean: x,
        }
    }
    fn add(&mut self, rhs: &Centroid) {
        self.sum += rhs.sum;
        self.weight += rhs.weight;
        self.mean = self.sum / self.weight;
    }
}

impl PartialEq for Centroid {
    fn eq(&self, other: &Centroid) -> bool {
        self.mean == other.mean
    }
}

impl PartialOrd for Centroid {
    fn partial_cmp(&self, other: &Centroid) -> Option<Ordering> {
        self.mean.partial_cmp(&other.mean)
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

/// Find the value `0 <= chi <= 1` percent of the way between `a` and `b`
fn interpolate(a: f64, b: f64, chi: f64) -> f64 {
    a + (b - a) * chi
}

impl TDigest {
    pub fn new(max_size: usize) -> TDigest {
        TDigest {
            max_size,
            centroids: Vec::new(),
            buffer: Vec::new(),
            min: std::f64::INFINITY,
            max: std::f64::NEG_INFINITY,
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
            acc.add(cur);
            weight_so_far += cur.weight;
        }
        result.push(acc);

        // TODO(rpb): see if calling `result.sort()` is necessary due to floating point arithmetic
        result
    }
}

impl Estimator for TDigest {
    fn add(&mut self, x: f64) {
        self.buffer.push(x);
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
        if self.buffer.len() >= self.max_size {
            self.flush_buffer();
        }
    }

    fn estimate(&mut self, q: f64) -> f64 {
        self.flush_buffer();
        if self.centroids.is_empty() {
            return 0.0;
        }
        let (first, last) = (
            self.centroids.first().unwrap(),
            self.centroids.last().unwrap(),
        );

        let total_weight: f64 = self.centroids.iter().map(|c| c.weight).sum();
        let target_weight = q * total_weight;

        if target_weight <= first.weight / 2.0 {
            return interpolate(self.min, first.mean, 2.0 * target_weight / first.weight);
        }

        let mut weight_so_far = self.centroids[0].weight / 2.0;
        for i in 0..self.centroids.len() - 1 {
            let dw = (self.centroids[i].weight + self.centroids[i + 1].weight) / 2.0;
            if weight_so_far + dw > target_weight {
                // The target is somewhere between this and the next centroid, so we interpolate.
                let cur = self.centroids[i].mean;
                let next = self.centroids[i + 1].mean;
                return cur + (next - cur) * (target_weight - weight_so_far) / dw;
            }
            weight_so_far += self.centroids[i].weight;
        }

        interpolate(
            last.mean,
            self.max,
            2.0 * (total_weight - weight_so_far) / last.weight,
        )
    }
}
