use super::Statistic;
use num_traits::Float;
use std::cmp::Ordering;
use std::fmt::Debug;


/// Empirical Cumulative Distribution Function (ECDF).
///
/// Represents the step function: Fₙ(x) = (1/n) * Σᵢ I(Xᵢ ≤ x)
///
/// # Float Handling Policy
/// For floating-point types (`f32`, `f64`):
/// - NaN values are **silently filtered** during construction (standard statistical practice)
/// - Remaining values are sorted using IEEE 754 semantics via `partial_cmp`
/// - Evaluation at NaN input returns `f64::NAN`
#[derive(Debug, Clone)]
pub struct EmpiricalCDF<T> {
    sorted: Vec<T>,
}

// Generic implementation for all Ord types (integers, strings, etc.)
// Note: f32/f64 do NOT implement Ord, so this path is mutually exclusive with float handling
impl<T: Clone> EmpiricalCDF<T> {
    // /// Creates ECDF from slice using standard Ord comparison.
    // pub fn from_slice(data: &[T]) -> Self {
    //     let mut sorted = data.to_vec();
    //     sorted.sort_unstable();
    //     Self { sorted }
    // }

    // /// Evaluates ECDF at point `x` using standard Ord comparison.
    // #[inline]
    // pub fn eval(&self, x: &T) -> f64 {
    //     let n = self.sorted.len();
    //     if n == 0 {
    //         return f64::NAN;
    //     }
    //     let idx = self.sorted.partition_point(|v| v <= x);
    //     idx as f64 / n as f64
    // }

    #[inline]
    pub fn n(&self) -> usize {
        self.sorted.len()
    }

    #[inline]
    pub fn points(&self) -> &[T] {
        &self.sorted
    }

    pub fn len(&self) -> usize {
            self.sorted.len()
        }

        pub fn is_empty(&self) -> bool {
            self.sorted.is_empty()
        }
}

// Specialized implementation for Float types with NaN-aware handling
impl<T> EmpiricalCDF<T>
where
    T: Float + Copy,
{
    /// Creates ECDF from float slice with NaN filtering.
    ///
    /// # Behavior
    /// - NaN values are **excluded** from the ECDF (not counted in `n()`)
    /// - Infinite values are preserved and sorted according to IEEE 754:
    ///   `-∞ < finite < +∞`
    /// - Empty result after filtering yields an empty ECDF (`n() == 0`)
    pub fn from_float_slice(data: &[T]) -> Self {
        // Filter out NaNs first (standard practice in statistical libraries)
        let mut sorted: Vec<T> = data.iter()
            .copied()
            .filter(|&x| !x.is_nan())
            .collect();

        // Sort remaining values using IEEE 754 partial ordering
        // Safe to unwrap because we filtered all NaNs
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("NaNs already filtered"));

        Self { sorted }
    }

    /// Evaluates ECDF at float point with IEEE 754 semantics.
    ///
    /// # Returns
    /// - `f64::NAN` if input `x` is NaN
    /// - `0.0` for `-∞`
    /// - `1.0` for `+∞`
    /// - Otherwise: proportion of values ≤ `x` using standard float comparison
    #[inline]
    pub fn eval_float(&self, x: &T) -> f64 {
        // Handle special cases explicitly
        if x.is_nan() {
            return f64::NAN;
        }
        if x.is_infinite() {
            return if x.is_sign_positive() { 1.0 } else { 0.0 };
        }

        let n = self.sorted.len();
        if n == 0 {
            return f64::NAN;
        }

        // Count values ≤ x using IEEE 754 partial ordering
        // partition_point requires total predicate - we use partial_cmp defensively
        let idx = self.sorted.partition_point(|v| {
            // Values that are not greater than x (i.e., less or equal)
            // map_or(false, ...) handles hypothetical NaNs defensively (shouldn't occur)
            v.partial_cmp(x).map_or(false, |ord| ord != Ordering::Greater)
        });

        idx as f64 / n as f64
    }

    /// Internal helper: count values ≤ x without float conversion
    #[inline]
    fn count_leq(&self, x: &T) -> usize {
        self.sorted.partition_point(|v| {
            v.partial_cmp(x).map_or(false, |ord| ord != Ordering::Greater)
        })
    }
}

