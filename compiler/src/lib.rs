extern crate ressa;
extern crate resast;
extern crate base64;

pub mod error;
pub mod bytecode;

use ressa::Parser;
use resast::prelude::*;

use crate::error::{CompilerError};
pub use crate::bytecode::{Bytecode};


pub struct JSSourceCode
{
    pub source_code: String
}

pub struct JSAst
{
    pub ast: resast::Program
}

impl JSAst {
    pub fn parse(source: &JSSourceCode) -> Result<Self, ressa::Error> {
        let mut parser = match ressa::Parser::new(&source.source_code) {
            Ok(parser) => parser,
            Err(e) => { return Err(e); }
        };

        match parser.parse() {
            Ok(ast) => Ok(JSAst{ ast }),
            Err(e) => Err(e)
        }
    }
}

pub struct BytecodeCompiler
{
    // ast: JSAst
}

impl BytecodeCompiler {
    pub fn compile(&mut self, source: &JSSourceCode) -> Result<Bytecode, CompilerError> {
        let ast = match JSAst::parse(source) {
            Ok(ast) => ast,
            Err(e) => { return Err(CompilerError::from(e)); }
        };

        Ok(Bytecode{
            commands: vec![]
        })
    }
}


