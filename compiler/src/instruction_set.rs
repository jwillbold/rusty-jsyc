use crate::bytecode::*;
use crate::scope::Reg;

pub use resast::prelude::*;


#[derive(Clone)]
pub struct InstructionSet
{

}

impl InstructionSet {
    pub fn default() -> Self {
        InstructionSet {

        }
    }

    pub fn load_op(&self, left: Reg, right: Operand) -> Command {
        let instruction = match right {
            Operand::String(_) => Instruction::LoadString,
            Operand::FloatNum(_) => Instruction::LoadFloatNum,
            Operand::LongNum(_) => Instruction::LoadLongNum,
            Operand::ShortNum(_) => Instruction::LoadNum,
            Operand::Register(_) => Instruction::Copy,
            Operand::RegistersArray(_) => unimplemented!("Register Arrays are not yet implement as seperte load operation"),
            Operand::SubstituteToken(_) => unimplemented!("...")
        };

        Command::new(instruction, vec![Operand::Register(left), right])
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

        Command::new(instr, vec![Operand::Register(rd), Operand::Register(rd), Operand::Register(rs)])
    }

    pub fn update_op(&self, op: &UpdateOperator, rd: Reg) -> Command {
        let instr = match op {
            UpdateOperator::Increment => Instruction::Add,
            UpdateOperator::Decrement => Instruction::Minus,
        };

        // TODO
        Command::new(instr, vec![Operand::Register(rd), Operand::Register(rd), Operand::Register(255)])
    }

    pub fn unary_op(&self, op: &UnaryOperator, rd: Reg, rs: Reg) -> Command {
        // match op {
            // Minus,
            // Plus,
            // Not,
            // Tilde,
            // TypeOf,
            // Void,
            // Delete,
        // }
        unimplemented!("unary operations")
    }
}