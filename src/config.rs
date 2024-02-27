//! Configuration structure

use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

/// Config Result type used for error propogation while creating
/// config instance
pub type CResult<T> = Result<T, ConfigError>;

/// 'Error' type representing application configuration issues
/// See [the module level documentation](index.html) for more.
#[derive(Debug)]
pub enum ConfigError {
    /// Arguments passed were incorrect
    WrongArguments,
    /// [std::io::Error] wrapper to represent errors from environment variables
    IOError(std::io::Error),
    /// Unknown type of error
    Other,
}

impl ConfigError {
    /// String slice representation of the underlying error
    fn as_str(&self) -> &str {
        use ConfigError::*;
        match *self {
            WrongArguments => "Wrong arguments",
            IOError(_) => "IO Error",
            Other => "Other error",
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConfigError::WrongArguments => f.write_str(self.as_str()),
            ConfigError::IOError(ref cause) => {
                write!(f, "{}", cause)
            }
            ConfigError::Other => f.write_str(self.as_str()),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

/// Configuration of the application.
///
/// Stores only source and destination paths.
///
#[derive(Debug)]
pub struct Config {
    /// Source path to monitor changes
    pub(super) source: PathBuf,
    /// Destination path for syncronisation
    pub(super) destination: PathBuf,
}

impl Config {
    /// Construct instance from command line arguments.
    ///
    /// # Panics
    /// Should not panic
    ///
    /// # Errors
    /// Will return [Err(ConfigError::WrongArguments)](ConfigError::WrongArguments)
    /// if less than two arguments were given
    /// Arguments mapped via [PathBuf::from] function, which should not fail.
    /// However, paths could probably be invalid.
    pub fn from_args() -> CResult<Config> {
        use std::{collections::VecDeque, env};

        let mut args = env::args().skip(1).map(PathBuf::from).collect::<VecDeque<_>>();

        let (Some(source), Some(destination)) = (args.pop_front(), args.pop_front()) else {
            return Err(ConfigError::WrongArguments);
        };

        Ok(Config::build(source, destination))
    }

    /// Default builder from two paths.
    ///
    /// # Examples
    ///
    /// Simple builder usage:
    ///
    /// ```
    /// let config = Config::build(
    ///     "./sync_test/dir1".into(),
    ///     "./sync_test/dir2".into(),
    /// );
    /// ```
    pub fn build(source: PathBuf, destination: PathBuf) -> Self {
        Self { source, destination }
    }

    /// Source getter
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    /// Destination getter
    pub fn destination(&self) -> &PathBuf {
        &self.destination
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
