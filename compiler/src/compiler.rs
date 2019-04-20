use crate::error::{CompilerError};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode};
use crate::scope::*;
use crate::bytecode::{*};

pub use resast::prelude::*;
pub use resast::prelude::Pat::Identifier;
use std::borrow::Borrow;
// use std::boxed::Box;

pub type CompilerResult<V> = Result<V, CompilerError>;
pub type BytecodeResult = Result<Bytecode, CompilerError>;


// pub struct ImportantRegisters {
//     undefined: Register,
//
// }

#[derive(Clone)]
pub struct BytecodeCompiler
{
    scopes: Scopes,
}

impl BytecodeCompiler {

    pub fn new() -> Self {
        BytecodeCompiler{
            scopes: Scopes::new(),
        }
    }

    pub fn add_decl(&mut self, decl: String) -> Result<Register, CompilerError> {
        self.scopes.add_decl(decl)
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
                    self.compile_program_part(part)
                }).collect::<Result<Bytecode, CompilerError>>()?
            },
        };

        Ok(bytecode)
    }

    fn compile_program_part(&mut self, progrm_part: &ProgramPart) -> BytecodeResult {
        match progrm_part {
            resast::ProgramPart::Dir(_) => Err(CompilerError::Custom("Directives are not supported".into())),
            resast::ProgramPart::Decl(decl) => self.compile_decl(&decl),
            resast::ProgramPart::Stmt(stmt) => self.compile_stmt(&stmt)
        }
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
            Stmt::Expr(expr) => self.compile_expr(&expr, *self.scopes.get_throwaway_register()?),
            Stmt::Block(stmts) => stmts.iter().map(|part| self.compile_program_part(part)).collect(),
            Stmt::Empty => Ok(Bytecode::new()),
            Stmt::Debugger => Err(CompilerError::are_unsupported("Debugger statments")),
            Stmt::With(_) => Err(CompilerError::are_unsupported("'with' statments")),
            Stmt::Throw(_) => Err(CompilerError::are_unsupported("'throw' statments")),
            Stmt::Try(_) => Err(CompilerError::are_unsupported("'try' statments")),
            Stmt::Var(decls) => self.compile_var_decl(&VariableKind::Var, &decls),
            _ => Err(CompilerError::is_unsupported("Statement type"))
        }
    }

    fn maybe_compile_expr(&mut self, expr: &Expr, target_reg: Option<Register>) -> Result<(Bytecode, Register), CompilerError> {
        let (opt_bytecode, target_reg) = match expr {
            Expr::Ident(ident) => match self.scopes.get_var(ident) {
                Ok(var) => (Some(Bytecode::new()), Some(var.register)),
                Err(_) => (None, target_reg)
            },
            _ => (None, target_reg)
        };

        let target_reg = match target_reg {
            Some(reg) => reg,
            None => self.scopes.reserve_register()?
        };

        let bytecode = match opt_bytecode {
            Some(bc) => bc,
            None => self.compile_expr(expr, target_reg)?
        };

        Ok((bytecode, target_reg))
    }

    fn compile_expr(&mut self, expr: &Expr, target_reg: Register) -> Result<Bytecode, CompilerError> {
        match expr {
            Expr::Call(call) => {
                let mut arg_regs = Vec::new();

                let bytecode = call.arguments.iter().map(|arg| {
                    let (arg_bc, arg_reg) = self.maybe_compile_expr(arg, None)?;
                    arg_regs.push(arg_reg);
                    Ok(arg_bc)
                }).collect::<BytecodeResult>()?;

                let (callee_bc, callee_reg) = self.maybe_compile_expr(call.callee.borrow(), Some(target_reg))?;

                Ok(bytecode
                    .combine(callee_bc)
                    .add(Command::new(Instruction::CallFunc, vec![
                            Operand::Register(target_reg),
                            Operand::Register(callee_reg),
                            Operand::RegistersArray(arg_regs)
                        ]
                )))
            },
            Expr::Ident(ident) => self.compile_operand_assignment(target_reg, Operand::Register(self.scopes.get_var(&ident)?.register)),
            Expr::Literal(lit) => self.compile_operand_assignment(target_reg, Operand::from_literal(lit.clone())?),
            Expr::Member(member) => {
                let (obj_bc, obj_reg) = self.maybe_compile_expr(member.object.borrow(), None)?;
                let (prop_bc, prop_reg) =  match member.property.borrow() {
                    Expr::Ident(ident) => self.maybe_compile_expr(&Expr::Literal(Literal::String(ident.to_string())), None)?,
                    _ => self.maybe_compile_expr(member.property.borrow(), None)?
                };

                Ok(obj_bc.combine(prop_bc)
                    .add(Command::new(Instruction::PropAccess, vec![
                            Operand::Register(target_reg), Operand::Register(obj_reg), Operand::Register(prop_reg)
                        ]
                    )))
            },
            _ => Err(CompilerError::is_unsupported("Expression type")),
        }
    }

    fn compile_func(&mut self, func: &Function) -> Result<Bytecode, CompilerError> {
        if func.generator || func.is_async {
            return Err(CompilerError::are_unsupported("generator ans async functions"))
        }

        unimplemented!("Functions")
    }

    fn compile_operand_assignment(&self, left: Register, right: Operand) -> Result<Bytecode, CompilerError> {
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
        Bytecode::new().add(Command::new(Instruction::Copy,
            vec![Operand::Register(test_expr_ident.scopes.get_var("testVar".into()).unwrap().register),
                 Operand::Register(test_expr_ident_reg)])));

     let mut test_expr_str_lit = BytecodeCompiler::new();
     assert_eq!(test_expr_str_lit.compile_var_decl(&VariableKind::Var, &vec![
             VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Literal(Literal::String("TestString".into())))}
         ]).unwrap(),
         Bytecode::new().add(Command::new(Instruction::LoadString,
             vec![Operand::Register(test_expr_str_lit.scopes.get_var("testVar".into()).unwrap().register),
                  Operand::String("TestString".into())])));
}
