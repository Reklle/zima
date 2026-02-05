use std::fmt;
use std::ops::{Add, Sub};
use num_traits::{NumOps, Float, FloatConst, One};

/// Statistical interval with optional estimate and confidence level.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Interval<T> {
    pub lower: T,
    pub upper: T,
    pub estimate: Option<T>,
    pub confidence: Option<f64>,
}

impl<T: PartialOrd + Copy> Interval<T> {
    /// Create asymmetric interval.
    #[inline]
    pub const fn new(lower: T, upper: T) -> Self {
        Self { lower, upper, estimate: None, confidence: None }
    }

    /// Create symmetric interval: `[estimate - error, estimate + error]`.
    #[inline]
    pub fn symmetric(estimate: T, error: T) -> Self
    where
        T: Sub<Output = T> + Add<Output = T>,
    {
        Self {
            lower: estimate - error,
            upper: estimate + error,
            estimate: Some(estimate),
            confidence: None,
        }
    }

    /// Fluent builder: attach point estimate.
    #[must_use]
    pub const fn estimate(mut self, estimate: T) -> Self {
        self.estimate = Some(estimate);
        self
    }

    /// Fluent builder: attach confidence level (0.0 < level < 1.0).
    #[must_use]
    pub const fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Check if value lies within `[lower, upper]` (inclusive).
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        self.lower <= *value && *value <= self.upper
    }

    /// Interval width: `upper - lower`.
    #[inline]
    pub fn width(&self) -> T
    where
        T: Sub<Output = T>,
    {
        self.upper - self.lower
    }

    /// Midpoint: `(lower + upper) / 2`.
    #[inline]
    pub fn midpoint(&self) -> T
    where
        T: NumOps + One,
    {
        (self.lower + self.upper) / (T::one() + T::one())
    }

    /// Basic validity check.
    #[inline]
    pub fn is_valid(&self) -> bool {
        if self.lower > self.upper {
            return false;
        }
        if let Some(est) = self.estimate {
            if est < self.lower || est > self.upper {
                return false;
            }
        }
        self.confidence.map_or(true, |c| c > 0.0 && c < 1.0)
    }
}

impl<T: Float> Interval<T> {
    /// Relative width: `(upper - lower) / |midpoint|`.
    #[inline]
    pub fn relative_width(&self) -> T {
        let width = self.width();
        let mid = self.midpoint();
        if mid.is_zero() {
            T::infinity()
        } else {
            width / mid.abs()
        }
    }

    /// Symmetric half-width.
    #[inline]
    pub fn half_width(&self) -> T {
        self.width() / (T::one() + T::one())
    }

    /// Check symmetry within relative tolerance.
    #[inline]
    pub fn is_symmetric(&self, rel_tol: T) -> bool {
        match self.estimate {
            Some(est) => {
                let left = (est - self.lower).abs();
                let right = (self.upper - est).abs();
                (left - right).abs() <= rel_tol * left.max(right)
            }
            None => {
                let mid = self.midpoint();
                let left = (mid - self.lower).abs();
                let right = (self.upper - mid).abs();
                (left - right).abs() <= rel_tol * left.max(right)
            }
        }
    }

    /// Interval representing NaN bounds.
    pub fn nan() -> Self {
        Self::new(T::nan(), T::nan())
    }

    /// Infinite interval `[-∞, +∞]`.
    pub fn infinite() -> Self {
        Self::new(T::neg_infinity(), T::infinity())
    }
}

/// Precision-aware formatter following GUM guidelines.
pub struct FormattedInterval<'a, T> {
    interval: &'a Interval<T>,
    style: IntervalStyle,
    uncertainty_digits: usize, // 1 or 2 significant digits for error
}

