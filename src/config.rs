use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

pub type CResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    WrongArguments,
    IOError(std::io::Error),
    Other,
}

impl ConfigError {
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
            ConfigError::IOError(ref cause) => write!(f, "{}", cause),
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

#[derive(Debug)]
pub struct Config {
    pub(super) source: PathBuf,
    pub(super) destination: PathBuf,
}

impl Config {
    pub fn from_args() -> CResult<Config> {
        use std::{collections::VecDeque, env};

        let mut args = env::args()
            .skip(1)
            .map(PathBuf::from)
            .collect::<VecDeque<_>>();

        let (Some(source), Some(destination)) = (args.pop_front(), args.pop_front()) else {
            return Err(ConfigError::WrongArguments);
        };

        Ok(Config::build(source, destination))
    }

    pub fn build(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
        }
    }

    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    pub fn destination(&self) -> &PathBuf {
        &self.destination
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
