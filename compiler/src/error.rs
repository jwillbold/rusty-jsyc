use ressa::Error as RessaError;
use std::{error, fmt};

#[derive(Debug)]
pub enum CompilerError {
    Parser(RessaError),
    // NotSupported(String),
    Custom(String)
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TO{0}", "DO")
    }
}

impl std::error::Error for CompilerError {
    fn description(&self) -> &str {
        match *self {
            CompilerError::Parser(ref e) => "TODO",
            // CompilerError::NotSupported(ref s) => format!("'{}' is not supported", s).as_str(),
            CompilerError::Custom(ref s) => s.as_str(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            _ => None
        }
    }
}

impl From<RessaError> for CompilerError {
    fn from(err: RessaError) -> CompilerError {
        CompilerError::Parser(err)
    }
}