use crate::bytecode::*;
use crate::scope::{Reg, Scope, Scopes};
use crate::error::{CompilerError, CompilerResult};

pub use resast::prelude::*;


macro_rules! make_enum_helper{
    (enum $name:ident { $($variants:ident),* }) =>
    (
        #[derive(Clone, PartialEq, Debug)]
        pub enum $name {
            $($variants,)*
            __VarinatsCountHelper__
        }
        impl $name {
            pub const fn enum_size() -> usize {
                $name::__VarinatsCountHelper__ as usize
            }

            pub fn enum_iterator() -> std::slice::Iter<'static, $name> {
                const VARINTS: [$name; $name::enum_size()] = [$($name::$variants,)*];
                VARINTS.into_iter()
            }

            pub fn variant_index(&self) -> usize {
                self.clone() as usize
            }
        }
    )
}

#[test]
fn test_make_enum_helper() {
    make_enum_helper!(
        enum TestEnumA {
            A,
            B,
            C
        }
    );

    assert_eq!(TestEnumA::enum_size(), 3);

    let mut iter = TestEnumA::enum_iterator();

    assert_eq!(iter.next(), Some(&TestEnumA::A));
    assert_eq!(iter.next(), Some(&TestEnumA::B));
    assert_eq!(iter.next(), Some(&TestEnumA::C));
    assert_eq!(iter.next(), None);

    assert_eq!(TestEnumA::A.variant_index(), 0);
    assert_eq!(TestEnumA::B.variant_index(), 1);
    assert_eq!(TestEnumA::C.variant_index(), 2);

    make_enum_helper!(
        enum TestEnumB {
            A
        }
    );

    assert_eq!(TestEnumB::enum_size(), 1);

    let mut iter = TestEnumB::enum_iterator();

    assert_eq!(iter.next(), Some(&TestEnumB::A));
    assert_eq!(iter.next(), None);

    assert_eq!(TestEnumB::A.variant_index(), 0);
}


make_enum_helper!(
enum CommonLiteral
{
    Num0,
    Num1,
    Void0 // Undefined
    // EmptyString
});

impl CommonLiteral {
    pub fn to_literal(&self) -> BytecodeLiteral {
        match &self {
            CommonLiteral::Num0 => BytecodeLiteral::IntNumber(0),
            CommonLiteral::Num1 => BytecodeLiteral::IntNumber(1),
            CommonLiteral::Void0 => BytecodeLiteral::Null,
            _ => panic!("")
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
        Ok(CommonLiteralRegs {
            regs: (0..CommonLiteral::enum_size()).map(|_| scope.reserve_register_back()).collect::<CompilerResult<Vec<Reg>>>()?
        })
    }

    pub fn add_to_lit_cache(&self, scopes: &mut Scopes) -> CompilerResult<()> {
        for common_lit in CommonLiteral::enum_iterator() {
            scopes.add_lit_decl(common_lit.to_literal(), self.regs[common_lit.variant_index()])?;
        }

        Ok(())
    }

    pub fn reg(&self, common_lit: &CommonLiteral) -> Reg {
        self.regs[common_lit.variant_index()]
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
            AssignmentOperator::DivEqual => Instruction::Div,
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
            UnaryOperator::Void => { return Err(CompilerError::Custom("The 'void' must be handled on compiler-level".into())); },
            // Delete,
            _ => { return Err(CompilerError::is_unsupported("Unary operation", op)); }
        })
    }

    pub fn binary_op(&self, op: &BinaryOperator, rd: Reg, r0: Reg, r1: Reg) -> CompilerResult<Command> {
        let instr = match op {
            BinaryOperator::Equal => Instruction::CompEqual,
            BinaryOperator::NotEqual => Instruction::CompNotEqual,
            BinaryOperator::StrictEqual => Instruction::CompStrictEqual,
            BinaryOperator::StrictNotEqual => Instruction::CompStrictNotEqual,
            BinaryOperator::LessThan => Instruction::CompLessThan,
            BinaryOperator::GreaterThan => Instruction::CompGreaterThan,
            BinaryOperator::LessThanEqual => Instruction::CompLessThanEqual,
            BinaryOperator::GreaterThanEqual => Instruction::CompGreaterThanEqual,
            // BinaryOperator::LeftShift => Instruction::Sh,
            // BinaryOperator::RightShift,
            // BinaryOperator::UnsignedRightShift,
            BinaryOperator::Plus => Instruction::Add,
            BinaryOperator::Minus => Instruction::Minus,
            BinaryOperator::Times => Instruction::Mul,
            BinaryOperator::Over => Instruction::Div,
            // Mod,
            // Or,
            // XOr,
            // And,
            // In,
            // InstanceOf,
            // PowerOf,
            _ => { return Err(CompilerError::is_unsupported("Binary operation", op)); }
        };

        Ok(Command::new(instr, vec![Operand::Reg(rd), Operand::Reg(r0), Operand::Reg(r1)]))
    }
}
