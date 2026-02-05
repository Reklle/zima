use num_traits::{Float, FromPrimitive};
use rand::thread_rng;
use statrs::distribution::{Normal, ContinuousCDF};

use crate::{CDF, Flip, Flipper, Mean, Re, Sample, SignBitFlip, Statistic, Variance};

/// Permutation test for the hypothesis about the population variance.
///
/// Tests the null hypothesis: `H₀: σ² = σ₀²` without assuming normality of the distribution.
/// Uses the sign-flipping method on centered data (centered by sample mean).
///
/// # Statistical rationale
/// 1. Center data by sample mean: `Yᵢ = Xᵢ - X̄` to eliminate unknown μ
/// 2. Under H₀, signs of centered observations are exchangeable
/// 3. Random sign flipping preserves the joint distribution under H₀
/// 4. Compare observed variance deviation `|s² - σ₀²|` with permuted distribution
///
/// # Statistical assumptions
/// - **Assumes**: i.i.d. sample, existence of population variance
/// - **Does not require**: normality, symmetry, or known mean μ
/// - **Test type**: two-sided (tests `σ² ≠ σ₀²`)
///
/// # Example
/// ```rust
/// use your_crate::VarianceTest;
///
/// let data = vec![0.5, -1.2, 0.8, 1.5, -0.3];
/// let test = VarianceTest::<f64>::unit(0.01); // test σ² = 1 with ±0.01 accuracy
/// let result = test.compute(&data);
///
/// println!("Observed variance: {:.4}", result.observed_statistic);
/// println!("p-value: {:.4}", result.p_value);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct VarianceTest<F> {
    pub null_variance: F,
    pub n_permutations: usize,
}

/// Result of the permutation test for variance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestResult<F: Float> {
    /// Observed test statistic: sample variance (unbiased estimator).
    pub observed_statistic: F,
    /// Estimated p-value with continuity correction.
    pub p_value: F,
}

impl<D, F> Statistic<D, TestResult<F>> for VarianceTest<F>
where
    D: AsRef<[F]> + Clone,
    F: Float + FromPrimitive + Copy,
    SignBitFlip: Flip<F>,
{
    fn compute(&self, data: &D) -> TestResult<F> {
        let data_slice = data.as_ref();
        let n = data_slice.len();

        if n < 2 {
            // Variance undefined for n < 2
            return TestResult {
                p_value: F::from(1.0).expect("1.0 is a valid float"),
                observed_statistic: F::nan(),
            };
        }

        let sample_mean = Mean.compute(&data);
        let centered: Sample<F> = data_slice
            .iter()
            .map(|&x| x - sample_mean)
            .collect();

        // Step 2: Compute observed unbiased variance
        let observed_var = Variance::default().compute(&centered); // Σ(x_i - x̄)² / (n-1)
        let observed_deviation = (observed_var - self.null_variance).abs();

        // Step 3: Generate permuted statistics via sign flipping
        // Sign flipping preserves variance distribution under H₀
        let flipper = Flipper::sign(thread_rng());
        let permuted_deviations: Sample<F> = flipper
            .re(&centered)
            .map(|resample| {
                let var = Variance::default().compute(&resample);
                (var - self.null_variance).abs()
            })
            .take(self.n_permutations)
            .collect();

        // Step 4: Count extreme deviations under null distribution
        let extreme_count = permuted_deviations
            .as_ref()
            .iter()
            .filter(|&&dev| dev >= observed_deviation)
            .count();

        // Continuity correction for discrete permutation distribution
        let p_value = F::from(extreme_count + 1).expect("extreme_count + 1 fits in float")
            / F::from(self.n_permutations + 1).expect("n_permutations + 1 fits in float");

        TestResult {
            p_value,
            observed_statistic: observed_var,
        }
    }
}

impl<F: Float + FromPrimitive> VarianceTest<F> {
    /// Creates a test with an explicitly specified number of permutations.
    ///
    /// # Arguments
    /// * `null_variance` — the hypothesized variance σ₀² under H₀ (e.g., 1.0 for σ = 1)
    /// * `n_permutations` — number of resamples to approximate the null distribution
    ///
    /// # Panics
    /// Panics if `n_permutations == 0`.
    pub fn new(null_variance: F, n_permutations: usize) -> Self {
        assert!(
            n_permutations > 0,
            "n_permutations must be positive"
        );
        Self {
            null_variance,
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
    /// 2. For small samples (n < 15), the exact permutation distribution has only
    ///    2ⁿ unique values — consider exhaustive enumeration instead.
    /// 3. The guarantee is conservative: for p-values far from 0.5, the actual
    ///    accuracy will be substantially better than requested.
    ///
    /// # Formula
    /// Uses the conservative sample size formula for binomial proportion estimation:
    /// ```text
    /// n_permutations = ceil( (z_{1−α/2}² · 0.25) / accuracy² )
    /// ```
    /// where 0.25 is the maximum variance of a Bernoulli variable (at p = 0.5).
    ///
    /// # Arguments
    /// * `null_variance` — hypothesized variance under H₀
    /// * `accuracy` — half-width of the target confidence interval (e.g., 0.01)
    /// * `confidence_level` — confidence level for the interval (e.g., 0.95)
    ///
    /// # Panics
    /// Panics if `accuracy ∉ (0, 0.5)` or `confidence_level ∉ (0.5, 1.0)`.
    pub fn from_absolute_accuracy(
        null_variance: F,
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
            null_variance,
            n_permutations,
        }
    }

    /// Tests the hypothesis `H₀: σ² = 1` with a specified absolute accuracy.
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
    /// let test = VarianceTest::<f64>::unit(0.01); // test σ² = 1 with ±1% accuracy
    /// ```
    pub fn unit(accuracy: f64) -> Self
    where
        F: FromPrimitive,
    {
        Self::from_absolute_accuracy(F::from(1.0).expect("1.0 is valid"), accuracy, 0.95)
    }
}
