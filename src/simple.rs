use std::f64::consts;
use std::fmt;

#[derive(Debug)]
pub struct TDigest {
    compression: f64,
    max_size: usize,
    min: f64,
    max: f64,
    centroids: Vec<Centroid>,
    total_weight: f64,
    buffer: Vec<Centroid>,
}

#[derive(Clone)]
struct Centroid {
    weight: f64,
    mean: f64,
}

impl fmt::Debug for Centroid {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Centroid({:9.6}; {})", self.mean, self.weight)
    }
}

impl Centroid {
    fn single(x: f64) -> Centroid {
        Centroid {
            weight: 1.0,
            mean: x,
        }
    }
    fn add(&mut self, other: &Centroid) {
        self.mean =
            (self.mean * self.weight + other.mean * other.weight) / (self.weight + other.weight);
        self.weight += other.weight;
    }
}

fn weighted_average(x1: f64, w1: f64, x2: f64, w2: f64) -> f64 {
    (x1 * w1 + x2 * w2) / (w1 + w2)
}

impl TDigest {
    pub fn new(compression: f64, max_size: usize) -> TDigest {
        TDigest {
            compression,
            max_size,
            min: 0.0,
            max: 0.0,
            centroids: Vec::new(),
            total_weight: 0.0,
            buffer: Vec::new(),
        }
    }

    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        self.total_weight += self.buffer.len() as f64;

        let mut tmp = Vec::with_capacity(self.centroids.len() + self.buffer.len());
        tmp.append(&mut self.centroids);
        tmp.append(&mut self.buffer);
        tmp.sort_by(|a, b| a.mean.partial_cmp(&b.mean).unwrap());

        let normalizer = self.compression / (consts::PI * self.total_weight);

        let mut weight_so_far = 0.0;
        let mut acc = tmp[0].clone();
        for c in tmp.into_iter().skip(1) {
            let proposed_weight = acc.weight + c.weight;
            let z = proposed_weight * normalizer;
            let q0 = weight_so_far / self.total_weight;
            let q2 = (weight_so_far + proposed_weight) / self.total_weight;

            if z * z <= q0 * (1.0 - q0) && z * z <= q2 * (1.0 - q2) {
                acc.add(&c)
            } else {
                // didn't fit ... move to next output, copy out first centroid
                weight_so_far += acc.weight;
                self.centroids.push(acc);
                acc = c;
            }
        }
        self.centroids.push(acc);
        self.min = self.centroids.first().unwrap().mean;
        self.max = self.centroids.last().unwrap().mean;
    }
}

impl ::Estimator for TDigest {
    fn add(&mut self, x: f64) {
        self.buffer.push(Centroid::single(x));
        if self.buffer.len() >= self.max_size {
            self.flush_buffer();
        }
    }

    fn estimate(&mut self, q: f64) -> f64 {
        self.flush_buffer();
        if self.centroids.is_empty() {
            return ::std::f64::NAN;
        }
        if self.centroids.len() == 1 {
            return self.centroids[0].mean;
        }
        if q <= 0.0 {
            return self.min;
        }
        if q >= 1.0 {
            return self.max;
        }

        // we know that there are at least two centroids now
        let n = self.centroids.len();

        // if values were stored in a sorted array, index would be the offset we are interested in
        let index = q * self.total_weight;

        // at the boundaries, we return min or max
        if index < self.centroids[0].weight / 2.0 {
            return self.min
                + 2.0 * index / self.centroids[0].weight * (self.centroids[0].mean - self.min);
        }

        // in between we interpolate between centroids
        let mut weight_so_far = self.centroids[0].weight / 2.0;
        for i in 0..n - 1 {
            let dw = (self.centroids[i].weight + self.centroids[i + 1].weight) / 2.0;
            if weight_so_far + dw > index {
                // centroids i and i+1 bracket our current point
                let z1 = index - weight_so_far;
                let z2 = weight_so_far + dw - index;
                return weighted_average(self.centroids[i].mean, z2, self.centroids[i + 1].mean, z1);
            }
            weight_so_far += dw;
        }

        let z1 = index - self.total_weight - self.centroids[n - 1].weight / 2.0;
        let z2 = self.centroids[n - 1].weight / 2.0 - z1;
        weighted_average(self.centroids[n - 1].mean, z1, self.max, z2)
    }
}

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    fn run(size: usize) -> f64 {
        let mut estimator = TDigest::new(100.0, 500);
        let mut n = 1;
        for _ in 0..size {
            estimator.add(n as f64 / size as f64);
            n = (19 * n) % size;
        }
        estimator.estimate(0.99)
    }

    #[bench]
    fn small(b: &mut Bencher) {
        b.iter(|| run(100));
    }

    #[bench]
    fn medium(b: &mut Bencher) {
        b.iter(|| run(10_000));
    }

    #[bench]
    fn big(b: &mut Bencher) {
        b.iter(|| run(1_000_000));
    }
}
