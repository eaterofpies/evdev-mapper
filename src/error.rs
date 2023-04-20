use std::{
    error::Error,
    fmt::{Display, Formatter},
    io,
};

#[derive(Debug)]
pub enum FatalError {
    Str(String),
    Io(io::Error),
    SerdeYaml(serde_yaml::Error),
}

impl Display for FatalError {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Io(e) => Display::fmt(e, f),
            Self::SerdeYaml(e) => Display::fmt(e, f),
            Self::Str(e) => Display::fmt(e, f),
        }
    }
}

impl Error for FatalError {}

impl From<&'static str> for FatalError {
    fn from(err: &'static str) -> FatalError {
        FatalError::Str(String::from(err))
    }
}

impl From<io::Error> for FatalError {
    fn from(err: io::Error) -> FatalError {
        FatalError::Io(err)
    }
}

impl From<serde_yaml::Error> for FatalError {
    fn from(err: serde_yaml::Error) -> FatalError {
        FatalError::SerdeYaml(err)
    }
}

#[derive(Debug)]
pub enum NonFatalError {
    Io(std::io::Error),
    Str(String),
}

impl From<&'static str> for NonFatalError {
    fn from(err: &'static str) -> NonFatalError {
        NonFatalError::Str(String::from(err))
    }
}

impl From<String> for NonFatalError {
    fn from(err: String) -> NonFatalError {
        NonFatalError::Str(err)
    }
}
