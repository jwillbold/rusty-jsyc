use crate::error::{CompilerError, CompilerResult};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode};
use crate::scope::*;
use crate::bytecode::{*};
use crate::instruction_set::InstructionSet;

pub use resast::prelude::*;
pub use resast::prelude::Pat::Identifier;
use std::borrow::Borrow;
// use std::boxed::Box;
use std::collections::HashMap;

pub type BytecodeResult = Result<Bytecode, CompilerError>;


#[derive(Clone)]
pub struct BytecodeFunction
{
    ident: String,
    bytecode: Bytecode,
    arguments: Vec<Register>,
    ast: Option<Function>
}

#[derive(Clone)]
pub struct LabelGenerator
{
    counter: u32
}

impl LabelGenerator {
    pub fn new() -> Self {
        LabelGenerator {
            counter: 0
        }
    }

    pub fn generate_label(&mut self) -> Label {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

#[derive(Clone)]
pub struct BytecodeCompiler
{
    scopes: Scopes,
    functions: Vec<BytecodeFunction>,
    isa: InstructionSet,
    label_generator: LabelGenerator
}

impl BytecodeCompiler {

    pub fn new() -> Self {
        let mut scopes = Scopes::new();
        let isa = InstructionSet::default(scopes.current_scope_mut().unwrap());

        BytecodeCompiler{
            scopes: scopes,
            functions: Vec::new(),
            isa: isa,
            label_generator: LabelGenerator::new()
        }
    }

    pub fn add_var_decl(&mut self, decl: String) -> Result<Register, CompilerError> {
        self.scopes.add_var_decl(decl)
    }

    pub fn compile(&mut self, source: &JSSourceCode) -> Result<Bytecode, CompilerError> {
        let ast = match JSAst::parse(source) {
            Ok(ast) => ast,
            Err(e) => { return Err(CompilerError::from(e)); }
        };

        let mut bytecode = match ast.ast {
            resast::Program::Mod(_) => { return Err(CompilerError::are_unsupported("ES6 modules")); },
            resast::Program::Script(s) => {
                s.iter().map(|part| {
                    self.compile_program_part(part)
                }).collect::<Result<Bytecode, CompilerError>>()?
            },
        };

        bytecode = self.finalize_label_addresses(bytecode)?;

        if self.functions.is_empty() {
            Ok(bytecode)
        } else {
            self.finalize_function_bytescodes(bytecode.add(Command::new(Instruction::Exit, vec![])))
        }
    }

    fn compile_program_part(&mut self, progrm_part: &ProgramPart) -> BytecodeResult {
        match progrm_part {
            resast::ProgramPart::Dir(_) => Err(CompilerError::are_unsupported("Directives")),
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
                    let reg = self.scopes.add_var_decl(ident.to_string())?;
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
            Stmt::Return(ret) => self.compile_return_stmt(ret),
            // Stmt::Labled()
            // Stmt::Break()
            // Stmt::Continue()
            Stmt::If(if_stmt) => self.compile_if_stmt(if_stmt),
            // Stmt::Switch()
            Stmt::Throw(_) => Err(CompilerError::are_unsupported("'throw' statments")),
            Stmt::Try(_) => Err(CompilerError::are_unsupported("'try' statments")),
            Stmt::While(while_stmt) => self.compile_while_stmt(while_stmt),
            Stmt::DoWhile(dowhile_stmt) => self.compile_dowhile_stmt(dowhile_stmt),
            Stmt::For(for_stmt) => self.compile_for_stmt(for_stmt),
            Stmt::ForIn(_) => Err(CompilerError::are_unsupported("for-in statments")),
            Stmt::ForOf(_) => Err(CompilerError::are_unsupported("for-of statments")),
            Stmt::Var(decls) => self.compile_var_decl(&VariableKind::Var, &decls),
            _ => Err(CompilerError::is_unsupported("Statement type"))
        }
    }

    fn compile_return_stmt(&mut self, ret: &Option<Expr>) -> BytecodeResult {
        let (bytecode, ret_regs) = match ret {
            Some(ret_expr) => {
                let (bytecode, ret_reg) = self.maybe_compile_expr(ret_expr, None)?;
                (bytecode, vec![ret_reg])
            },
            None => (Bytecode::new(), vec![])
        };

        Ok(bytecode
            .add(Command::new(Instruction::ReturnBytecodeFunc,
                              vec![Operand::RegistersArray(ret_regs)]))
        )
    }

    fn compile_if_stmt(&mut self, if_stmt: &IfStmt) -> BytecodeResult {
        let (test_bytecode, test_reg) = self.maybe_compile_expr(&if_stmt.test, None)?;

        let if_branch_bc = self.compile_stmt(if_stmt.consequent.borrow())?;

        let if_branch_end_label = self.label_generator.generate_label();
        let else_branch_end_label = self.label_generator.generate_label();

        let bytecode = test_bytecode
                .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(if_branch_end_label)]))
                .combine(if_branch_bc);

