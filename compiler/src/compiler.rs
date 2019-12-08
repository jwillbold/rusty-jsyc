use crate::error::{CompilerError, CompilerResult};
use crate::jshelper::{JSSourceCode, JSAst};
use crate::bytecode::{Bytecode, BytecodeResult};
use crate::scope::*;
use crate::bytecode::{*};
use crate::instruction_set::{InstructionSet, CommonLiteral, ReservedeRegister};

use resast::prelude::*;
use std::borrow::Borrow;
use std::collections::{HashMap};
use std::rc::Rc;


#[derive(Debug, Clone)]
struct BytecodeFunction
{
    ident: String,
    // During the compilation of a function block, "bytecode" is not known yet (obviously).
    // But an instance of this struct is anyway inserted into the function list,
    // to allow functions using callbacks to themselves.
    bytecode: Option<Bytecode>,
    arguments: Vec<Register>,
    // Same explanation as above for 'bytecode'
    used_decls: Option<Vec<Register>>,
}

impl BytecodeFunction {
    pub fn new_phantom(ident: Identifier, arg_regs: Vec<Register>) -> Self {
        BytecodeFunction {
            ident: ident,
            bytecode: None,
            arguments: arg_regs,
            used_decls: None,
        }
    }

    pub fn from_phantom(phantom: Self, bytecode: Bytecode, used_decls: Vec<Register>) -> Self {
        BytecodeFunction {
            ident: phantom.ident,
            bytecode: Some(bytecode),
            arguments: phantom.arguments,
            used_decls: Some(used_decls),
        }
    }
}

#[derive(Clone)]
struct LoopBlock {
    start_label: Label,
    end_label: Label
}

impl LoopBlock {
    pub fn new(start_label: Label, end_label: Label) -> Self {
        LoopBlock {start_label, end_label}
    }

    pub fn start_label(&self) -> Label {
        self.start_label
    }

    pub fn end_label(&self) -> Label {
        self.end_label
    }
}

#[derive(Clone)]
struct LabelGenerator
{
    counter: u32,
    // js_labels: HashMap<Identifier, &'a LoopBlock>,
    // loop_blocks: Vec<LoopBlock>,
    js_labels: HashMap<Identifier, Rc<LoopBlock>>,
    loop_blocks: Vec<Rc<LoopBlock>>,
    current_js_label: Option<Identifier>
}

impl LabelGenerator {
    pub fn new() -> Self {
        LabelGenerator {
            counter: 0,
            js_labels: HashMap::new(),
            loop_blocks: Vec::new(),
            current_js_label: None,
        }
    }

    pub fn generate_label(&mut self) -> Label {
        let counter = self.counter;
        self.counter += 1;
        counter
    }

    pub fn generate_loop_label_block(&mut self) -> Rc<LoopBlock> {
        let block = Rc::new(LoopBlock::new(self.generate_label(), self.generate_label()));
        self.loop_blocks.push(block.clone());

        if let Some(current_js_label) = &self.current_js_label {
            self.js_labels.insert(current_js_label.to_string(), block.clone());
        }

        block
    }

    pub fn get_js_labled_block(&self, js_label: &Identifier) -> Option<&LoopBlock> {
        self.js_labels.get(js_label).map(|x| x.borrow())
    }

    pub fn get_current_label_block(&self) -> Option<&LoopBlock> {
        self.loop_blocks.last().map(|x| x.borrow())
    }
}

/// Represents a set of declaration dependencies
///
/// In JavaScript there a severeal dependencies like ``document``, ``window`` or ``setInterval``.
/// The usage these of as well as their expected register postion is tracked in this object.
#[derive(Clone, Debug)]
pub struct DeclDepencies {
    pub decls_decps: HashMap<Identifier, Register>
}

impl DeclDepencies {
    pub fn new() -> Self {
        DeclDepencies {
            decls_decps: HashMap::new()
        }
    }

    pub fn add_decl_dep(&mut self, ident: Identifier, reg: Register) {
        self.decls_decps.insert(ident.to_string(), reg);
    }

    pub fn try_get_dep(&self, ident: &Identifier) -> Option<&Register> {
        self.decls_decps.get(ident)
    }
}

/// Compiles JavaScript source code into bytecode.
///
/// ```
/// use jsyc_compiler::{JSSourceCode, BytecodeCompiler};
///
/// let js_code = JSSourceCode::new("console.log('Hello World');".into());
/// let mut compiler = BytecodeCompiler::new();
///
/// let bytecode = compiler.compile(&js_code).expect("Failed to compile code");
/// println!("bytecode: {}", bytecode);
/// ```
#[derive(Clone)]
pub struct BytecodeCompiler {
    scopes: Scopes,
    // This is not a hashmap but a vector only to make tetsing easier
    functions: Vec<BytecodeFunction>,
    isa: InstructionSet,
    label_generator: LabelGenerator,
    decl_dependencies: DeclDepencies
}

