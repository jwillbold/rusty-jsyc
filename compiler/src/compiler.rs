use crate::error::{CompilerError};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode};
use crate::scope::*;
use crate::bytecode::{*};

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

        let bytecode = match ast.ast {
            resast::Program::Mod(_) => { return Err(CompilerError::Custom("ES6 Modules are not supported".into())); },
            resast::Program::Script(s) => {
                s.iter().map(|part| {
                    match part {
                        resast::ProgramPart::Dir(_) => Err(CompilerError::Custom("Directives are not supported".into())),
                        resast::ProgramPart::Decl(decl) => self.compile_decl(&decl),
                        resast::ProgramPart::Stmt(stmt) => self.compile_stmt(&stmt)
                    }
                }).collect::<Result<Bytecode, CompilerError>>()?
            },
        };

        Ok(bytecode)
    }

    fn compile_decl(&mut self, decl: &Decl) -> Result<Bytecode, CompilerError> {
        match decl {
            Decl::Variable(var_kind, var_decls) => self.compile_var_decl(var_kind, var_decls),
            Decl::Function(func) => self.compile_func(func),
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

        decls.iter().map(|decl| {
            match &decl.id {
                Pat::Identifier(ident) => {
                    let reg = self.scopes.add_decl(ident.to_string())?;
                    match &decl.init {
                        Some(expr) => self.compile_expr(expr, reg),
                        None => Ok(Bytecode::new())
                    }
                }
                Pat::Array(_) => Err(CompilerError::Custom("'Array Patterns' are not supported".into())),
                Pat::Object(_) => Err(CompilerError::Custom("'Object Patterns' are not supported".into())),
                Pat::RestElement(_) => Err(CompilerError::Custom("'Rest Elements' are not supported".into())),
                Pat::Assignment(_) => Err(CompilerError::Custom("'Assignment Patterns' are not supported".into()))
            }
        }).collect()
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<Bytecode, CompilerError> {
        match stmt {
            Stmt::Var(decls) => self.compile_var_decl(&VariableKind::Var, &decls),
            Stmt::Empty => Ok(Bytecode::new()),

            _ => Err(CompilerError::is_unsupported("Statement type"))
        }
    }

    fn compile_expr(&mut self, expr: &Expr, target_reg: Register) -> Result<Bytecode, CompilerError> {
        match expr {
            Expr::Ident(ident) => self.compile_operand_assignment(target_reg, Operand::Register(self.scopes.get_var(&ident)?.register)),
            Expr::Literal(lit) => self.compile_operand_assignment(target_reg, Operand::from_literal(lit.clone())?),
            _ => Err(CompilerError::is_unsupported("Expression type")),
        }
    }

    fn compile_func(&mut self, func: &Function) -> Result<Bytecode, CompilerError> {
        if func.generator || func.is_async {
            return Err(CompilerError::are_unsupported("generator ans async functions"))
        }

        unimplemented!("Functions")
    }

    fn compile_operand_assignment(&mut self, left: Register, right: Operand) -> Result<Bytecode, CompilerError> {
        Ok(Bytecode::new().add(Command::new(right.get_assign_instr_type(), vec![Operand::Register(left), right])))
    }
}

#[test]
fn test_bytecode_compile_var_decl() {
    assert_eq!(BytecodeCompiler::new().compile_var_decl(&VariableKind::Var, &vec![
            VariableDecl{id: Pat::Identifier("testVar".into()), init: None}
        ]).unwrap(),
        Bytecode::new());

    let mut test_expr_ident = BytecodeCompiler::new();
    let test_expr_ident_reg = test_expr_ident.scopes.add_decl("anotherVar".into()).unwrap();
    assert_eq!(test_expr_ident.compile_var_decl(&VariableKind::Var, &vec![
            VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Ident("anotherVar".into()))}
        ]).unwrap(),
        Bytecode::new().add(Command::new(Instruction::LoadNum,
            vec![Operand::Register(test_expr_ident.scopes.get_var("testVar".into()).unwrap().register),
                 Operand::Register(test_expr_ident_reg)])));

     let mut test_expr_str_lit = BytecodeCompiler::new();
     assert_eq!(test_expr_str_lit.compile_var_decl(&VariableKind::Var, &vec![
             VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Literal(Literal::String("TestString".into())))}
         ]).unwrap(),
         Bytecode::new().add(Command::new(Instruction::LoadString,
             vec![Operand::Register(test_expr_str_lit.scopes.get_var("testVar".into()).unwrap().register),
                  Operand::Str("TestString".into())])));
}
