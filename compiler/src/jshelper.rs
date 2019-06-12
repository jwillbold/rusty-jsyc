use crate::error::{CompilerError};

use ressa::{Parser};

/// A wrapper for JavaScript source code
///
/// ```
/// use jsyc_compiler::JSSourceCode;
///
/// let js_code = jsyc_compiler::JSSourceCode::new("console.log('Hello World')".into());
/// ```
///
pub struct JSSourceCode {
    source_code: String
}

impl JSSourceCode {
    pub fn new(source_code: String) -> Self {
        JSSourceCode { source_code }
    }

    pub fn from_str(js_code: &str) -> Self {
        JSSourceCode::new(js_code.into())
    }
}

/// A wrapper for the AST of the provided JavaScript code
///
/// ```
/// use jsyc_compiler::{JSSourceCode, JSAst};
///
/// let js_code = JSSourceCode::new("console.log('Hello World')".into());
/// let js_ast = JSAst::parse(&js_code).expect("Failed to parse input code");
/// ```
pub struct JSAst {
    pub ast: resast::Program
}

impl JSAst {
    pub fn parse(source: &JSSourceCode) -> Result<Self, CompilerError> {
        let mut parser = match Parser::new(&source.source_code) {
            Ok(parser) => parser,
            Err(e) => { return Err(CompilerError::Parser(e)); }
        };

        match parser.parse() {
            Ok(ast) => Ok(JSAst{ ast }),
            Err(e) => Err(CompilerError::Parser(e))
        }
    }
}
