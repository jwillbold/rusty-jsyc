use ressa::Parser;
use resast::prelude::*;
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