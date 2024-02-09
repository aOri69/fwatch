use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

pub type CResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    MissingArguments,
    IOError(std::io::Error),
    Other,
}

impl ConfigError {
    fn as_str(&self) -> &str {
        use ConfigError::*;
        match *self {
            MissingArguments => "Missing input arguments",
            IOError(_) => "IO Error",
            Other => "Other error",
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ConfigError::MissingArguments => f.write_str(self.as_str()),
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
    source: PathBuf,
    destination: PathBuf,
}

impl Config {
    pub fn from_args() -> CResult<Config> {
        use std::env;

        let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();

        dbg!(&args);

        if args.len() < 2 {
            Ok(Config::build(
                env::current_dir()?,
                PathBuf::from(args.first().ok_or(ConfigError::MissingArguments)?),
            ))
        } else {
            Ok(Config::build(
                PathBuf::from(args.first().ok_or(ConfigError::MissingArguments)?),
                PathBuf::from(args.get(1).ok_or(ConfigError::MissingArguments)?),
            ))
        }
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
