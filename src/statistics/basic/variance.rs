use num_traits::{Float, FromPrimitive};
use crate::statistics::*;

#[derive(Debug, Clone, Copy)]
pub struct Variance {
    pub ddof: usize,
}

impl Variance {
    /// Creates a new `Variance` estimator with the given degrees of freedom adjustment.
    ///
    /// - `ddof = 0`: population variance (biased)
    /// - `ddof = 1`: sample variance (unbiased, Bessel's correction) — this is the default
    pub fn new(ddof: usize) -> Self {
        Variance { ddof }
    }
}

impl Default for Variance {
    /// Returns a `Variance` estimator with `ddof = 1` (unbiased sample variance).
    fn default() -> Self {
        Variance { ddof: 1 }
    }
}

impl<D, T> Statistic<D, T> for Variance
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();

        // Variance undefined for n < 2
        if slice.len() < 2 {
            return T::nan();
        }

        let mean = Mean.compute(data);

        // Kahan summation for squared deviations
        let mut sq_sum = T::zero();
        let mut c2 = T::zero();
        for &x in slice {
            let dev = x - mean;
            let y = dev * dev - c2;
            let t = sq_sum + y;
            c2 = (t - sq_sum) - y;
            sq_sum = t;
        }

        let dof = T::from_usize(slice.len() - self.ddof).expect("usize fits in float");
        sq_sum / dof
    }
}
