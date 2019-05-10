use crate::bytecode::*;
use crate::scope::{Reg, Scope};
use crate::error::{CompilerError, CompilerResult};

pub use resast::prelude::*;

#[derive(Clone)]
pub enum CommonLiteral
{
    Num0,
    Num1,
    Void0 // Undefined
    // EmptyString
}

impl CommonLiteral {
    pub fn idx(&self) -> usize {
        match self {
            CommonLiteral::Num0 => 0,
            CommonLiteral::Num1 => 1,
            CommonLiteral::Void0 => 2,
        }
    }
}

#[derive(Clone)]
pub struct CommonLiteralRegs
{
    regs: Vec<Reg>
}

impl CommonLiteralRegs {
    pub fn new(scope: &mut Scope) -> CompilerResult<Self> {
        // This construct is a reminder, that will fail to compile if the enum CommonLiteral
        // is changed without adjusting this enum_size. This it will be almost impossible
        // to forget changing this enum_size when changing the num above
        // Rust 1.34.0 completly optmizes this out.
        let reminder = CommonLiteral::Num0;
        let enum_size = match reminder {
            CommonLiteral::Num0 | CommonLiteral::Num1 | CommonLiteral::Void0 => 3
        };

        Ok(CommonLiteralRegs {
            regs: (0..enum_size).map(|_| scope.reserve_register_back()).collect::<CompilerResult<Vec<Reg>>>()?
        })
    }

    pub fn reg(&self, common_lit: &CommonLiteral) -> Reg {
        self.regs[common_lit.idx()]
    }
}

#[derive(Clone)]
pub struct InstructionSet
{
    common_regs: CommonLiteralRegs
}

impl InstructionSet {
    pub fn default(scope: &mut Scope) -> Self {
        InstructionSet {
            common_regs: CommonLiteralRegs::new(scope).unwrap()
        }
    }

    pub fn common_lits(&self) -> &CommonLiteralRegs {
        &self.common_regs
    }

    pub fn common_literal_reg(&self, common_lit: &CommonLiteral) -> Reg {
        self.common_regs.reg(common_lit)
    }

    pub fn load_op(&self, left: Reg, right: Operand) -> Command {
        let instruction = match right {
            Operand::String(_) => Instruction::LoadString,
            Operand::FloatNum(_) => Instruction::LoadFloatNum,
            Operand::LongNum(_) => Instruction::LoadLongNum,
            Operand::ShortNum(_) => Instruction::LoadNum,
            Operand::Reg(_) => Instruction::Copy,
            Operand::RegistersArray(_) => unimplemented!("Register Arrays are not yet implement as seperte load operation"),
            Operand::FunctionAddr(_) |
            Operand::BranchAddr(_) => unimplemented!("...")
        };

        Command::new(instruction, vec![Operand::Reg(left), right])
    }

    pub fn assignment_op(&self, op: &AssignmentOperator, rd: Reg, rs: Reg) -> Command {
        let instr = match op {
            AssignmentOperator::Equal => Instruction::Copy,
            AssignmentOperator::PlusEqual => Instruction::Add,
            AssignmentOperator::MinusEqual => Instruction::Minus,
            AssignmentOperator::TimesEqual => Instruction::Mul,
            // DivEqual,
            // ModEqual,
            // LeftShiftEqual,
            // RightShiftEqual,
            // UnsignedRightShiftEqual,
            // OrEqual,
            // XOrEqual,
            // AndEqual,
            // PowerOfEqual,
            _ => unimplemented!("The correct branch for the assignment op ist not yet implemented")
        };

        Command::new(instr, vec![Operand::Reg(rd), Operand::Reg(rd), Operand::Reg(rs)])
    }

    pub fn update_op(&self, op: &UpdateOperator, rd: Reg) -> Command {
        let instr = match op {
            UpdateOperator::Increment => Instruction::Add,
            UpdateOperator::Decrement => Instruction::Minus,
        };

        Command::new(instr, vec![
            Operand::Reg(rd),
            Operand::Reg(rd),
            Operand::Reg(self.common_literal_reg(&CommonLiteral::Num1))
            ]
        )
    }

    pub fn unary_op(&self, op: &UnaryOperator, rd: Reg, rs: Reg) -> CompilerResult<Command> {
        Ok(match op {
            UnaryOperator::Minus => Command::new(Instruction::Minus, vec![
                Operand::Reg(rd),
                Operand::Reg(self.common_literal_reg(&CommonLiteral::Num0)),
                Operand::Reg(rs)
                ]
            ),
            UnaryOperator::Plus => Command::new(Instruction::Add, vec![
                Operand::Reg(rd),
                Operand::Reg(self.common_literal_reg(&CommonLiteral::Num0)),
                Operand::Reg(rs)
                ]
            ),
            // Not,
            // Tilde,
            // TypeOf,
            // Void,
            // Delete,
            _ => { return Err(CompilerError::is_unsupported("Unary operation")); }
        })
    }

    pub fn binary_op(&self, op: &BinaryOperator, rd: Reg, r0: Reg, r1: Reg) -> CompilerResult<Command> {
        Ok(match op {
            // Equal,
            // NotEqual,
            // StrictEqual,
            // StrictNotEqual,
            // LessThan,
            // GreaterThan,
            // LessThanEqual,
            // GreaterThanEqual,
            // LeftShift,
            // RightShift,
            // UnsignedRightShift,
            BinaryOperator::Plus => Command::new(Instruction::Add, vec![
                    Operand::Reg(rd),
                    Operand::Reg(r0),
                    Operand::Reg(r1),
                ]
            ),
            // Minus,
            // Times,
            // Over,
            // Mod,
            // Or,
            // XOr,
            // And,
            // In,
            // InstanceOf,
            // PowerOf,
            _ => { return Err(CompilerError::is_unsupported("Binary operation")); }
        })
    }
}