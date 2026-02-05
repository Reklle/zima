use std::path::Path;
use std::error::Error as StdError;
use std::fmt;

use serde::de::DeserializeOwned;
use csv::ReaderBuilder;
use super::Sample;

#[derive(Debug)]
pub enum SampleError {
    Io(std::io::Error),
    Csv(csv::Error),
    EmptyFile,
}

impl fmt::Display for SampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SampleError::Io(e) => write!(f, "I/O error: {}", e),
            SampleError::Csv(e) => write!(f, "CSV parsing error: {}", e),
            SampleError::EmptyFile => write!(f, "CSV file contains no data records"),
        }
    }
}

impl StdError for SampleError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            SampleError::Io(e) => Some(e),
            SampleError::Csv(e) => Some(e),
            SampleError::EmptyFile => None,
        }
    }
}

impl From<std::io::Error> for SampleError {
    fn from(e: std::io::Error) -> Self {
        SampleError::Io(e)
    }
}

impl From<csv::Error> for SampleError {
    fn from(e: csv::Error) -> Self {
        SampleError::Csv(e)
    }
}

impl<T> Sample<T> {
    /// Read sample data from a CSV file with headers matching struct fields
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, SampleError>
    where
        T: DeserializeOwned,
    {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut records = Vec::new();
        for result in rdr.deserialize() {
            records.push(result?);
        }

        if records.is_empty() {
            return Err(SampleError::EmptyFile);
        }

        Ok(Self { data: records })
    }
}
