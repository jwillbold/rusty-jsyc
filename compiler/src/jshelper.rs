use ressa::Parser;

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