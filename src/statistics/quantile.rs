use super::Statistic;
use crate::EmpiricalCDF;

/// Discrete quantile estimator (inverse ECDF).
///
/// Computes the smallest value `x` where Fₙ(x) ≥ p using the definition:
/// ```text
/// Q(p) = inf { x : Fₙ(x) ≥ p }
/// ```
/// This corresponds to R's type=1 quantiles and requires no interpolation.
///
/// # Performance
/// - O(1) evaluation (single array index after ECDF construction)
/// - Branchless index calculation
/// - Zero allocations
/// - Works with ANY `Ord` type (integers, strings, timestamps, etc.)
#[derive(Debug, Clone, Copy)]
pub struct Quantile {
    p: f64,
}

impl Quantile {
    /// Creates a quantile estimator for probability `p ∈ [0, 1]`.
    #[inline]
    pub fn new(p: f64) -> Self {
        debug_assert!((0.0..=1.0).contains(&p), "Quantile p must be in [0,1]");
        Self { p }
    }

    /// Convenience constructor for median (p = 0.5).
    #[inline]
    pub fn median() -> Self {
        Self { p: 0.5 }
    }
}

impl<T: Clone> Statistic<EmpiricalCDF<T>, T> for Quantile {
    #[inline]
    fn compute(&self, ecdf: &EmpiricalCDF<T>) -> T {
        let n = ecdf.n();
        // Panic on empty ECDF (consistent with statistical convention)
        // Users should validate non-empty samples before quantile computation
        assert!(n > 0, "Quantile undefined for empty distribution");

        // Branchless index calculation:
        // i = ceil(n * p) - 1, clamped to [0, n-1]
        // For p=0.0: ceil(0)=0 → 0-1 underflows → saturating_sub gives 0 ✓
        // For p=1.0: ceil(n)=n → n-1 ✓
        let idx = ((n as f64 * self.p).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1);

        // Safe: idx guaranteed in [0, n-1] by clamping above
        ecdf.points()[idx].clone()
    }
}

/// Quantile interval estimator (e.g., IQR, 95% interval).
///
/// Returns tuple `(Q(lower), Q(upper))` where Q is the discrete quantile.
#[derive(Debug, Clone, Copy)]
pub struct QuantileInterval {
    lower: f64,
    upper: f64,
}

impl QuantileInterval {
    /// Creates interval estimator for `[lower, upper]` probabilities.
    #[inline]
    pub fn new(lower: f64, upper: f64) -> Self {
        debug_assert!((0.0..=1.0).contains(&lower));
        debug_assert!((0.0..=1.0).contains(&upper));
        debug_assert!(lower <= upper);
        Self { lower, upper }
    }

    #[inline]
    pub fn percentile(confidence: f64) -> Self {
        let alpha = 1.0 - confidence;
        Self::new(alpha / 2.0, 1.0 - alpha / 2.0)
    }
}

impl<T: Clone> Statistic<EmpiricalCDF<T>, (T, T)> for QuantileInterval {
    #[inline]
    fn compute(&self, ecdf: &EmpiricalCDF<T>) -> (T, T) {
        let n = ecdf.n();
        assert!(n > 0, "Quantile interval undefined for empty distribution");

        // Compute lower index (branchless)
        let idx_low = ((n as f64 * self.lower).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1);

        // Compute upper index (branchless)
        let idx_up = ((n as f64 * self.upper).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1);

        // Safe: indices clamped to valid range
        let points = ecdf.points();
        (points[idx_low].clone(), points[idx_up].clone())
    }
}