// First-order Stochastic Dominance (FSD) partial ordering
impl<T> PartialOrd for EmpiricalCDF<T>
where
    T: Float + Copy + Debug,
{
    /// Partial ordering based on First-order Stochastic Dominance (FSD).
    ///
    /// # Definition
    /// ECDF `F` dominates `G` (written `F ≼ G`) iff:
    ///   ∀x: F(x) ≤ G(x)
    ///
    /// This means `F` assigns higher probability to larger values.
    ///
    /// # Ordering Semantics
    /// - `F.partial_cmp(&G) == Some(Less)` → `F` dominates `G` (F is "better")
    /// - `F.partial_cmp(&G) == Some(Greater)` → `G` dominates `F`
    /// - `Some(Equal)` → identical distributions
    /// - `None` → incomparable (crossing CDFs)
    ///
    /// # Implementation
    /// Uses exact rational comparison (k₁/n₁ vs k₂/n₂) to avoid floating-point errors:
    ///   k₁/n₁ ≤ k₂/n₂  ⇔  k₁·n₂ ≤ k₂·n₁
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Empty ECDFs are incomparable (undefined distribution)
        if self.sorted.is_empty() || other.sorted.is_empty() {
            return None;
        }

        // Collect all unique jump points from both distributions
        let mut points: Vec<T> = Vec::with_capacity(self.sorted.len() + other.sorted.len());
        points.extend_from_slice(&self.sorted);
        points.extend_from_slice(&other.sorted);

        // Sort and deduplicate using IEEE 754 ordering
        points.sort_by(|a, b| a.partial_cmp(b).expect("No NaNs in ECDF points"));
        points.dedup_by(|a, b| a == b); // Float equality is sufficient for jump points

        let n_self = self.n() as u64;
        let n_other = other.n() as u64;

        let mut self_dominates = true;  // self(x) ≤ other(x) ∀x ?
        let mut other_dominates = true; // other(x) ≤ self(x) ∀x ?

        for &x in &points {
            let k_self = self.count_leq(&x) as u64;
            let k_other = other.count_leq(&x) as u64;

            // Exact rational comparison: k_self/n_self ≤ k_other/n_other ?
            // ⇔ k_self * n_other ≤ k_other * n_self
            let self_le_other = k_self * n_other <= k_other * n_self;
            let other_le_self = k_other * n_self <= k_self * n_other;

            if !self_le_other {
                self_dominates = false;
            }
            if !other_le_self {
                other_dominates = false;
            }

            // Early termination if incomparable
            if !self_dominates && !other_dominates {
                return None;
            }
        }

        match (self_dominates, other_dominates) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),    // self dominates other
            (false, true) => Some(Ordering::Greater), // other dominates self
            (false, false) => None,
        }
    }
}

impl<T> PartialEq for EmpiricalCDF<T>
where
    T: Float + Copy + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// ECDF statistic constructor.
#[derive(Debug, Clone, Copy, Default)]
pub struct CDF;

// // Implementation for Ord types (integers, strings, etc.)
// impl<D, T> Statistic<D, EmpiricalCDF<T>> for CDF
// where
//     D: AsRef<[T]>,
//     T: Ord + Clone,
// {
//     #[inline]
//     fn compute(&self, data: &D) -> EmpiricalCDF<T> {
//         EmpiricalCDF::from_slice(data.as_ref())
//     }
// }

// Implementation for Float types with NaN filtering
impl<D, T> Statistic<D, EmpiricalCDF<T>> for CDF
where
    D: AsRef<[T]>,
    T: Float + Copy,
{
    #[inline]
    fn compute(&self, data: &D) -> EmpiricalCDF<T> {
        EmpiricalCDF::from_float_slice(data.as_ref())
    }
}

// use plotters::coord::Shift;
// use plotters::prelude::*;
// use statrs::distribution::{Normal, ContinuousCDF};

// impl<T> EmpiricalCDF<T>
// where
//     T: Clone + PartialOrd + Into<f64>,
// {
//     pub fn plot<DB>(
//         &self,
//         root: DrawingArea<DB, Shift>,
//         mean: f64,
//         std_dev: f64,
//     ) -> Result<(), Box<dyn std::error::Error>>
//     where
//         DB: DrawingBackend + 'static,
//         DB::ErrorType: 'static,
//     {
//         if self.is_empty() {
//             return Ok(());
//         }

//         let n = self.len() as f64;
//         let points: Vec<f64> = self.sorted.iter().map(|x| (*x).clone().into()).collect();

//         let x_min = points.first().copied().unwrap_or(0.0);
//         let x_max = points.last().copied().unwrap_or(1.0);
//         let pad = (x_max - x_min) * 0.05;
//         let x_start = x_min - pad.max(0.1);
//         let x_end = x_max + pad.max(0.1);

//         let mut chart = ChartBuilder::on(&root)
//             .caption("ECDF vs Theoretical Normal CDF", ("sans-serif", 18))
//             .margin(10)
//             .x_label_area_size(30)
//             .y_label_area_size(30)
//             .build_cartesian_2d(x_start..x_end, 0.0..1.0)?;

//         chart.configure_mesh().draw()?;

//         // ECDF (step function)
//         let mut ecdf_series = vec![(x_start, 0.0)];
//         for (i, &x) in points.iter().enumerate() {
//             let y_prev = i as f64 / n;
//             let y_curr = (i + 1) as f64 / n;
//             ecdf_series.push((x, y_prev));
//             ecdf_series.push((x, y_curr));
//         }
//         ecdf_series.push((x_end, 1.0));

//         chart.draw_series(LineSeries::new(ecdf_series, BLUE))?
//             .label("Empirical CDF")
//             .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

//         // Theoretical Normal CDF
//         let normal = Normal::new(mean, std_dev)
//             .map_err(|e| format!("Invalid normal distribution: {}", e))?;

//         let theoretical_series: Vec<(f64, f64)> = (0..200)
//             .map(|i| {
//                 let t = i as f64 / 199.0;
//                 let x = x_start + (x_end - x_start) * t;
//                 (x, normal.cdf(x))
//             })
//             .collect();

//         // Pass owned ShapeStyle: RED.stroke_width(2)
//         chart.draw_series(LineSeries::new(theoretical_series, RED.stroke_width(2)))?
//             .label("Theoretical Normal CDF")
//             .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

//         chart.configure_series_labels()
//             .background_style(WHITE.mix(0.8))
//             .border_style(BLACK)
//             .draw()?;

//         Ok(())
//     }
// }
