mod read;

use std::iter::Iterator;
use crate::statistics::Statistic;

#[derive(Debug, Clone, Default)]
pub struct Sample<T> {
    pub data: Vec<T>,
}

impl<T> Sample<T> {
    /// Create a new sample from raw data
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Get the number of observations in the sample
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the sample contains no observations
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Estimate a statistic from the sample data
    pub fn estimate<Output, D>(&self, statistic: impl Statistic<Self, Output>) -> Output
    where
        Self: AsRef<[T]>,
        T: Clone,
    {
        statistic.compute(&self)
    }
}

impl<T> FromIterator<T> for Sample<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Sample::new(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for Sample<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

pub trait SamplingIterator: Iterator {
    fn sample(self, n: usize) -> Sample<Self::Item>
    where
        Self: Sized,
    {
        self.take(n).collect()
    }
}

impl<I: Iterator> SamplingIterator for I {}

impl<T> AsRef<[T]> for Sample<T> {
    fn as_ref(&self) -> &[T] { &self.data }
}
