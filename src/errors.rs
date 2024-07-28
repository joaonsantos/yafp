//! Defines errors encountered when parsing.
use std::fmt::{self, Display};

/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// A missing required argument.
    MissingArgument(String),

    /// A missing value to an argument.
    MissingValue(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingArgument(key) => {
                write!(f, "argument '{}' is required", key)
            }
            Error::MissingValue(key) => {
                write!(f, "argument '{}' requires a value", key)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Alias for a [`std::result::Result`] with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
