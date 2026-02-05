use num_traits::{Float, FromPrimitive};
use crate::{Jackknife, SE};

use super::Statistic;

/// Studentized statistic for hypothesis testing: t = (θ̂ - θ₀) / SE(θ̂)
///
/// Computes the standardized distance between an estimate and a null hypothesis value,
/// scaled by the estimate's standard error. Used for t-tests and Wald tests.
///
/// # Mathematical definition
/// ```text
/// t = (estimate - null_value) / standard_error
/// ```
/// where:
/// - `estimate` = θ̂ = statistic computed on data
/// - `null_value` = θ₀ = hypothesized parameter value under H₀
/// - `standard_error` = SE(θ̂) = estimated standard deviation of θ̂
///
/// # Properties
/// - Returns `NaN` when SE = 0 or inputs are invalid (consistent with IEEE 754)
/// - Works with any float type implementing `num_traits::Float`
/// - Null value stored as generic `T` (no f64 conversions needed)
#[derive(Debug, Clone, Copy)]
pub struct Studentized<Estimator, SEE, T> {
    pub statistic: Estimator,
    pub se: SEE,
    pub null_value: T,
}

impl<Estimator, SEE, T> Studentized<Estimator, SEE, T> {
    /// Creates a studentized statistic (t-statistic) for hypothesis testing.
    pub fn new(statistic: Estimator, se: SEE, null_value: T) -> Self {
        Self {
            statistic,
            se,
            null_value,
        }
    }
}

impl<D, T, Estimator, SEE> Statistic<D, T> for Studentized<Estimator, SEE, T>
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive + Copy,
    Estimator: Statistic<D, T>,
    SEE: Statistic<D, T>,
{
    #[inline]
    fn compute(&self, data: &D) -> T {
        let estimate = self.statistic.compute(data);
        let se_val = self.se.compute(data);

        if se_val.is_zero() || estimate.is_nan() || se_val.is_nan() {
            T::nan()
        } else {
            (estimate - self.null_value) / se_val
        }
    }
}

// impl<Estimator, T> Studentized<Estimator, SE<Estimator, Jackknife>, T>
// where
//     T: Float + Copy,
// {
//     pub fn new(statistic: Estimator, null_value: T) -> Self {
//         Self {
//             statistic,
//             se: SE::jackknife(statistic),
//             null_value,
//         }
//     }
// }
