use crate::error::{CompilerError};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode};
use crate::scope::{*};

pub use resast::prelude::*;
pub use resast::prelude::Pat::Identifier;


pub struct BytecodeCompiler
{
    scopes: Scopes
}

impl BytecodeCompiler {

    pub fn new() -> Self {
        BytecodeCompiler{
            scopes: Scopes::new()
        }
    }

    pub fn compile(&mut self, source: &JSSourceCode) -> Result<Bytecode, CompilerError> {
        let ast = match JSAst::parse(source) {
            Ok(ast) => ast,
            Err(e) => { return Err(CompilerError::from(e)); }
        };

        match ast.ast {
            resast::Program::Mod(m) => { return Err(CompilerError::Custom("ES6 Modules are not supported".into())); },
            resast::Program::Script(s) => {
                for part in s {
                    match part {
                        resast::ProgramPart::Dir(_) => { return Err(CompilerError::Custom("Directives are not supported".into())); },
                        resast::ProgramPart::Decl(decl) => {},
                        resast::ProgramPart::Stmt(stmt) => {}
                    }
                }
            },
        }


        Ok(Bytecode{
            commands: vec![]
        })
    }

    fn compile_decl(&mut self, decl: &Decl) -> Result<Bytecode, CompilerError> {
        match decl {
            Decl::Variable(var_kind, var_decls) => self.compile_var_decl(var_kind, var_decls),
            Decl::Function(func) => Ok(Bytecode{commands:vec![]}),
            Decl::Class(_) => Err(CompilerError::Custom("Class declarations are not supported".into())),
            Decl::Import(_) => Err(CompilerError::Custom("Import declarations are not supported".into())),
            Decl::Export(_) => Err(CompilerError::Custom("Export declarations are not supported".into())),
        }
    }

    fn compile_var_decl(&mut self, kind: &VariableKind, decls: &[VariableDecl]) -> Result<Bytecode, CompilerError> {
        match kind {
            VariableKind::Let => { warn!("'let' will be treated as 'var'"); }
            VariableKind::Const => { info!("'const' will be trated as 'var'"); }
            _ => {}
        }

        Ok(decls.iter().map(|decl| {
            match &decl.id {
                Identifier(ident) => {
                    let reg = self.scopes.add_decl(ident.to_string());
                }
                Pat::Array(_) => { return Err(CompilerError::Custom("'Array Patterns' are not supported".into())); },
                Pat::Object(_) => { return Err(CompilerError::Custom("'Object Patterns' are not supported".into())); },
                Pat::RestElement(_) => { return Err(CompilerError::Custom("'Rest Elements' are not supported".into())); }
                Pat::Assignment(_) => { return Err(CompilerError::Custom("'Assignment Patterns' are not supported".into())); }
            }

            Ok(Bytecode{commands:vec![]})
        }).flatten().collect())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<Bytecode, CompilerError> {
        Ok(Bytecode{commands:vec![]})
    }
}
