use compiler::{CompilerError};
use std::{error};

pub type CompositionResult<V> = Result<V, CompositionError>;

#[derive(Debug)]
pub enum CompositionError {
    Compiler(CompilerError),
    IoError(std::io::Error),
    Custom(String)
}

impl CompositionError {
    pub fn from_str(msg: &str) -> CompositionError {
        CompositionError::Custom(msg.into())
    }
}

impl std::fmt::Display for CompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TO{0}", "DO")
    }
}

impl std::error::Error for CompositionError {
    fn description(&self) -> &str {
        match *self {
            CompositionError::Compiler(ref e) => e.description(),
            CompositionError::IoError(ref e) => e.description(),
            CompositionError::Custom(ref s) => s.as_str(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            _ => None
        }
    }
}

impl From<CompilerError> for CompositionError {
    fn from(err: CompilerError) -> CompositionError {
        CompositionError::Compiler(err)
    }
}

impl From<std::io::Error> for CompositionError {
    fn from(err: std::io::Error) -> CompositionError {
        CompositionError::IoError(err)
    }
}