        if let Some(else_branch) = if_stmt.alternate.borrow() {
            let else_branch_bc = self.compile_stmt(&else_branch.borrow())?;
            //If-Else
            Ok(bytecode
                .add(Command::new(Instruction::Jump, vec![Operand::branch_addr(else_branch_end_label)]))
                .add_label(if_branch_end_label)
                .combine(else_branch_bc)
                .add_label(else_branch_end_label)
            )
        } else {
            // If
            Ok(bytecode
                .add_label(if_branch_end_label))
        }
    }

    fn compile_while_stmt(&mut self, while_stmt: &WhileStmt) -> BytecodeResult {
        let (test_bc, test_reg) = self.maybe_compile_expr(&while_stmt.test, None)?;

        let while_cond_label = self.label_generator.generate_label();
        let while_end_label = self.label_generator.generate_label();

        Ok(test_bc
            .add_label(while_cond_label)
            .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(while_end_label)]))
            .combine(self.compile_stmt(while_stmt.body.borrow())?)
            .add(Command::new(Instruction::Jump, vec![Operand::branch_addr(while_cond_label)]))
            .add_label(while_end_label))
    }

    fn compile_dowhile_stmt(&mut self, dowhile_stmt: &DoWhileStmt) -> BytecodeResult {
        let body_bc = self.compile_stmt(dowhile_stmt.body.borrow())?;
        let (test_bc, test_reg) = self.maybe_compile_expr(&dowhile_stmt.test, None)?;

        let dowhile_start_label = self.label_generator.generate_label();

        Ok(Bytecode::new()
            .add_label(dowhile_start_label)
            .combine(body_bc)
            .combine(test_bc)
            .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(dowhile_start_label)])))
    }

    fn compile_for_stmt(&mut self, for_stmt: &ForStmt) -> BytecodeResult {
        let init_bc = match &for_stmt.init {
            Some(loop_init) => match loop_init {
                LoopInit::Variable(kind, decls) => self.compile_var_decl(&kind, &decls)?,
                LoopInit::Expr(expr) => self.maybe_compile_expr(&expr, None)?.0
            },
            None => Bytecode::new()
        };

        let loop_start_label = self.label_generator.generate_label();
        let loop_end_label = self.label_generator.generate_label();

        let test_bc = match &for_stmt.test {
            Some(test_expr) => {
                let (test_bc, test_reg) = self.maybe_compile_expr(&test_expr, None)?;

                test_bc
                    .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(loop_end_label)]))
            }
            None => Bytecode::new()
        };

        let update_bc = match &for_stmt.update {
            Some(update_expr) => self.maybe_compile_expr(&update_expr, None)?.0,
            None => Bytecode::new()
        };

        let body_bc = self.compile_stmt(&for_stmt.body)?;

        Ok(init_bc
            .add_label(loop_start_label)
            .combine(test_bc)
            .combine(body_bc)
            .combine(update_bc)
            .add(Command::new(Instruction::Jump, vec![Operand::branch_addr(loop_start_label)]))
            .add_label(loop_end_label))
    }

    fn maybe_compile_expr(&mut self, expr: &Expr, target_reg: Option<Register>) -> CompilerResult<(Bytecode, Register)> {
        let (opt_bytecode, opt_reg) = match expr {
            Expr::Ident(ident) => match self.scopes.get_var(ident) {
                Ok(var) => match target_reg {
                    Some(reg) => (Some(self.compile_operand_assignment(reg, Operand::Reg(var.register))?), Some(reg)),
                    None => (Some(Bytecode::new()), Some(var.register)),
                },
                Err(_) => (None, None)
            },
            // TODO: Check test_member_expr
            // Expr::Member(member) => match member.object.borrow() {
            //         Expr::Ident(obj_ident) => match member.property.borrow() {
            //                 Expr::Ident(prop_ident) => {
            //                     match self.scopes.get_var(&format!("{}.{}", obj_ident, prop_ident)) {
            //                         Ok(var) => (Some(Bytecode::new()), Some(var.register)),
            //                         Err(_) => (None, target_reg)
            //                     }
            //                 },
            //                 _ => (None, target_reg)
            //         },
            //         _ => (None, target_reg)
            // },
            _ => (None, None)
        };

        let target_reg = match opt_reg {
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
            Expr::Array(array_exprs) => self.compile_array_expr(array_exprs, target_reg),
            Expr::ArrowFunction(_) => Err(CompilerError::are_unsupported("Arrow functions")),
            Expr::ArrowParamPlaceHolder(_,_) => Err(CompilerError::are_unsupported("Arrow parameter placeholder")),
            Expr::Assignment(assignment) => self.compile_assignment_expr(assignment, target_reg),
            Expr::Await(_) => Err(CompilerError::are_unsupported("'await' expressions")),
            Expr::Binary(bin) => self.compile_binary_expr(bin, target_reg),
            Expr::Class(_) => Err(CompilerError::are_unsupported("'class' expressions")),
            Expr::Call(call) => self.compile_call_expr(call, target_reg),
            Expr::Conditional(cond) => self.compile_conditional_expr(cond, target_reg),
            Expr::Function(_) => Err(CompilerError::are_unsupported("function expressions")),
            Expr::Ident(ident) => self.compile_operand_assignment(target_reg, Operand::Reg(self.scopes.get_var(&ident)?.register)),
            Expr::Literal(lit) => self.compile_operand_assignment(target_reg, Operand::from_literal(lit.clone())?),
            Expr::Logical(logical) => self.compile_logical_expr(logical, target_reg),
            Expr::Member(member) => self.compile_member_expr(member, target_reg),
            Expr::MetaProperty(_) => Err(CompilerError::are_unsupported("meta properties")),
            Expr::New(_) => Err(CompilerError::are_unsupported("object related expressions (new, this, {})")),
            Expr::Object(_) => Err(CompilerError::are_unsupported("object related expressions (new, this, {})")),
            Expr::Sequence(_) => Err(CompilerError::are_unsupported("seqeunce expressions")),
            Expr::Spread(_) => Err(CompilerError::are_unsupported("spread expressions")),
            Expr::Super => Err(CompilerError::are_unsupported("'super' expressions")),
            Expr::TaggedTemplate(_) => Err(CompilerError::are_unsupported("tagged template expressions")),
            Expr::This => Err(CompilerError::are_unsupported("object related expressions (new, this, {})")),
            Expr::Update(update) => self.compile_update_expr(update, target_reg),
            Expr::Unary(unary) => self.compile_unary_expr(unary, target_reg),
            Expr::Yield(_) => Err(CompilerError::are_unsupported("'yield' expressions")),
        }
    }

    fn compile_array_expr(&mut self, array: &ArrayExpr, target_reg: Reg) -> BytecodeResult {
        let (bytecodes, regs): (Vec<Bytecode>, Vec<Reg>) = array.iter().map(|opt_expr| {
            match opt_expr {
                Some(expr) => self.maybe_compile_expr(expr, None),
                None => Err(CompilerError::are_unsupported("'null' array fields"))
            }
        }).collect::<CompilerResult<Vec<(Bytecode, Reg)>>>()?.into_iter().unzip();

        Ok(bytecodes.into_iter().collect::<Bytecode>()
            .add(Command::new(Instruction::LoadArray, vec![Operand::Reg(target_reg), Operand::RegistersArray(regs)]))
        )
    }

    fn compile_assignment_expr(&mut self, assign: &AssignmentExpr, _target_reg: Register) -> BytecodeResult {
        let (left_bc, left_reg) = match &assign.left {
            AssignmentLeft::Pat(_) => { return Err(CompilerError::are_unsupported("Patterns in assignments")); },
            AssignmentLeft::Expr(expr) => self.maybe_compile_expr(&expr, None)?
        };

        match assign.operator {
            AssignmentOperator::Equal => {
                Ok(left_bc.combine(self.compile_expr(assign.right.borrow(), left_reg)?))
            }
            _ => {
                let (right_bc, right_reg) = self.maybe_compile_expr(assign.right.borrow(), None)?;
                Ok(left_bc.combine(right_bc)
                    .add(self.isa.assignment_op(&assign.operator, left_reg, right_reg)))
            }
        }
    }

    fn compile_binary_expr(&mut self, bin: &BinaryExpr, target_reg: Reg) -> BytecodeResult {
        let (left_bc, left_reg) = self.maybe_compile_expr(bin.left.borrow(), None)?;
        let (right_bc, right_reg) = self.maybe_compile_expr(bin.right.borrow(), None)?;

        Ok(left_bc
            .combine(right_bc)
            .add(self.isa.binary_op(&bin.operator, target_reg, left_reg, right_reg)?)
        )
    }

    fn compile_call_expr(&mut self, call: &CallExpr, target_reg: Register) -> BytecodeResult {
        match call.callee.borrow() {
            Expr::Ident(ident) => {
                if self.functions.iter().any(|func| func.ident==*ident) {
                    self.compile_bytecode_func_call(ident.to_string(), &call.arguments, target_reg)
                } else {
                    self.compile_extern_func_call(call, target_reg)
                }
            }
            _ => self.compile_extern_func_call(call, target_reg)
        }
    }

    fn compile_conditional_expr(&mut self, conditional: &ConditionalExpr, target_reg: Reg) -> BytecodeResult {
        let (test_bc, test_reg) = self.maybe_compile_expr(conditional.test.borrow(), None)?;
        let (consequent_bc, _) = self.maybe_compile_expr(conditional.consequent.borrow(), Some(target_reg))?;
        let (alt_bc, _) = self.maybe_compile_expr(conditional.alternate.borrow(), Some(target_reg))?;

        let after_alt_label = self.label_generator.generate_label();
        let after_cons_label = self.label_generator.generate_label();

        Ok(test_bc
            .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(after_alt_label)]))
            .combine(consequent_bc)
            .add(Command::new(Instruction::Jump, vec![Operand::branch_addr(after_cons_label)]))
            .add_label(after_alt_label)
            .combine(alt_bc)
            .add_label(after_cons_label))
    }

    fn compile_bytecode_func_call(&mut self, func: String, args: &[Expr], _target_reg: Reg) -> BytecodeResult {
        let (args_bytecode, arg_regs): (Vec<Bytecode>, Vec<Reg>) = args.iter().map(|arg_expr| {
            self.maybe_compile_expr(arg_expr, None)
        }).collect::<CompilerResult<Vec<(Bytecode, Reg)>>>()?.into_iter().unzip();

        Ok(args_bytecode.into_iter().collect::<Bytecode>()
            .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::function_addr(func), Operand::RegistersArray(arg_regs)])))
    }

    fn compile_extern_func_call(&mut self, call: &CallExpr, target_reg: Reg) -> BytecodeResult {
        let (callee_bc, callee_reg) = self.maybe_compile_expr(call.callee.borrow(), None)?;

        let (bytecode, arg_regs): (Vec<Bytecode>, Vec<Reg>) = call.arguments.iter().map(|arg| {
            self.maybe_compile_expr(arg, None)
        }).collect::<CompilerResult<Vec<(Bytecode, Reg)>>>()?.into_iter().unzip();

        Ok(bytecode.into_iter().collect::<Bytecode>()
            .combine(callee_bc)
            .add(Command::new(Instruction::CallFunc, vec![
                    Operand::Reg(target_reg),
                    Operand::Reg(callee_reg),
                    Operand::RegistersArray(arg_regs)
                ]
        )))
    }

    fn compile_operand_assignment(&self, left: Register, right: Operand) -> BytecodeResult {
        Ok(Bytecode::new().add(self.isa.load_op(left, right)))
    }

    fn compile_logical_expr(&mut self, logical: &LogicalExpr, target_reg: Reg) -> BytecodeResult {
        let (left_bc, left_reg) = self.maybe_compile_expr(logical.left.borrow(), None)?;
        let (right_bc, right_reg) = self.maybe_compile_expr(logical.right.borrow(), None)?;

        unimplemented!("logic expresions")
        // TODO how to lazy-evaluate right
        // left_bc
        //     .add(Command::new(Instruction::LogicAnd, ))
        //     .add(self.isa.logical_op(logical.operator, target_reg, left_reg, right_reg)?)
    }

    fn compile_member_expr(&mut self, member: &MemberExpr, target_reg: Register) -> BytecodeResult {
        let (obj_bc, obj_reg) = self.maybe_compile_expr(member.object.borrow(), None)?;
        let (prop_bc, prop_reg) =  match member.property.borrow() {
            Expr::Ident(ident) => self.maybe_compile_expr(&Expr::Literal(Literal::String(ident.to_string())), None)?,
            _ => self.maybe_compile_expr(member.property.borrow(), None)?
        };

        Ok(obj_bc.combine(prop_bc)
            .add(Command::new(Instruction::PropAccess, vec![
                    Operand::Reg(target_reg), Operand::Reg(obj_reg), Operand::Reg(prop_reg)
                ]
            )))
    }

    fn compile_update_expr(&mut self, update: &UpdateExpr, _target_reg: Register) -> BytecodeResult {
        if update.prefix {
            let (arg_bc, arg_reg) = self.maybe_compile_expr(update.argument.borrow(), None)?;
            Ok(arg_bc.add(self.isa.update_op(&update.operator, arg_reg)))
        } else {
            Err(CompilerError::are_unsupported("suffix update expressions"))
        }
    }

    fn compile_unary_expr(&mut self, unary: &UnaryExpr, target_reg: Register) -> BytecodeResult {
        if unary.prefix {
            let (arg_bc, arg_reg) = self.maybe_compile_expr(unary.argument.borrow(), None)?;
            Ok(arg_bc.add(self.isa.unary_op(&unary.operator, target_reg, arg_reg)?))
        } else {
            Err(CompilerError::are_unsupported("suffix unary expressions"))
        }
    }

    fn compile_func(&mut self, func: &Function) -> BytecodeResult {
        if func.generator || func.is_async {
            return Err(CompilerError::are_unsupported("generator and async functions"))
        }

        self.scopes.enter_new_scope()?;

        let arg_regs = func.params.iter().map(|param| {
            match param {
                FunctionArg::Expr(expr) => match expr {
                    Expr::Ident(ident) => self.scopes.add_var_decl(ident.to_string()),
                    _ => Err(CompilerError::Custom("Only identifiers are accepted as function arguments".into()))
                },
                FunctionArg::Pat(pat) => match pat {
                    Pat::Identifier(ident) => self.scopes.add_var_decl(ident.to_string()),
                    _ => Err(CompilerError::Custom("Only identifiers are accepted as function arguments".into()))
                }
            }
        }).collect::<CompilerResult<Vec<Register>>>()?;

        let mut func_bc = func.body.iter().map(|part| self.compile_program_part(&part)).collect::<BytecodeResult>()?;

        self.scopes.leave_current_scope()?;


        if !func_bc.last_op_is_return() {
            func_bc = func_bc.add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]));
        }

        self.functions.push(BytecodeFunction {
            ident: match &func.id {
                Some(ident) => ident.to_string(),
                None => unimplemented!("Anonymous function")
            },
            bytecode: func_bc,
            arguments: arg_regs,
            ast: Some(func.clone())
        });

        Ok(Bytecode::new())
    }

    fn finalize_label_addresses(&self, mut bc: Bytecode) -> BytecodeResult {
        let mut offset_counter = 0;
        let label_offsets: HashMap<Label, usize> = bc.elements.iter().filter_map(|element| {
            match element {
                BytecodeElement::Command(cmd) => {offset_counter += cmd.length_in_bytes(); None},
                BytecodeElement::Label(label) => Some((label.clone(), offset_counter.clone()))
            }
        }).collect();

        for cmd in bc.commands_iter_mut() {
            for op in cmd.operands.iter_mut() {
                if let Operand::BranchAddr(token) = op {
                    *op = Operand::LongNum(*label_offsets.get(&token.label).ok_or(
                        CompilerError::Custom(format!("Found unknown label {}", token.label))
                    )? as i64);
                }
            }
        }

        Ok(bc)
    }

    fn finalize_function_bytescodes(&self, main: Bytecode) -> BytecodeResult{
        let mut function_offsets: HashMap<String, usize> = HashMap::new();
        let mut offset_counter = main.length_in_bytes();

        let functions_bytecode: Bytecode = self.functions.iter().map(|func| {
            function_offsets.insert(func.ident.to_string(), offset_counter);
            offset_counter += func.bytecode.length_in_bytes();

            func.bytecode.clone()
        }).collect();

        let mut complete_bytecode = main.combine(functions_bytecode);

        for cmd in complete_bytecode.commands_iter_mut() {
            for op in cmd.operands.iter_mut() {
                if let Operand::FunctionAddr(token) = op {
                    *op = Operand::LongNum(*function_offsets.get(&token.ident).ok_or(
                        CompilerError::Custom(format!("Found unknown function ident {}", token.ident))
                    )? as i64);
                }
            }
        }

        Ok(complete_bytecode)
    }
}

