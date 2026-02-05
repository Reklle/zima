use num_traits::{Float, FromPrimitive};
use crate::{Jackknife, Re, Variance};
use super::Statistic;


/// Standard Error of the Mean (SEM).
///
/// Computes the standard error of the sample mean:
/// ```text
/// SE = sqrt( variance / n )
/// ```
/// where `variance` is computed using the configured `Variance` estimator
/// (e.g., sample variance with Bessel's correction by default).
#[derive(Debug, Clone, Copy, Default)]
pub struct SEMean {
    variance: Variance,
}

impl SEMean {
    /// Creates a new `SEMean` with a custom variance estimator.
    pub fn with_variance(variance: Variance) -> Self {
        Self { variance }
    }
}

/// TODO: some problems with slice+data simultaniusly using
impl<D, T> Statistic<D, T> for SEMean
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive,
{
    fn compute(&self, data: &D) -> T {
        let slice = data.as_ref();

        // SE undefined for empty samples
        if slice.is_empty() {
            return T::nan();
        }

        // Compute variance estimate (e.g., s² = Σ(xᵢ - x̄)² / (n - ddof))
        let var_est = self.variance.compute(data);

        // Propagate NaN from variance (e.g., n < 2 for sample variance with ddof=1)
        if var_est.is_nan() {
            return T::nan();
        }

        // Convert sample size to float
        let n = T::from_usize(slice.len()).expect("usize-to-float conversion failed");

        // Compute SE² = variance_estimate / n
        let se_sq = var_est / n;

        // Guard against negative values due to floating-point errors
        if se_sq < T::zero() {
            // Clamp near-zero negatives to zero (within 10× machine epsilon)
            let tolerance = T::epsilon() * T::from_f64(10.0).unwrap_or(T::one());
            if se_sq >= -tolerance {
                T::zero()
            } else {
                // Significantly negative → invalid variance estimate
                T::nan()
            }
        } else {
            se_sq.sqrt()
        }
    }
}

/// Bootstrap/Jackknife standard error estimator.
///
/// Computes the standard error of a statistic via resampling:
/// ```text
/// SE_boot(θ̂) = std({ θ̂*(b) | b = 1..B })
/// ```
/// where θ̂*(b) is the statistic computed on the b-th resample.
#[derive(Debug, Clone, Copy)]
pub struct SE<Stat, Resampler> {
    statistic: Stat,
    resampler: Resampler,
    samples: usize,
}

impl<Stat> SE<Stat, Jackknife> {
    pub fn jackknife(statistic: Stat) -> Self {
        Self {
            statistic,
            resampler: Jackknife,
            samples: usize::MAX,
        }
    }
}

impl<Stat, Resampler> SE<Stat, Resampler> {
    /// Creates a new resampling-based SE estimator.
    ///
    /// # Parameters
    /// - `statistic`: The base statistic to estimate (e.g., `Mean`)
    /// - `resampler`: Resampling strategy (`Bootstrap`, `Jackknife`, etc.)
    /// - `samples`: Number of resamples (B). Use `usize::MAX` for full jackknife.
    pub fn new(statistic: Stat, resampler: Resampler, samples: usize) -> Self {
        Self {
            statistic,
            resampler,
            samples,
        }
    }
}

impl<D, T, Stat, Resampler> Statistic<D, T> for SE<Stat, Resampler>
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive,
    Resampler: Re<D, Item = D>,
    Stat: Statistic<D, T>,
{
    fn compute(&self, data: &D) -> T {
        let estimates: Vec<T> = self
            .resampler
            .re(data)
            .take(self.samples)
            .map(|resample| self.statistic.compute(&resample))
            .collect();

        Variance::default()
            .compute(&estimates)
            .sqrt()
    }
}
