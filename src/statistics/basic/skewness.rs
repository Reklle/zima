use num_traits::{Float, FromPrimitive};
use crate::statistics::*;

/// Sample skewness (third standardized moment).
///
/// γ₁ = κ̂₃ / κ̂₂^{3/2}
///
/// - Biased version uses raw central moments
/// - Unbiased version uses bias-corrected cumulants (default)
///
/// Requires n ≥ 3 for unbiased estimation.
#[derive(Debug, Clone, Copy)]
pub struct Skewness {
    pub unbiased: bool,
}

impl Skewness {
    pub fn new(unbiased: bool) -> Self {
        Skewness { unbiased }
    }

    pub fn unbiased() -> Self {
        Skewness { unbiased: true }
    }
}

impl Default for Skewness {
    fn default() -> Self {
        Skewness { unbiased: true } // Matches scipy.stats.skew(bias=False)
    }
}

impl<D, T> Statistic<D, T> for Skewness
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();
        let n = slice.len();
        if n < 3 && self.unbiased {
            return T::nan();
        }
        if n < 2 {
            return T::nan();
        }

        let n_f = T::from_usize(n).expect("n fits in float");
        let mean = Mean.compute(data);

        // Single-pass Kahan summation for m2 and m3
        let mut sum2 = T::zero();
        let mut sum3 = T::zero();
        let mut c2 = T::zero();
        let mut c3 = T::zero();

        for &x in slice {
            let dev = x - mean;
            let dev2 = dev * dev;
            let dev3 = dev2 * dev;

            // Kahan for m2
            let y2 = dev2 - c2;
            let t2 = sum2 + y2;
            c2 = (t2 - sum2) - y2;
            sum2 = t2;

            // Kahan for m3
            let y3 = dev3 - c3;
            let t3 = sum3 + y3;
            c3 = (t3 - sum3) - y3;
            sum3 = t3;
        }

        let m2 = sum2 / n_f; // Biased variance (κ₂ when ddof=0)
        let m3 = sum3 / n_f; // Biased third central moment

        if self.unbiased {
            // Unbiased κ̂₂ = n/(n-1) * m2   [same as Variance{ddof=1}]
            // Unbiased κ̂₃ = [n² / ((n-1)(n-2))] * m3
            let n1 = n_f - T::one();
            let n2 = n_f - T::from_u8(2).unwrap();

            if n1 == T::zero() || n2 == T::zero() {
                return T::nan();
            }

            let k2 = (n_f / n1) * m2;           // Unbiased variance
            let k3 = (n_f * n_f) / (n1 * n2) * m3; // Unbiased third cumulant

            let denom = k2.sqrt().powi(3);
            if denom == T::zero() {
                T::nan()
            } else {
                k3 / denom
            }
        } else {
            // Biased: γ₁ = m3 / m2^{3/2}
            let denom = m2.sqrt().powi(3);
            if denom == T::zero() {
                T::nan()
            } else {
                m3 / denom
            }
        }
    }
}
