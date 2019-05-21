use ressa::{Parser};

pub struct JSSourceCode
{
    pub source_code: String
}

impl JSSourceCode {
    pub fn new(source_code: String) -> Self {
        JSSourceCode { source_code }
    }

    pub fn from_str(js_code: &str) -> Self {
        JSSourceCode::new(js_code.into())
    }
}

pub struct JSAst
{
    pub ast: resast::Program
}

impl JSAst {
    pub fn parse(source: &JSSourceCode) -> Result<Self, ressa::Error> {
        let mut parser = match Parser::new(&source.source_code) {
            Ok(parser) => parser,
            Err(e) => { return Err(e); }
        };

        match parser.parse() {
            Ok(ast) => Ok(JSAst{ ast }),
            Err(e) => Err(e)
        }
    }
}
