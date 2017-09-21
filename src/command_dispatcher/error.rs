use store;
use openssl;
use std::error::Error;
use std;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    StoreError(store::error::StoreError),
    IoError(std::io::Error),
    OpensslError(openssl::error::ErrorStack),
    ParseIntError(std::num::ParseIntError),
    OtherError(&'static str),
}

impl Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::StoreError(ref err) => err.description(),
            CliError::IoError(ref err) => err.description(),
            CliError::OpensslError(ref err) => err.description(),
            CliError::ParseIntError(ref err) => err.description(),
            CliError::OtherError(m) => m,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            CliError::StoreError(ref err) => Some(err as &Error),
            CliError::IoError(ref err) => Some(err as &Error),
            CliError::OpensslError(ref err) => Some(err as &Error),
            CliError::ParseIntError(ref err) => Some(err as &Error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::IoError(err)
    }
}

impl From<openssl::error::ErrorStack> for CliError {
    fn from(err_stack: openssl::error::ErrorStack) -> CliError {
        CliError::OpensslError(err_stack)
    }
}

impl From<std::num::ParseIntError> for CliError {
    fn from(err: std::num::ParseIntError) -> CliError {
        CliError::ParseIntError(err)
    }
}

impl From<store::error::StoreError> for CliError {
    fn from(err: store::error::StoreError) -> CliError {
        CliError::StoreError(err)
    }
}


impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::OpensslError(ref err) => fmt::Display::fmt(err, f),
            CliError::IoError(ref err) => fmt::Display::fmt(err, f),
            CliError::ParseIntError(ref err) => fmt::Display::fmt(err, f),
            CliError::StoreError(ref err) => fmt::Display::fmt(err, f),
            CliError::OtherError(m) => f.write_str(m),
        }
    }
}
