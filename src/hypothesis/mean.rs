use num_traits::{Float, FromPrimitive};
use rand::thread_rng;
use statrs::distribution::{Normal, ContinuousCDF};

use crate::{CDF, Flip, Flipper, Mean, Re, Sample, SignBitFlip, Statistic};

/// Permutation test for the hypothesis about the population mean.
///
/// Tests the null hypothesis: `H₀: μ = μ₀` without assuming normality of the distribution.
/// Uses the sign-flipping (random sign inversion) method on centered data.
///
/// # Statistical assumptions
/// - **Assumes**: i.i.d. sample, existence of the population mean
/// - **Does not require**: normality, symmetry, or finite variance
/// - **Test type**: two-sided (tests `μ ≠ μ₀`)
///
/// # Example
/// ```rust
/// use your_crate::MeanTest;
///
/// let data = vec![0.5, -1.2, 0.8, 1.5, -0.3];
/// let test = MeanTest::<f64>::zero(0.01); // accuracy ±0.01
/// let result = test.compute(&data);
///
/// println!("p-value: {:.4}", result.p_value);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MeanTest<F> {
    pub null_mean: F,
    pub n_permutations: usize,
}

/// Result of the permutation test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestResult<F: Float> {
    /// Observed test statistic (mean of centered data).
    pub observed_statistic: F,
    /// Estimated p-value with continuity correction.
    pub p_value: F,
}

impl<D, F> Statistic<D, TestResult<F>> for MeanTest<F>
where
    D: AsRef<[F]> + Clone,
    F: Float + FromPrimitive + Copy,
    SignBitFlip: Flip<F>,
{
    fn compute(&self, data: &D) -> TestResult<F> {
        let data_slice = data.as_ref();
        let n = data_slice.len();

        if n == 0 {
            return TestResult {
                p_value: F::from(1.0).expect("1.0 is a valid float"),
                observed_statistic: F::zero(),
            };
        }

        let centered: Sample<F> = data_slice
            .iter()
            .map(|&x| x - self.null_mean)
            .collect();

        let observed_stat = Mean.compute(&centered);
        let observed_abs = observed_stat.abs();

        let flipper = Flipper::sign(thread_rng());
        let permuted_stats: Sample<F> = flipper
            .re(&centered)
            .map(|resample| Mean.compute(&resample))
            .take(self.n_permutations)
            .collect();

        let extreme_count = permuted_stats
            .as_ref()
            .iter()
            .filter(|&&stat| stat.abs() >= observed_abs)
            .count();

        let p_value = F::from(extreme_count + 1).expect("extreme_count + 1 fits in float")
            / F::from(self.n_permutations + 1).expect("n_permutations + 1 fits in float");

        TestResult {
            p_value,
            observed_statistic: observed_stat,
        }
    }
}

impl<F: Float + FromPrimitive> MeanTest<F> {
    /// Creates a test with an explicitly specified number of permutations.
    ///
    /// # Arguments
    /// * `null_mean` — the hypothesized mean μ₀ under H₀
    /// * `n_permutations` — number of resamples to approximate the null distribution
    ///
    /// # Panics
    /// Panics if `n_permutations == 0`.
    pub fn new(null_mean: F, n_permutations: usize) -> Self {
        assert!(n_permutations > 0, "n_permutations must be positive");
        Self {
            null_mean,
            n_permutations,
        }
    }

    /// Creates a test with a desired absolute accuracy for the p-value estimate.
    ///
    /// # Statistical guarantee
    /// Guarantees that the **width of the (1−α) confidence interval** for the
    /// estimated p-value does not exceed `2·accuracy` in the worst case
    /// (when the true p-value ≈ 0.5), assuming:
    /// - The permutation distribution is well-approximated by independent sampling
    /// - The normal approximation to the binomial distribution is adequate
    ///   (typically satisfied when n_permutations > 30)
    ///
    /// # Important caveats
    /// 1. This controls **sampling error** from approximating the permutation
    ///    distribution, NOT the inherent discreteness of the exact permutation test.
    ///    The exact test has only 2ⁿ unique sign-flipping configurations.
    /// 2. For small samples (n < 15), the discreteness dominates — consider
    ///    exhaustive enumeration of all 2ⁿ permutations instead of sampling.
    /// 3. The guarantee is conservative: actual accuracy is substantially better
    ///    when the true p-value is far from 0.5 (e.g., p < 0.1 or p > 0.9).
    /// 4. "Accuracy ±0.01" refers to estimation error of p-value, NOT error in
    ///    hypothesis decision (Type I/II error rates remain unaffected).
    ///
    /// # Formula
    /// Uses the conservative sample size formula for binomial proportion estimation:
    /// ```text
    /// n_permutations = ceil( (z_{1−α/2}² · 0.25) / accuracy² )
    /// ```
    /// where 0.25 is the maximum variance of a Bernoulli variable (at p = 0.5).
    ///
    /// # Arguments
    /// * `null_mean` — hypothesized mean under H₀
    /// * `accuracy` — half-width of the target confidence interval (e.g., 0.01)
    /// * `confidence_level` — confidence level for the interval (e.g., 0.95)
    ///
    /// # Panics
    /// Panics if `accuracy ∉ (0, 0.5)` or `confidence_level ∉ (0.5, 1.0)`.
    pub fn from_absolute_accuracy(
        null_mean: F,
        accuracy: f64,
        confidence_level: f64,
    ) -> Self {
        assert!(
            accuracy > 0.0 && accuracy < 0.5,
            "accuracy must be in (0, 0.5), got {}",
            accuracy
        );
        assert!(
            confidence_level > 0.5 && confidence_level < 1.0,
            "confidence_level must be in (0.5, 1.0), got {}",
            confidence_level
        );

        // Get z-quantile of the standard normal distribution
        let alpha = 1.0 - confidence_level;
        let z = Normal::new(0.0, 1.0)
            .expect("Valid N(0,1) distribution")
            .inverse_cdf(1.0 - alpha / 2.0);

        // Conservative estimate of the minimum number of permutations
        let n_min = (z * z * 0.25) / (accuracy * accuracy);
        let n_permutations = n_min.ceil() as usize;

        // Practical bounds for safety and performance
        let n_permutations = n_permutations.clamp(100, 10_000_000);

        Self {
            null_mean,
            n_permutations,
        }
    }

    /// Tests the hypothesis `H₀: μ = 0` with a specified absolute accuracy.
    ///
    /// Uses the standard 95% confidence level.
    ///
    /// # Arguments
    /// * `accuracy` — absolute error tolerance for the p-value estimate
    ///
    /// # Example accuracy levels
    /// - `accuracy = 0.02` → 2,401 permutations (exploratory analysis)
    /// - `accuracy = 0.01` → 9,604 permutations (standard testing)
    /// - `accuracy = 0.005` → 38,416 permutations (publication-ready)
    /// - `accuracy = 0.001` → 960,385 permutations (critical decisions)
    ///
    /// # Example
    /// ```rust
    /// let test = MeanTest::<f64>::zero(0.01); // ±1% accuracy
    /// ```
    pub fn zero(accuracy: f64) -> Self
    where
        F: FromPrimitive,
    {
        Self::from_absolute_accuracy(F::zero(), accuracy, 0.95)
    }
}
