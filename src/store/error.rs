use std;
use std::error::Error;
use rusqlite;
use std::fmt;
use std::borrow::Borrow;

#[derive(Debug)]
pub enum StoreError {
    IoError(std::io::Error),
    InternalError(Box<Error>),
}

//TODO try to really convert errors.
impl From<rusqlite::Error> for StoreError {
    fn from(err: rusqlite::Error) -> StoreError {
        StoreError::InternalError(Box::new(err))
    }
}

impl From<std::io::Error> for StoreError {
    fn from(err: std::io::Error) -> StoreError {
        StoreError::IoError(err)
    }
}

impl Error for StoreError {
    fn description(&self) -> &str {
        match *self {
            StoreError::InternalError(ref err) => err.description(),
            StoreError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            StoreError::InternalError(ref err) => Some(err.borrow()),
            StoreError::IoError(ref err) => Some(err as &Error),
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StoreError::InternalError(ref err) => fmt::Display::fmt(err, f),
            StoreError::IoError(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
