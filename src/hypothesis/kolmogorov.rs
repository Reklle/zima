use num_traits::{Float, FromPrimitive, ToPrimitive};
use rand::thread_rng;
use statrs::distribution::{Normal, ContinuousCDF};

use crate::{CDF, Statistic, TestResult}; // Assuming TestResult is defined in crate root

/// Kolmogorov-Smirnov goodness-of-fit test against the standard normal distribution.
///
/// Tests the null hypothesis: `H₀: X ~ N(0, 1)`.
///
/// # Statistical background
/// The test statistic is the supremum distance between the empirical CDF Fₙ(x)
/// and the theoretical CDF Φ(x) of the standard normal distribution:
/// ```text
/// D = supₓ |Fₙ(x) - Φ(x)|
/// ```
///
/// # Statistical assumptions
/// - **Assumes**: i.i.d. sample, continuous distribution under H₀
/// - **Does not require**: estimation of parameters (μ = 0, σ = 1 are fixed)
/// - **Test type**: two-sided (any deviation from N(0,1))
#[derive(Debug, Clone, Copy)]
pub struct KSTest;

impl<D, F> Statistic<D, TestResult<F>> for KSTest
where
    D: AsRef<[F]> + Clone,
    F: Float + FromPrimitive + ToPrimitive + Copy,
{
    fn compute(&self, data: &D) -> TestResult<F> {
        let data_slice = data.as_ref();
        let n = data_slice.len();

        // Handle empty sample
        if n == 0 {
            return TestResult {
                observed_statistic: F::zero(),
                p_value: F::from(1.0).expect("1.0 is valid float"),
            };
        }

        let n_f = F::from(n).expect("sample size fits in float");
        let normal = Normal::new(0.0, 1.0).expect("Valid standard normal distribution");

        let mut d_plus_max = F::zero();
        let mut d_minus_max = F::zero();

        // Compute KS statistic using order statistics
        let ecdf =  CDF.compute(data);
        for (i, &x) in ecdf.points().iter().enumerate() {
            let i_f = F::from(i).expect("index fits in float");
            let i1_f = F::from(i + 1).expect("index+1 fits in float");

            // Convert to f64 for statrs CDF computation
            let x_f64 = x.to_f64().expect("value convertible to f64");
            let f_x = F::from(normal.cdf(x_f64)).expect("CDF value convertible to float");

            // D⁺ = max_i [i/n - F(x_i)] (using 1-based indexing: i → i+1)
            let d_plus = i1_f / n_f - f_x;
            // D⁻ = max_i [F(x_i) - (i-1)/n] (left limit at x_i is i/n in 0-based)
            let d_minus = f_x - i_f / n_f;

            if d_plus > d_plus_max {
                d_plus_max = d_plus;
            }
            if d_minus > d_minus_max {
                d_minus_max = d_minus;
            }
        }

        // KS statistic is the maximum of the two one-sided deviations
        let d_max = if d_plus_max > d_minus_max {
            d_plus_max
        } else {
            d_minus_max
        };

        // Compute p-value using asymptotic approximation
        let d_f64 = d_max.to_f64().expect("statistic convertible to f64");
        let p_value_f64 = if d_f64 <= 0.0 {
            1.0 // Perfect fit
        } else if d_f64 >= 1.0 {
            0.0 // Maximum possible deviation
        } else {
            // Asymptotic series: p = 2 * Σ_{k=1}^∞ (-1)^(k-1) * exp(-2*k²*D²*n)
            let n_f64 = n as f64;
            let mut p = 0.0;
            let mut k = 1;
            let mut prev_term = f64::INFINITY;

            // Sum until convergence or max iterations
            while k <= 100 {
                let exponent = -2.0 * (k as f64).powi(2) * d_f64 * d_f64 * n_f64;
                // Prevent underflow for large exponents
                if exponent < -700.0 {
                    break;
                }
                let term = (-1.0f64).powi(k - 1) * exponent.exp();

                // Break when terms become negligible
                if term.abs() < 1e-15 || term.abs() < prev_term * 1e-12 {
                    p += term;
                    break;
                }

                p += term;
                prev_term = term.abs();
                k += 1;
            }

            // Apply two-sided correction and clamp to valid probability range
            let p_val = 2.0 * p;
            p_val.max(0.0).min(1.0)
        };

        let p_value = F::from(p_value_f64).expect("p-value convertible to float");

        TestResult {
            observed_statistic: d_max,
            p_value,
        }
    }
}
