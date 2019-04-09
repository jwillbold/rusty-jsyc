use std::error::Error;

use ressa::Parser;
use resast::prelude::*;

pub struct JSSourceCode
{
    // pub source_code: String
}

#[derive(Debug, PartialEq)]
pub struct Bytecode
{

}

// struct Parser
// {
//
// }

pub struct BytecodeCompiler
{

}

impl BytecodeCompiler {
    pub fn compile(&self, source: &JSSourceCode) -> Result<Bytecode, Box<Error>> {
        Ok(Bytecode{})
    }
}


fn main() {
    let js = "function helloWorld() { alert('Hello world'); }";
    let p = Parser::new(&js).unwrap();
    let f = ProgramPart::decl(
        Decl::Function(
            Function {
                id: Some("helloWorld".to_string()),
                params: vec![],
                body: vec![
                    ProgramPart::Stmt(
                        Stmt::Expr(
                            Expr::call(Expr::ident("alert"), vec![Expr::string("'Hello world'")])
                        )
                    )
                ],
                generator: false,
                is_async: false,
            }
        )
    );
    for part in p {
        assert_eq!(part.unwrap(), f);
    }
}

