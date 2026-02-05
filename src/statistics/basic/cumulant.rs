use num_traits::{Float, FromPrimitive};
use crate::statistics::*;

/// Third cumulant (κ₃ = μ₃) — numerator for skewness computation.
///
/// # Bias correction
/// - Biased estimator:   m₃ = (1/n) Σ(xᵢ - x̄)³
/// - Unbiased estimator: κ̂₃ = [n² / ((n-1)(n-2))] · m₃
///
/// Requires n ≥ 3 for unbiased estimation.
#[derive(Debug, Clone, Copy)]
pub struct ThirdCumulant {
    pub unbiased: bool,
}

impl ThirdCumulant {
    pub fn new(unbiased: bool) -> Self {
        ThirdCumulant { unbiased }
    }

    pub fn unbiased() -> Self {
        ThirdCumulant { unbiased: true }
    }
}

impl Default for ThirdCumulant {
    fn default() -> Self {
        ThirdCumulant { unbiased: true } // Unbiased is standard for inference
    }


}

impl<D, T> Statistic<D, T> for ThirdCumulant
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();
        let n = slice.len();
        let n_f = T::from_usize(n).expect("n fits in float");

        if n < 3 && self.unbiased {
            return T::nan();
        }
        if n < 2 {
            return T::nan();
        }

        let mean = Mean.compute(data);

        let mut sum3 = T::zero();
        let mut c3 = T::zero();

        for &x in slice {
            let dev = x - mean;
            let dev3 = dev * dev * dev;

            let y = dev3 - c3;
            let t = sum3 + y;
            c3 = (t - sum3) - y;
            sum3 = t;
        }

        let m3 = sum3 / n_f;

        if self.unbiased {
            let n1 = n_f - T::one();
            let n2 = n_f - T::from_u8(2).unwrap();
            if n1 == T::zero() || n2 == T::zero() {
                return T::nan();
            }
            let correction = (n_f * n_f) / (n1 * n2);
            m3 * correction
        } else {
            m3
        }
    }
}

/// Fourth cumulant (κ₄ = μ₄ - 3μ₂²) — numerator for excess kurtosis.
///
/// # Bias correction
/// - Biased estimator:   κ₄ = m₄ - 3m₂²
/// - Unbiased estimator: κ̂₄ = [n² / ((n-1)(n-2)(n-3))] · [(n+1)m₄ - 3(n-1)m₂²]
///
/// Requires n ≥ 4 for unbiased estimation.
#[derive(Debug, Clone, Copy)]
pub struct FourthCumulant {
    pub unbiased: bool,
}

impl FourthCumulant {
    pub fn new(unbiased: bool) -> Self {
        FourthCumulant { unbiased }
    }

    pub fn unbiased() -> Self {
        FourthCumulant { unbiased: true }
    }
}

impl Default for FourthCumulant {
    fn default() -> Self {
        FourthCumulant { unbiased: true } // Unbiased is standard for inference
    }
}

impl<D, T> Statistic<D, T> for FourthCumulant
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();
        let n = slice.len();
        let n_f = T::from_usize(n).expect("n fits in float");

        if n < 4 && self.unbiased {
            return T::nan();
        }
        if n < 2 {
            return T::nan();
        }

        let mean = Mean.compute(data);

        // Single-pass Kahan summation for m₂, m₃, m₄
        let mut sum2 = T::zero();
        let mut sum4 = T::zero();
        let mut c2 = T::zero();
        let mut c4 = T::zero();

        for &x in slice {
            let dev = x - mean;
            let dev2 = dev * dev;
            let dev4 = dev2 * dev2;

            // Kahan for squared deviations (m₂)
            let y2 = dev2 - c2;
            let t2 = sum2 + y2;
            c2 = (t2 - sum2) - y2;
            sum2 = t2;

            // Kahan for fourth-power deviations (m₄)
            let y4 = dev4 - c4;
            let t4 = sum4 + y4;
            c4 = (t4 - sum4) - y4;
            sum4 = t4;
        }

        let m2 = sum2 / n_f; // Biased second central moment
        let m4 = sum4 / n_f; // Biased fourth central moment

        if self.unbiased {
            // Unbiased κ₄ = [n² / ((n-1)(n-2)(n-3))] · [(n+1)m₄ - 3(n-1)m₂²]
            let n1 = n_f - T::one();
            let n2 = n_f - T::from_u8(2).unwrap();
            let n3 = n_f - T::from_u8(3).unwrap();
            if n1 == T::zero() || n2 == T::zero() || n3 == T::zero() {
                return T::nan();
            }

            let numerator = (n_f + T::one()) * m4 - (T::from_u8(3).unwrap() * n1) * (m2 * m2);
            let denominator = n1 * n2 * n3;
            let correction = (n_f * n_f) / denominator;
            numerator * correction
        } else {
            // Biased κ₄ = m₄ - 3m₂²
            m4 - (T::from_u8(3).unwrap() * m2 * m2)
        }
    }
}