// fn testy<'xzy>(s: &'xzy mut BytecodeCompiler<'xzy>, pp: &ProgramPart) -> BytecodeResult {
//     s.compile_program_part(pp)
// }

impl BytecodeCompiler {

    /// Creates a new bytecode compiler
    pub fn new() -> Self {
        let mut scopes = Scopes::new();
        let isa = InstructionSet::default(scopes.current_scope_mut().unwrap());
        isa.common_lits().add_to_lit_cache(&mut scopes).unwrap();

        BytecodeCompiler{
            scopes: scopes,
            functions: vec![],
            isa: isa,
            label_generator: LabelGenerator::new(),
            decl_dependencies: DeclDepencies::new()
        }
    }

    /// Add a variable decleration to the compiler
    ///
    /// By adding a variable declaration, you can inform the compiler about the existence of
    /// an external declaration. However, this is not necessary since dependencies are tracked
    /// and can be retrieved after the compilation through [decl_dependencies](struct.BytecodeCompiler.html#method.decl_dependencies).
    pub fn add_var_decl(&mut self, decl: String) ->  CompilerResult<Reg> {
        self.scopes.add_decl(decl, DeclarationType::Variable(MyVariableKind::Var))
    }

    /// Returns all dependencies on external declarations
    ///
    /// Usually JavaScript depends on several function and variable declarations from outside
    /// of the current script: such as `window`, `document` or `setInterval`.
    /// # Returns
    /// A list of these dependencies and in which register they are expected to be.
    /// This result will be available after BytecodeCompiler::compile ran.
    pub fn decl_dependencies(&self) -> &DeclDepencies{
        &self.decl_dependencies
    }

    /// Compiles the provided JavaScript code into bytecode.
    ///
    /// ```
    /// use jsyc_compiler::{JSSourceCode, BytecodeCompiler};
    ///
    /// let js_code = JSSourceCode::new("console.log('Hello World');".into());
    /// let mut compiler = BytecodeCompiler::new();
    ///
    /// let bytecode = compiler.compile(&js_code).expect("Failed to compile code");
    /// println!("bytecode: {}", bytecode);
    /// ```
    pub fn compile(&mut self, source: &JSSourceCode) -> BytecodeResult {
        let ast = JSAst::parse(source)?;
        let mut bytecode = match ast.ast {
            resast::Program::Mod(_) => Err(CompilerError::are_unsupported("ES6 modules")),
            resast::Program::Script(s) => {
                s.iter().map(|part| self.compile_program_part(part)).collect::<BytecodeResult>()
            },
        }?;

        bytecode = self.finalize_label_addresses(bytecode, 0)?;

        if self.functions.is_empty() {
            Ok(bytecode)
        } else {
            self.finalize_function_bytescodes(bytecode.add(Operation::new(Instruction::Exit, vec![])))
        }
    }

    pub fn compile_program_part(&mut self, program_part: &ProgramPart) -> BytecodeResult {
        match program_part {
            resast::ProgramPart::Dir(_) => Err(CompilerError::are_unsupported("Directives")),
            resast::ProgramPart::Decl(decl) => self.compile_decl(&decl),
            resast::ProgramPart::Stmt(stmt) => self.compile_stmt(&stmt)
        }
    }

    fn compile_decl(&mut self, decl: &Decl) -> BytecodeResult{
        match decl {
            Decl::Variable(var_kind, var_decls) => self.compile_var_decl(var_kind, var_decls),
            Decl::Function(func) => self.compile_func(func),
            Decl::Class(_) => Err(CompilerError::are_unsupported("Class declarations")),
            Decl::Import(_) => Err(CompilerError::are_unsupported("Import declarations")),
            Decl::Export(_) => Err(CompilerError::are_unsupported("Export declarations")),
        }
    }

