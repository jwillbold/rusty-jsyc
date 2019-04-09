use crate::error::{CompilerError};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode};

pub use resast::prelude::*;


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

    fn compile_decl(&mut self, decl: &resast::Decl) -> Result<Bytecode, CompilerError> {
        match decl {
            resast::Decl::Variable(var_kind, var_decls) => Ok(Bytecode{commands:vec![]}),
            resast::Decl::Function(func) => Ok(Bytecode{commands:vec![]}),
            resast::Decl::Class(_) => { return Err(CompilerError::Custom("Class declarations are not supported".into())); },
            resast::Decl::Import(_) => { return Err(CompilerError::Custom("Import declarations are not supported".into())); },
            resast::Decl::Export(_) => { return Err(CompilerError::Custom("Export declarations are not supported".into())); },
        }
    }
}
