use num_traits::{Float, FromPrimitive};
use crate::statistics::*;

/// Sample excess kurtosis (fourth standardized cumulant).
///
/// γ₂ = κ̂₄ / κ̂₂²
///
/// - Reports *excess* kurtosis (0 for normal distribution)
/// - Unbiased version uses bias-corrected cumulants (default)
///
/// Requires n ≥ 4 for unbiased estimation.
#[derive(Debug, Clone, Copy)]
pub struct Kurtosis {
    pub unbiased: bool,
}

impl Kurtosis {
    pub fn new(unbiased: bool) -> Self {
        Kurtosis { unbiased }
    }

    pub fn unbiased() -> Self {
        Kurtosis { unbiased: true }
    }
}

impl Default for Kurtosis {
    fn default() -> Self {
        Kurtosis { unbiased: true } // Matches scipy.stats.kurtosis(fisher=True, bias=False)
    }
}

impl<D, T> Statistic<D, T> for Kurtosis
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();
        let n = slice.len();
        if n < 4 && self.unbiased {
            return T::nan();
        }
        if n < 2 {
            return T::nan();
        }

        let n_f = T::from_usize(n).expect("n fits in float");
        let mean = Mean.compute(data);

        // Single-pass Kahan summation for m2 and m4
        let mut sum2 = T::zero();
        let mut sum4 = T::zero();
        let mut c2 = T::zero();
        let mut c4 = T::zero();

        for &x in slice {
            let dev = x - mean;
            let dev2 = dev * dev;
            let dev4 = dev2 * dev2;

            // Kahan for m2
            let y2 = dev2 - c2;
            let t2 = sum2 + y2;
            c2 = (t2 - sum2) - y2;
            sum2 = t2;

            // Kahan for m4
            let y4 = dev4 - c4;
            let t4 = sum4 + y4;
            c4 = (t4 - sum4) - y4;
            sum4 = t4;
        }

        let m2 = sum2 / n_f;
        let m4 = sum4 / n_f;

        if self.unbiased {
            // Unbiased κ̂₂ = n/(n-1) * m2
            // Unbiased κ̂₄ = [n²(n+1)m4 - 3n(n-1)m2²] / [(n-1)(n-2)(n-3)]
            let n1 = n_f - T::one();
            let n2 = n_f - T::from_u8(2).unwrap();
            let n3 = n_f - T::from_u8(3).unwrap();

            if n1 == T::zero() || n2 == T::zero() || n3 == T::zero() {
                return T::nan();
            }

            let k2 = (n_f / n1) * m2;
            let numerator = (n_f * n_f * (n_f + T::one())) * m4
                          - (T::from_u8(3).unwrap() * n_f * n1) * (m2 * m2);
            let denominator = n1 * n2 * n3;
            let k4 = numerator / denominator;

            let denom = k2 * k2;
            if denom == T::zero() {
                T::nan()
            } else {
                k4 / denom
            }
        } else {
            // Biased excess kurtosis: (m4 / m2²) - 3
            let ratio = m4 / (m2 * m2);
            ratio - T::from_u8(3).unwrap()
        }
    }
}
