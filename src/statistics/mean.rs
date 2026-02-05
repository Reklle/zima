use num_traits::Float;

use crate::math::Vector;
use super::Statistic;

/// Computes component-wise arithmetic mean using **Kahan summation** to
/// minimize floating-point error accumulation. This is critical when:
/// - Summing >10⁴ values
/// - Values have large dynamic range
/// - High precision required for downstream statistics
#[derive(Clone, Copy, Default)]
pub struct Mean;

impl<D, T> Statistic<D, T> for Mean
where
    D: AsRef<[T]>,
    T: Vector,
{
    fn compute(&self, data: &D) -> T {
        let slice: &[T] = data.as_ref();

        // if slice.is_empty() {
        //     return T::nan();
        // }

        // Kahan summation: compensates for floating-point rounding errors
        let mut sum = T::zero();
        let mut c = T::zero();

        for &x in slice {
            let y = x - c;
            let t = sum + y;
            c = (t - sum) - y;
            sum = t;
        }

        // Length conversion is exact for practical dataset sizes
        // (f32: exact ≤ 16M elements; f64: exact ≤ 9 quadrillion)
        sum * T::from_usize(slice.len()).recip()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    #[test]
    fn empty_slice_returns_nan() {
        let mean_f32: f32 = Mean.compute(&Vec::<f32>::new());
        assert!(mean_f32.is_nan(), "Empty slice must return NaN (got: {})", mean_f32);

        let mean_f64: f64 = Mean.compute(&Vec::<f64>::new());
        assert!(mean_f64.is_nan(), "Empty slice must return NaN (got: {})", mean_f64);
    }

    #[test]
    fn single_element_returns_value() {
        assert_abs_diff_eq!(Mean.compute(&[42.5_f32]), 42.5, epsilon = 1e-6);
        assert_abs_diff_eq!(Mean.compute(&[std::f64::consts::PI]), std::f64::consts::PI, epsilon = 1e-12);
    }

    #[test]
    fn exact_integer_means() {
        assert_abs_diff_eq!(Mean.compute(&[1.0_f32, 2.0, 3.0, 4.0, 5.0]), 3.0, epsilon = 1e-6);
        assert_abs_diff_eq!(Mean.compute(&[1.0_f64, 2.0, 3.0, 4.0, 5.0]), 3.0, epsilon = 1e-12);
    }

    #[test]
    fn handles_negative_values_and_zero() {
        assert_abs_diff_eq!(Mean.compute(&[-5.0_f32, -2.0, 0.0, 3.0, 4.0]), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(Mean.compute(&[-10.5_f64, -3.2, 0.0, 7.1, 6.6]), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn kahan_reduces_accumulation_error() {
        // Summing 0.1 × 10,000 exposes naive summation drift
        let n = 10_000;
        let data: Vec<f32> = vec![0.1_f32; n];
        let expected = 0.1_f32;

        let kahan_mean = Mean.compute(&data);
        let naive_mean: f32 = data.iter().sum::<f32>() / (n as f32);

        // Kahan must be strictly more accurate
        let kahan_error = (kahan_mean - expected).abs();
        let naive_error = (naive_mean - expected).abs();
        assert!(
            kahan_error < naive_error * 0.5,
            "Kahan error ({:.2e}) should be <50% of naive error ({:.2e})",
            kahan_error,
            naive_error
        );

        // Absolute accuracy guarantee
        assert_abs_diff_eq!(kahan_mean, expected, epsilon = 5e-5);
    }

    #[test]
    fn maintains_precision_at_scale() {
        // 1M values at 1e-10 tests both magnitude stability and accumulation fidelity
        let n = 1_000_000;
        let small = 1e-10_f64;
        let data: Vec<f64> = vec![small; n];

        // Relative error is appropriate for small magnitudes
        assert_relative_eq!(
            Mean.compute(&data),
            small,
            epsilon = 1e-13,
            max_relative = 1e-13
        );
    }

    #[test]
    fn symmetric_distribution_yields_zero_mean() {
        // Stress-test cancellation behavior with balanced positives/negatives
        let data: Vec<f64> = (-1000..=1000)
            .map(|x| x as f64 * 0.123456789)
            .collect();
        assert_abs_diff_eq!(Mean.compute(&data), 0.0, epsilon = 1e-10);
    }
}
