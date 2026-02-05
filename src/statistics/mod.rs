pub trait Statistic<D, T> {
    fn compute(&self, data: &D) -> T;
}

pub trait Standardize<D, F> {
    fn standardize(&self, data: &D) -> D;
}

impl<D, F> Standardize<D, F> for Mean
where
    D: AsRef<[F]> + FromIterator<F>,
    F: Float + FromPrimitive,
{
    fn standardize(&self, data: &D) -> D {
        let mean = self.compute(data);
        data.as_ref()
            .iter()
            .map(|x| *x - mean)
            .collect()
    }
}

impl<D, F> Standardize<D, F> for Variance
where
    D: AsRef<[F]> + FromIterator<F>,
    F: Float + FromPrimitive,
{
    fn standardize(&self, data: &D) -> D {
        let var = self.compute(data);
        data.as_ref()
            .iter()
            .map(|x| *x / var.sqrt())
            .collect()
    }
}


mod mean;
mod basic;
mod se;
mod studentized;
mod cdf;
mod studentized_bootstrap;
mod quantile;
mod ci;


pub use mean::Mean;
pub use basic::*;

use num_traits::{Float, FromPrimitive};
pub use se::{SEMean, SE};
pub use studentized::Studentized;
pub use cdf::{CDF, EmpiricalCDF};
pub use quantile::{Quantile, QuantileInterval};
pub use studentized_bootstrap::StudentizedBootstrap;
pub use ci::{Interval, IntervalStyle};

// ===== 0-tuple: Identity statistic (no-op) =====
impl<D> Statistic<D, ()> for () {
    #[inline]
    fn compute(&self, _data: &D) -> () {
        ()
    }
}

// ===== 1-tuple: Transparent wrapper =====
impl<D, T1, S1> Statistic<D, (T1,)> for (S1,)
where
    S1: Statistic<D, T1>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1,) {
        (self.0.compute(data),)
    }
}

// ===== 2-tuple =====
impl<D, T1, T2, S1, S2> Statistic<D, (T1, T2)> for (S1, S2)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2) {
        let out1 = self.0.compute(data);
        let out2 = self.1.compute(data);
        (out1, out2)
    }
}

// ===== 3-tuple =====
impl<D, T1, T2, T3, S1, S2, S3> Statistic<D, (T1, T2, T3)> for (S1, S2, S3)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
        )
    }
}

// ===== 4-tuple =====
impl<D, T1, T2, T3, T4, S1, S2, S3, S4>
    Statistic<D, (T1, T2, T3, T4)> for (S1, S2, S3, S4)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
    S4: Statistic<D, T4>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3, T4) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
            self.3.compute(data),
        )
    }
}

// ===== 5-tuple =====
impl<D, T1, T2, T3, T4, T5, S1, S2, S3, S4, S5>
    Statistic<D, (T1, T2, T3, T4, T5)> for (S1, S2, S3, S4, S5)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
    S4: Statistic<D, T4>,
    S5: Statistic<D, T5>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3, T4, T5) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
            self.3.compute(data),
            self.4.compute(data),
        )
    }
}

// ===== 6-tuple =====
impl<D, T1, T2, T3, T4, T5, T6, S1, S2, S3, S4, S5, S6>
    Statistic<D, (T1, T2, T3, T4, T5, T6)> for (S1, S2, S3, S4, S5, S6)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
    S4: Statistic<D, T4>,
    S5: Statistic<D, T5>,
    S6: Statistic<D, T6>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3, T4, T5, T6) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
            self.3.compute(data),
            self.4.compute(data),
            self.5.compute(data),
        )
    }
}

// ===== 7-tuple =====
impl<D, T1, T2, T3, T4, T5, T6, T7, S1, S2, S3, S4, S5, S6, S7>
    Statistic<D, (T1, T2, T3, T4, T5, T6, T7)> for (S1, S2, S3, S4, S5, S6, S7)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
    S4: Statistic<D, T4>,
    S5: Statistic<D, T5>,
    S6: Statistic<D, T6>,
    S7: Statistic<D, T7>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3, T4, T5, T6, T7) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
            self.3.compute(data),
            self.4.compute(data),
            self.5.compute(data),
            self.6.compute(data),
        )
    }
}

// ===== 8-tuple =====
impl<
    D,
    T1, T2, T3, T4, T5, T6, T7, T8,
    S1, S2, S3, S4, S5, S6, S7, S8,
> Statistic<D, (T1, T2, T3, T4, T5, T6, T7, T8)>
    for (S1, S2, S3, S4, S5, S6, S7, S8)
where
    S1: Statistic<D, T1>,
    S2: Statistic<D, T2>,
    S3: Statistic<D, T3>,
    S4: Statistic<D, T4>,
    S5: Statistic<D, T5>,
    S6: Statistic<D, T6>,
    S7: Statistic<D, T7>,
    S8: Statistic<D, T8>,
{
    #[inline]
    fn compute(&self, data: &D) -> (T1, T2, T3, T4, T5, T6, T7, T8) {
        (
            self.0.compute(data),
            self.1.compute(data),
            self.2.compute(data),
            self.3.compute(data),
            self.4.compute(data),
            self.5.compute(data),
            self.6.compute(data),
            self.7.compute(data),
        )
    }
}