impl<'a, T: Float + fmt::Display + Copy> Interval<T> {
    /// Format with metrology-aware precision rules.
    pub fn format(&'a self, style: IntervalStyle) -> FormattedInterval<'a, T> {
        FormattedInterval {
            interval: self,
            style,
            uncertainty_digits: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IntervalStyle {
    /// Minimal bounds only: `[4.46, 5.43]`
    Bounds,
    /// Symmetric notation when applicable: `4.87 ± 0.48`
    Symmetric,
    /// Set notation: `4.87 ∈ [4.46, 5.43]`
    Set,
    /// Set notation with explicit confidence: `4.87 ∈ [4.46, 5.43] with 0.95`
    SetWith,
    /// Auto-select representation per GUM/statistical guidelines (default)
    #[default]
    Guideline,
}

impl<'a, T> fmt::Display for FormattedInterval<'a, T>
where
    T: Float + fmt::Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Interval { lower, upper, estimate, confidence } = *self.interval;
        let rel_tol = T::from(0.05).unwrap_or(T::zero());
        let is_sym = self.interval.is_symmetric(rel_tol);

        // Determine effective style per guidelines
        let effective_style = match self.style {
            IntervalStyle::Guideline => {
                if is_sym && confidence.is_none() {
                    IntervalStyle::Symmetric
                } else if confidence.is_some() {
                    IntervalStyle::SetWith
                } else {
                    IntervalStyle::Set
                }
            }
            style => style,
        };

        // Precision calculation
        let (est, left_err, right_err) = match estimate {
            Some(e) => (e, (e - lower).abs(), (upper - e).abs()),
            None => {
                let m = self.interval.midpoint();
                (m, (m - lower).abs(), (upper - m).abs())
            }
        };
        let max_err = left_err.max(right_err);

        if !max_err.is_finite() || max_err.is_zero() {
            // Safe fallback formatting using current bounds (T: Display + Copy)
            match effective_style {
                IntervalStyle::Bounds => write!(f, "[{}, {}]", lower, upper)?,
                IntervalStyle::Symmetric | IntervalStyle::Set | IntervalStyle::SetWith | IntervalStyle::Guideline => {
                    write!(f, "{} ∈ [{}, {}]", est, lower, upper)?
                }
            }
            if let Some(conf) = confidence {
                write!(f, " with {:.2}", conf)?;
            }
            return Ok(());
        }

        let max_err_f64 = max_err.to_f64().unwrap_or(f64::NAN);
        if !max_err_f64.is_finite() || max_err_f64 <= 0.0 {
            // Direct fallback without recursive Display call
            write!(f, "[{}, {}]", lower, upper)?;
            if let Some(e) = estimate {
                write!(f, " ({})", e)?;
            }
            if let Some(c) = confidence {
                write!(f, " with {:.2}", c)?;
            }
            return Ok(());
        }

        let err_digits = self.uncertainty_digits.min(2).max(1);
        let err_log10 = max_err_f64.log10().floor();
        let decimals = (-(err_log10 - (err_digits as f64 - 1.0))).ceil() as i32;

        let format_with_precision = |x: T| -> String {
            if !x.is_finite() {
                return format!("{}", x);
            }
            let x_f64 = x.to_f64().unwrap_or(f64::NAN);
            if !x_f64.is_finite() {
                return format!("{}", x);
            }
            format!("{:.*}", decimals.max(0) as usize, x_f64)
        };

        let lower_fmt = format_with_precision(lower);
        let upper_fmt = format_with_precision(upper);
        let est_fmt = format_with_precision(est);

        // Render according to effective style
        match effective_style {
            IntervalStyle::Bounds => {
                write!(f, "[{}, {}]", lower_fmt, upper_fmt)?;
            }
            IntervalStyle::Symmetric => {
                if is_sym {
                    let err_fmt = format_with_precision(max_err);
                    write!(f, "{} ± {}", est_fmt, err_fmt)?;
                } else {
                    write!(f, "[{}, {}]", lower_fmt, upper_fmt)?;
                }
            }
            IntervalStyle::Set => {
                write!(f, "{} ∈ [{}, {}]", est_fmt, lower_fmt, upper_fmt)?;
            }
            IntervalStyle::SetWith | IntervalStyle::Guideline => {
                if is_sym {
                    let err_fmt = format_with_precision(max_err);
                    write!(f, "{} ± {}", est_fmt, err_fmt)?;
                } else {
                    write!(f, "{} ∈ [{}, {}]", est_fmt, lower_fmt, upper_fmt)?;
                }
                if let Some(conf) = confidence {
                    write!(f, " with {:.2}", conf)?;
                }
            }
        }

        Ok(())
    }
}
