use ressa::Error as RessaError;
use std::{error};

#[derive(Debug)]
pub enum CompilerError {
    Parser(RessaError),
    Unsupported(String),
    Custom(String)
}

pub type CompilerResult<V> = Result<V, CompilerError>;

impl CompilerError {
    pub fn is_unsupported<D>(error: &str, unsuppoted: D) -> Self
        where D: std::fmt::Debug
    {
        CompilerError::Unsupported(format!("{} '{:?}' is not supported", error, unsuppoted))
    }

    pub fn are_unsupported(error: &str) -> Self {
        CompilerError::Unsupported(format!("'{}' are not supported", error))
    }
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TO{0}", "DO")
    }
}

impl std::error::Error for CompilerError {
    fn description(&self) -> &str {
        match *self {
            CompilerError::Parser(_) => unimplemented!("RessaError handling"),
            CompilerError::Unsupported(ref s) |
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