#[test]
fn test_bytecode_compile_var_decl() {
    assert_eq!(BytecodeCompiler::new().compile_var_decl(&VariableKind::Var, &vec![
            VariableDecl{id: Pat::Identifier("testVar".into()), init: None}
        ]).unwrap(),
        Bytecode::new());

    let mut test_expr_ident = BytecodeCompiler::new();
    let test_expr_ident_reg = test_expr_ident.scopes.add_var_decl("anotherVar".into()).unwrap();
    assert_eq!(test_expr_ident.compile_var_decl(&VariableKind::Var, &vec![
            VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Ident("anotherVar".into()))}
        ]).unwrap(),
        Bytecode::new().add(Command::new(Instruction::Copy,
            vec![Operand::Reg(test_expr_ident.scopes.get_var("testVar".into()).unwrap().register),
                 Operand::Reg(test_expr_ident_reg)])));

     let mut test_expr_str_lit = BytecodeCompiler::new();
     assert_eq!(test_expr_str_lit.compile_var_decl(&VariableKind::Var, &vec![
             VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Literal(Literal::String("TestString".into())))}
         ]).unwrap(),
         Bytecode::new().add(Command::new(Instruction::LoadString,
             vec![Operand::Reg(test_expr_str_lit.scopes.get_var("testVar".into()).unwrap().register),
                  Operand::String("TestString".into())])));
}