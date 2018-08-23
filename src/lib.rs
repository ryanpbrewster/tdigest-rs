pub mod simple;

pub trait Estimator {
    fn add(&mut self, x: f64);
    fn estimate(&mut self, q: f64) -> f64;
}

pub struct Oracle {
    values: Vec<f64>,
}
impl Oracle {
    pub fn new(mut values: Vec<f64>) -> Oracle {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Oracle { values }
    }

    pub fn quantile(&self, q: f64) -> f64 {
        if q <= 0.0 {
            return *self.values.first().unwrap();
        }
        if q >= 1.0 {
            return *self.values.last().unwrap();
        }
        self.values[(q * (self.values.len() - 1) as f64) as usize]
    }

    /** Given a value, `x`, find which quantile that value belongs to. */
    pub fn rank(&self, x: f64) -> f64 {
        let idx = match self.values.binary_search_by(|y| y.partial_cmp(&x).unwrap()) {
            Ok(present) => present,
            Err(missing) => missing,
        };
        idx as f64 / (self.values.len() - 1) as f64
    }
}