    fn compile_var_decl(&mut self, kind: &VariableKind, decls: &[VariableDecl]) -> BytecodeResult {
        match kind {
            VariableKind::Let => { println!("Warning: 'let' will be treated as 'var'", ); }
            VariableKind::Const => { println!("Info: 'const' will be treated as 'var'"); }
            _ => {}
        }

        decls.iter().map(|decl| {
            match &decl.id {
                Pat::Identifier(ident) => {
                    let reg = self.scopes.add_decl(ident.to_string(), DeclarationType::Variable(MyVariableKind::from(kind)))?;
                    match &decl.init {
                        Some(expr) => Ok(self.maybe_compile_expr(expr, Some(reg))?.0),
                        None => Ok(Bytecode::new())
                    }
                }
                Pat::Array(_) => Err(CompilerError::are_unsupported("'Array Patterns'")),
                Pat::Object(_) => Err(CompilerError::are_unsupported("'Object Patterns'")),
                Pat::RestElement(_) => Err(CompilerError::are_unsupported("'Rest Elements'")),
                Pat::Assignment(_) => Err(CompilerError::are_unsupported("'Assignment Patterns'"))
            }
        }).collect()
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> BytecodeResult {
        match stmt {
            Stmt::Expr(expr) => self.compile_expr(&expr, self.isa.reserved_reg(&ReservedeRegister::TrashRegister)),
            Stmt::Block(block_stmt) => self.compile_block_stmt(block_stmt),
            Stmt::Empty => Ok(Bytecode::new()),
            Stmt::Debugger => Err(CompilerError::are_unsupported("Debugger statements")),
            Stmt::With(_) => Err(CompilerError::are_unsupported("'with' statements")),
            Stmt::Return(ret) => self.compile_return_stmt(ret),
            Stmt::Labeled(labeled_stmt) => self.compile_label_stmt(labeled_stmt),
            Stmt::Break(break_stmt) => self.compile_break_stmt(break_stmt),
            Stmt::Continue(continue_stmt) => self.compile_continue_stmt(continue_stmt),
            Stmt::If(if_stmt) => self.compile_if_stmt(if_stmt),
            Stmt::Switch(_) => Err(CompilerError::are_unsupported("'switch' statements")),
            Stmt::Throw(throw_expr) => self.compile_throw_stmt(throw_expr),
            Stmt::Try(try_stmt) => self.compile_try_stmt(try_stmt),
            Stmt::While(while_stmt) => self.compile_while_stmt(while_stmt),
            Stmt::DoWhile(dowhile_stmt) => self.compile_dowhile_stmt(dowhile_stmt),
            Stmt::For(for_stmt) => self.compile_for_stmt(for_stmt),
            Stmt::ForIn(_) => Err(CompilerError::are_unsupported("for-in statements")),
            Stmt::ForOf(_) => Err(CompilerError::are_unsupported("for-of statements")),
            Stmt::Var(decls) => self.compile_var_decl(&VariableKind::Var, &decls),
        }
    }

    fn compile_block_stmt(&mut self, block_stmt: &BlockStmt) -> BytecodeResult {
        self.scopes.enter_new_block_scope()?;
        let maybe_bc = block_stmt.iter().map(|part| self.compile_program_part(part)).collect();
        self.scopes.leave_current_block_scope()?;

        maybe_bc
    }

    fn compile_return_stmt(&mut self, ret: &Option<Expr>) -> BytecodeResult {
        let used_decl_regs: Vec<Reg> = self.scopes.current_scope()?.used_decls.iter()
                                            .map(|used_decl| used_decl.register).collect();

        let (bytecode, ret_reg) = match ret {
            Some(ret_expr) => {
                let (bytecode, ret_reg) = self.maybe_compile_expr(ret_expr, None)?;
                (bytecode, ret_reg)
            },
            None => (Bytecode::new(), self.isa.common_literal_reg(&CommonLiteral::Void0))
        };

        Ok(bytecode
            .add(Operation::new(Instruction::ReturnBytecodeFunc,
                                vec![Operand::Reg(ret_reg), Operand::RegistersArray(used_decl_regs)]))
        )
    }

    fn compile_label_stmt(&mut self, labeled: &LabeledStmt) -> BytecodeResult {
        self.label_generator.current_js_label = Some(labeled.label.clone()); // TODO no clone

        self.compile_stmt(labeled.body.borrow())
    }

    fn try_get_block_with_maybe_js_label(&self, js_label: &Option<Identifier>) -> CompilerResult<&LoopBlock> {
        match js_label {
            Some(label) => {
                if let Some(loop_block) = self.label_generator.get_js_labled_block(label) {
                    Ok(loop_block)
                } else {
                    Err(CompilerError::Custom("Used to unknown label".into()))
                }
            },
            None => {
                if let Some(loop_block) = self.label_generator.get_current_label_block() {
                    Ok(loop_block)
                } else {
                    Err(CompilerError::Custom("Used break/continue while not in a loop-block".into()))
                }
            }
        }
    }

    fn compile_break_stmt(&mut self, break_stmt: &Option<Identifier>) -> BytecodeResult {
        let maybe_block = self.try_get_block_with_maybe_js_label(break_stmt);

        maybe_block.map(|block| Bytecode::new().add(Operation::new(Instruction::Jump, vec![
            Operand::branch_addr(block.end_label())])))
    }

    fn compile_continue_stmt(&mut self, continue_stmt: &Option<Identifier>) -> BytecodeResult {
        let maybe_block = self.try_get_block_with_maybe_js_label(continue_stmt);

        maybe_block.map(|block| Bytecode::new().add(Operation::new(Instruction::Jump, vec![
            Operand::branch_addr(block.start_label())])))
    }

    fn compile_if_stmt(&mut self, if_stmt: &IfStmt) -> BytecodeResult {
        let (test_bytecode, test_reg) = self.maybe_compile_expr(&if_stmt.test, None)?;

        let if_branch_end_label = self.label_generator.generate_label();
        let else_branch_end_label = self.label_generator.generate_label();

        let if_branch_bc = self.compile_stmt(if_stmt.consequent.borrow())?;

        let bytecode = test_bytecode
                .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(test_reg), Operand::branch_addr(if_branch_end_label)]))
                .add_bytecode(if_branch_bc);

        if let Some(else_branch) = if_stmt.alternate.borrow() {
            let else_branch_bc = self.compile_stmt(&else_branch.borrow())?;
            //If-Else
            Ok(bytecode
                .add(Operation::new(Instruction::Jump, vec![Operand::branch_addr(else_branch_end_label)]))
                .add_label(if_branch_end_label)
                .add_bytecode(else_branch_bc)
                .add_label(else_branch_end_label)
            )
        } else {
            // If
            Ok(bytecode
                .add_label(if_branch_end_label))
        }
    }

    fn compile_throw_stmt(&mut self, throw_expr: &Expr) -> BytecodeResult {
        let (bc, reg) = self.maybe_compile_expr(throw_expr, None)?;

        Ok(bc.add(Operation::new(Instruction::Throw, vec![Operand::Reg(reg)])))
    }

    fn compile_try_stmt(&mut self, try_stmt: &TryStmt) -> BytecodeResult {
        let try_block_bc = self.compile_block_stmt(&try_stmt.block)?;
        let trash_reg = self.isa.reserved_reg(&ReservedeRegister::TrashRegister);
        let (catch_block_bc, catch_reg) = try_stmt.handler.as_ref()
                                            .map(|h| self.compile_catch_clause(&h))
                                            .unwrap_or_else(|| Ok((Bytecode::new(), trash_reg)))?;
        let final_block_bc = try_stmt.finalizer.as_ref()
                                            .map(|b| self.compile_block_stmt(&b))
                                            .unwrap_or_else(|| Ok(Bytecode::new()))?;

        let catch_block_label = self.label_generator.generate_label();
        let finally_start_label = self.label_generator.generate_label();

        let stop_prog_flow_op = Operation::new(Instruction::LoadLongNum, vec![
            Operand::Reg(self.isa.reserved_reg(&ReservedeRegister::BytecodePointer)),
            Operand::BytecodeEnd]);

        Ok(Bytecode::new()
            .add(Operation::new(Instruction::Try, vec![
                Operand::Reg(catch_reg),
                Operand::branch_addr(catch_block_label),
                Operand::branch_addr(finally_start_label),
            ]))
            .add_bytecode(try_block_bc)
            .add(stop_prog_flow_op.clone())
            .add_label(catch_block_label)
            .add_bytecode(catch_block_bc)
            .add(stop_prog_flow_op.clone())
            .add_label(finally_start_label)
            .add_bytecode(final_block_bc)
            .add(stop_prog_flow_op)
        )
    }

    fn compile_catch_clause(&mut self, catch_clause: &CatchClause) -> CompilerResult<(Bytecode, Register)> {
        self.scopes.enter_new_scope()?;

        let reg = if let Some(param) = &catch_clause.param {
            if let Pat::Identifier(ident) = param {
                self.scopes.add_decl(ident.to_string(),
                        DeclarationType::Variable(MyVariableKind::from(&VariableKind::Let)))?
            } else {
                return Err(CompilerError::are_unsupported("Catch patterns other than an identifier".into()));
            }
        } else {
            self.isa.reserved_reg(&ReservedeRegister::TrashRegister)
        };

        let body_bc = self.compile_block_stmt(&catch_clause.body)?;

        self.scopes.leave_current_scope()?;

        Ok((body_bc, reg))
    }

    fn compile_while_stmt(&mut self, while_stmt: &WhileStmt) -> BytecodeResult {
        let (test_bc, test_reg) = self.maybe_compile_expr(&while_stmt.test, None)?;

        let while_block = self.label_generator.generate_loop_label_block();
        let while_cond_label = while_block.start_label();
        let while_end_label = while_block.end_label();

        Ok(test_bc
            .add_label(while_cond_label)
            .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(test_reg), Operand::branch_addr(while_end_label)]))
            .add_bytecode(self.compile_stmt(while_stmt.body.borrow())?)
            .add(Operation::new(Instruction::Jump, vec![Operand::branch_addr(while_cond_label)]))
            .add_label(while_end_label))
    }

    fn compile_dowhile_stmt(&mut self, dowhile_stmt: &DoWhileStmt) -> BytecodeResult {
        let body_bc = self.compile_stmt(dowhile_stmt.body.borrow())?;
        let (test_bc, test_reg) = self.maybe_compile_expr(&dowhile_stmt.test, None)?;

        let dowhile_block = self.label_generator.generate_loop_label_block();
        let dowhile_start_label = dowhile_block.start_label();

        Ok(Bytecode::new()
            .add_label(dowhile_start_label)
            .add_bytecode(body_bc)
            .add_bytecode(test_bc)
            .add(Operation::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(dowhile_start_label)]))
            .add_label(dowhile_block.end_label()))
    }

    fn compile_for_stmt(&mut self, for_stmt: &ForStmt) -> BytecodeResult {
        let init_bc = match &for_stmt.init {
            Some(loop_init) => match loop_init {
                LoopInit::Variable(kind, decls) => self.compile_var_decl(&kind, &decls)?,
                LoopInit::Expr(expr) => self.maybe_compile_expr(&expr, None)?.0
            },
            None => Bytecode::new()
        };

        let for_block = self.label_generator.generate_loop_label_block();
        let loop_start_label = for_block.start_label();
        let loop_end_label = for_block.end_label();

        let test_bc = match &for_stmt.test {
            Some(test_expr) => {
                let (test_bc, test_reg) = self.maybe_compile_expr(&test_expr, None)?;

                test_bc
                    .add(Operation::new(Instruction::JumpCondNeg,
                            vec![Operand::Reg(test_reg), Operand::branch_addr(loop_end_label)]))
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
            .add_bytecode(test_bc)
            .add_bytecode(body_bc)
            .add_bytecode(update_bc)
            .add(Operation::new(Instruction::Jump, vec![Operand::branch_addr(loop_start_label)]))
            .add_label(loop_end_label))
    }

    fn maybe_compile_expr(&mut self, expr: &Expr, target_reg: Option<Register>) -> CompilerResult<(Bytecode, Register)> {
        let opt_reg = match expr {
            Expr::Ident(ident) => match self.scopes.get_var(ident) {
                Ok(var) => Some(var.register),
                Err(_) => self.decl_dependencies.try_get_dep(ident).map(|&reg| reg)
            },
            Expr::Literal(lit) => {
                match self.scopes.get_lit_decl(&BytecodeLiteral::from_lit(lit.clone())?) {
                    Ok(lit_decl) => Some(lit_decl.register),
                    Err(_) => None
                }
            }
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
            _ => None
        };

        let (opt_bytecode, target_reg) = match opt_reg {
            Some(reg) => match target_reg {
                Some(tar_reg) => (Some(self.compile_operand_assignment(tar_reg, Operand::Reg(reg))?), tar_reg),
                None => (Some(Bytecode::new()), reg)
            },
            None => match target_reg {
                Some(tar_reg) => (None, tar_reg),
                None => (None, self.scopes.reserve_register()?)
            }
        };

        let bytecode = match opt_bytecode {
            Some(bc) => bc,
            None => self.compile_expr(expr, target_reg)?
        };

        Ok((bytecode, target_reg))
    }

    fn compile_expr(&mut self, expr: &Expr, target_reg: Reg) -> BytecodeResult {
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
            Expr::Ident(ident) => self.compile_identifier_expr(ident, target_reg),
            Expr::Literal(lit) => self.compile_literal_expr(lit, target_reg),
            Expr::Logical(logical) => self.compile_logical_expr(logical, target_reg),
            Expr::Member(member) => self.compile_member_expr_access(member, target_reg),
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
            .add(Operation::new(Instruction::LoadArray, vec![Operand::Reg(target_reg), Operand::RegistersArray(regs)]))
        )
    }

    fn compile_assignment_expr(&mut self, assign: &AssignmentExpr, _target_reg: Reg) -> BytecodeResult {
        let ((left_bc, left_reg), maybe_prop_reg) = match &assign.left {
            AssignmentLeft::Pat(_) => { return Err(CompilerError::are_unsupported("Patterns in assignments")); },
            AssignmentLeft::Expr(expr) => match expr.borrow() {
                Expr::Member(member) => {
                    let (member_bc, obj_reg, prop_reg) = self.compile_member_expr(member)?;
                    ((member_bc, obj_reg), Some(prop_reg))
                },
                _ => (self.maybe_compile_expr(&expr, None)?, None)
            }
        };

        match assign.operator {
            AssignmentOperator::Equal => {
                if let Some(prop_reg) = maybe_prop_reg {
                    let (value_bc, value_reg) = self.maybe_compile_expr(assign.right.borrow(), None)?;
                    Ok(left_bc
                        .add_bytecode(value_bc)
                        .add(Operation::new(Instruction::PropertySet,
                                vec![Operand::Reg(left_reg), Operand::Reg(prop_reg), Operand::Reg(value_reg)])))
                } else {
                    Ok(left_bc.add_bytecode(self.compile_expr(assign.right.borrow(), left_reg)?))
                }
            }
            _ => {
                let (right_bc, right_reg) = self.maybe_compile_expr(assign.right.borrow(), None)?;
                Ok(left_bc.add_bytecode(right_bc)
                    .add(self.isa.assignment_op(&assign.operator, left_reg, right_reg)))
            }
        }
    }

    fn compile_binary_expr(&mut self, bin: &BinaryExpr, target_reg: Reg) -> BytecodeResult {
        let (left_bc, left_reg) = self.maybe_compile_expr(bin.left.borrow(), None)?;
        let (right_bc, right_reg) = self.maybe_compile_expr(bin.right.borrow(), None)?;

        Ok(left_bc
            .add_bytecode(right_bc)
            .add(self.isa.binary_op(&bin.operator, target_reg, left_reg, right_reg)?)
        )
    }

    fn compile_call_expr(&mut self, call: &CallExpr, target_reg: Reg) -> BytecodeResult {
        match call.callee.borrow() {
            Expr::Ident(ident) => {
                if self.functions.iter().any(|func| func.ident == *ident) {
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
            .add(Operation::new(Instruction::JumpCond, vec![Operand::Reg(test_reg), Operand::branch_addr(after_alt_label)]))
            .add_bytecode(consequent_bc)
            .add(Operation::new(Instruction::Jump, vec![Operand::branch_addr(after_cons_label)]))
            .add_label(after_alt_label)
            .add_bytecode(alt_bc)
            .add_label(after_cons_label))
    }

    fn compile_bytecode_func_call(&mut self, func: String, args: &[Expr], target_reg: Reg) -> BytecodeResult {
        let (args_bytecode, arg_regs): (Vec<Bytecode>, Vec<Reg>) = args.iter().map(|arg_expr| {
            self.maybe_compile_expr(arg_expr, None)
        }).collect::<CompilerResult<Vec<(Bytecode, Reg)>>>()?.into_iter().unzip();

        Ok(args_bytecode.into_iter().collect::<Bytecode>()
            .add(Operation::new(Instruction::CallBytecodeFunc,
                                vec![Operand::function_addr(func),
                                     Operand::Reg(target_reg),
                                     Operand::bc_func_args(arg_regs)])))
    }

    fn compile_extern_func_call(&mut self, call: &CallExpr, target_reg: Reg) -> BytecodeResult {
        let (callee_bc, callee_reg) = self.maybe_compile_expr(&call.callee, None)?;

        let (callee_this_bc, callee_this_reg) =
            if let Expr::Member(member_expr) = call.callee.borrow() {
                self.maybe_compile_expr(&member_expr.object, None)?
            } else {
                (Bytecode::new(), self.isa.common_literal_reg(&CommonLiteral::Void0))
            };

        let (bytecode, arg_regs): (Vec<Bytecode>, Vec<Reg>) = call.arguments.iter().map(|arg| {
            self.maybe_compile_expr(arg, None)
        }).collect::<CompilerResult<Vec<(Bytecode, Reg)>>>()?.into_iter().unzip();

        Ok(bytecode.into_iter().collect::<Bytecode>()
            .add_bytecode(callee_bc)
            .add_bytecode(callee_this_bc)
            .add(Operation::new(Instruction::CallFunc, vec![
                    Operand::Reg(target_reg),
                    Operand::Reg(callee_reg),
                    Operand::Reg(callee_this_reg),
                    Operand::RegistersArray(arg_regs)
                ]
        )))
    }

    fn compile_operand_assignment(&self, left: Reg, right: Operand) -> BytecodeResult {
        Ok(Bytecode::new().add(self.isa.load_op(left, right)))
    }

    fn compile_identifier_expr(&mut self, ident: &Identifier, target_reg: Reg) -> BytecodeResult {
        match self.scopes.get_var(&ident).map(|decl| decl.clone()) {
            Ok(decl) => self.compile_operand_assignment(target_reg, Operand::Reg(decl.register)),
            Err(_) => match self.functions.iter().find(|func| func.ident == *ident) {
                Some(func) => {
                    Ok(Bytecode::new()
                        .add(Operation::new(Instruction::BytecodeFuncCallback, vec![
                            Operand::Reg(target_reg),
                            Operand::function_addr(ident.clone()),
                            Operand::RegistersArray(func.arguments.clone())])))
                },
                None => {
                    for i in 0..self.scopes.scopes.len()-1 {
                        self.scopes.scopes[i].try_reserve_specific_reg(target_reg)?;
                    }

                    self.decl_dependencies.add_decl_dep(ident.to_string(), target_reg);
                    Ok(Bytecode::new())
                },
            }
        }
    }

    fn compile_literal_expr(&mut self, lit: &Literal, target_reg: Reg) -> BytecodeResult {
        let operand = Operand::from_literal(BytecodeLiteral::from_lit(lit.clone())?)?;
        // This feature is currenlty disabled
        if false { // operand.is_worth_caching()
            self.scopes.add_lit_decl(BytecodeLiteral::from_lit(lit.clone())?, target_reg)?;
        }

        self.compile_operand_assignment(target_reg, operand)
    }

    fn compile_logical_expr(&mut self, logical: &LogicalExpr, target_reg: Reg) -> BytecodeResult {
        let (left_bc, _) = self.maybe_compile_expr(logical.left.borrow(), Some(target_reg))?;
        let (right_bc, _) = self.maybe_compile_expr(logical.right.borrow(), Some(target_reg))?;

        let after_right_label = self.label_generator.generate_label();

        match logical.operator {
            LogicalOperator::And => Ok(left_bc
                .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(target_reg), Operand::branch_addr(after_right_label)]))
                .add_bytecode(right_bc)
                .add_label(after_right_label)),
            LogicalOperator::Or => Ok(left_bc
                .add(Operation::new(Instruction::JumpCond, vec![Operand::Reg(target_reg), Operand::branch_addr(after_right_label)]))
                .add_bytecode(right_bc)
                .add_label(after_right_label))
        }

    }

    fn compile_member_expr(&mut self, member: &MemberExpr) -> CompilerResult<(Bytecode, Reg, Reg)> {
        let (obj_bc, obj_reg) = self.maybe_compile_expr(member.object.borrow(), None)?;
        let (prop_bc, prop_reg) = if member.computed {
            self.maybe_compile_expr(member.property.borrow(), None)?
        } else {
            match member.property.borrow() {
                Expr::Ident(ident) => self.maybe_compile_expr(&Expr::Literal(Literal::String(format!("\"{}\"", ident))), None)?,
                _ => self.maybe_compile_expr(member.property.borrow(), None)?
            }
        };

        Ok((obj_bc.add_bytecode(prop_bc), obj_reg, prop_reg))
    }

    fn compile_member_expr_access(&mut self, member: &MemberExpr, target_reg: Reg) -> BytecodeResult {
        let (member_bc, obj_reg, prop_reg) = self.compile_member_expr(member)?;

        Ok(member_bc
            .add(Operation::new(Instruction::PropAccess, vec![
                    Operand::Reg(target_reg), Operand::Reg(obj_reg), Operand::Reg(prop_reg)
                ]
            )))
    }

    fn compile_update_expr(&mut self, update: &UpdateExpr, _target_reg: Reg) -> BytecodeResult {
        if update.prefix {
            let (arg_bc, arg_reg) = self.maybe_compile_expr(update.argument.borrow(), None)?;
            Ok(arg_bc.add(self.isa.update_op(&update.operator, arg_reg)))
        } else {
            Err(CompilerError::are_unsupported("suffix update expressions"))
        }
    }

    fn compile_unary_expr(&mut self, unary: &UnaryExpr, target_reg: Reg) -> BytecodeResult {
        if unary.prefix {
            if UnaryOperator::Void == unary.operator {
                let (arg_bc, _) = self.maybe_compile_expr(unary.argument.borrow(), None)?;
                let void0_reg = self.isa.common_literal_reg(&CommonLiteral::Void0);
                Ok(arg_bc
                    .add_bytecode(self.compile_operand_assignment(target_reg, Operand::Reg(void0_reg))?))
            } else {
                let (arg_bc, arg_reg) = self.maybe_compile_expr(unary.argument.borrow(), None)?;
                Ok(arg_bc.add(self.isa.unary_op(&unary.operator, target_reg, arg_reg)?))
            }
        } else {
            Err(CompilerError::are_unsupported("suffix unary expressions"))
        }
    }

    fn compile_func(&mut self, func: &Function) -> BytecodeResult {
        if func.generator || func.is_async {
            return Err(CompilerError::are_unsupported("generator and async functions"))
        }

        let func_ident =  match &func.id {
            Some(ident) => ident.to_string(),
            None => { return Err(CompilerError::are_unsupported("anonymous functions")); }
        };

        self.scopes.enter_new_scope()?;

        let arg_regs = func.params.iter().map(|param| {
            match param {
                FunctionArg::Expr(expr) => match expr {
                    Expr::Ident(ident) => self.scopes.add_decl(ident.to_string(), DeclarationType::Function),
                    _ => Err(CompilerError::Custom("Only identifiers are accepted as function arguments".into()))
                },
                FunctionArg::Pat(pat) => match pat {
                    Pat::Identifier(ident) => self.scopes.add_decl(ident.to_string(), DeclarationType::Function),
                    _ => Err(CompilerError::Custom("Only identifiers are accepted as function arguments".into()))
                }
            }
        }).collect::<CompilerResult<Vec<Register>>>()?;

        self.functions.push(BytecodeFunction::new_phantom(func_ident, arg_regs));

        let mut func_bc = func.body.iter().map(|part| self.compile_program_part(&part))
                                   .collect::<BytecodeResult>()?;

        if !func_bc.last_op_is_return() {
            func_bc = func_bc.add_bytecode(self.compile_return_stmt(&None)?)
        }

        let func_scope = self.scopes.leave_current_scope()?;
        let used_decls = func_scope.used_decls.into_iter().map(|used_decl| used_decl.register).collect();

        // It is save to unwrap here since it was definitly pushed above
        let phantom_func = self.functions.pop().unwrap();
        self.functions.push(BytecodeFunction::from_phantom(phantom_func, func_bc, used_decls));

        Ok(Bytecode::new())
    }

    fn finalize_label_addresses(&self, mut bc: Bytecode, offset: usize) -> BytecodeResult {
        let mut offset_counter = offset;
        let label_offsets: HashMap<Label, usize> = bc.elements.iter().filter_map(|element| {
            match element {
                BytecodeElement::Operation(cmd) => {offset_counter += cmd.length_in_bytes(); None},
                BytecodeElement::Label(label) => Some((label.clone(), offset_counter.clone()))
            }
        }).collect();

        let total_bc_len_operand = Operand::LongNum(bc.length_in_bytes() as i32);

        for cmd in bc.commands_iter_mut() {
            for op in cmd.operands.iter_mut() {
                if let Operand::BranchAddr(token) = op {
                    *op = Operand::LongNum(*label_offsets.get(&token.label).ok_or(
                        CompilerError::Custom(format!("Found unknown label {}", token.label))
                    )? as i32);
                } else if let Operand::BytecodeEnd = op {
                    *op = total_bc_len_operand.clone()
                }
            }
        }

        Ok(bc)
    }

    fn finalize_function_bytescodes(&self, main: Bytecode) -> BytecodeResult {
        let mut functions_and_offsets: HashMap<String, (usize, &BytecodeFunction)> = HashMap::new();
        let mut offset_counter = main.length_in_bytes();

        let functions_bytecode = self.functions.iter().map(|func| -> BytecodeResult {
            functions_and_offsets.insert(func.ident.to_string(), (offset_counter, func));

            let func_bc = func.bytecode.clone().expect("Found phantom function defintion");
            let finalized_func_bc = self.finalize_label_addresses(func_bc, offset_counter)?;
            offset_counter += finalized_func_bc.length_in_bytes();

            Ok(finalized_func_bc)
        }).collect::<BytecodeResult>()?;

        let mut complete_bytecode = main.add_bytecode(functions_bytecode);

        // Patch bytecode function argument lists
        for cmd in complete_bytecode.commands_iter_mut() {
            if let Instruction::CallBytecodeFunc = cmd.instruction {
                let target_func = cmd.operands.get(0).expect("Failed to retrieve bytecode functions token");
                let args = cmd.operands.get(2).expect("Failed to retrieve bytecode functions argument list");

                let func = match target_func {
                    Operand::FunctionAddr(token) => functions_and_offsets.get(&token.ident).ok_or(
                        CompilerError::Custom(format!("Found unknown function ident {}", token.ident))
                    )?.1,
                    _ => { return Err(CompilerError::Custom(
                        "Bytecode function name should be a function address token".into())) }
                };

                if let Operand::FunctionArguments(arg_regs) = args {
                    cmd.operands[2] = Operand::RegistersArray(
                        func.arguments.iter().zip(arg_regs.args.iter()).map(|(&a, &b)| vec![a, b]).flatten().collect()
                    );
                } else {
                    return Err(CompilerError::Custom(
                        "Bytecode function argument should be a bytecode func args placeholder".into()))
                }
            }
        }

        // Replace function tokens (function names) with their corresponding bytecode offset
        for cmd in complete_bytecode.commands_iter_mut() {
            for op in cmd.operands.iter_mut() {
                if let Operand::FunctionAddr(token) = op {
                    *op = Operand::LongNum(functions_and_offsets.get(&token.ident).ok_or(
                        CompilerError::Custom(format!("Found unknown function ident {}", token.ident))
                    )?.0 as i32);
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
        Bytecode::new().add(Operation::new(Instruction::Copy,
            vec![Operand::Reg(test_expr_ident.scopes.get_var("testVar".into()).unwrap().register),
                 Operand::Reg(test_expr_ident_reg)])));

     let mut test_expr_str_lit = BytecodeCompiler::new();
     assert_eq!(test_expr_str_lit.compile_var_decl(&VariableKind::Var, &vec![
             VariableDecl{id: Pat::Identifier("testVar".into()), init: Some(Expr::Literal(Literal::String("\"TestString\"".into())))}
         ]).unwrap(),
         Bytecode::new().add(Operation::new(Instruction::LoadString,
             vec![Operand::Reg(test_expr_str_lit.scopes.get_var("testVar".into()).unwrap().register),
                  Operand::String("TestString".into())])));
}
