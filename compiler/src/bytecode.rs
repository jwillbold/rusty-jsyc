use crate::error::{CompilerError, CompilerResult};
use crate::scope::Register;

use std::{u16};
use std::iter::FromIterator;
use resast::prelude::*;


pub type BytecodeResult = Result<Bytecode, CompilerError>;

/// Labels are used as targets of jumps
pub type Label = u32;

/// This trait is implemented by elements that are part of the final bytecode
pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;

    fn length_in_bytes(&self) -> usize {
        self.to_bytes().len()
    }
}

/// Represents the basics instructions known to this compiler
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction
{
    LoadString,
    LoadFloatNum,
    LoadLongNum,
    LoadNum,
    LoadArray,

    PropAccess,
    CallFunc,
    Eval,
    CallBytecodeFunc,
    ReturnBytecodeFunc,
    Copy,
    Exit,
    BytecodeFuncCallback,
    PropertySet,

    JumpCond,
    Jump,
    JumpCondNeg,

    CompEqual,
    CompNotEqual,
    CompStrictEqual,
    CompStrictNotEqual,
    CompLessThan,
    CompGreaterThan,
    CompLessThanEqual,
    CompGreaterThanEqual,

    Add,
    Minus,
    Mul,
    Div,
    // LeftShift
    // RightShift
    // Mod,
    // Or,
    // XOr,
    // And,
    // In,
}

impl Instruction {
    fn to_byte(&self) -> u8 {
        match self {
            Instruction::LoadString => 1,
            Instruction::LoadNum => 2,
            Instruction::LoadFloatNum => 3,
            Instruction::LoadLongNum => 4,
            Instruction::LoadArray => 5,

            Instruction::PropAccess => 10,
            Instruction::CallFunc => 11,
            Instruction::Eval => 12,
            Instruction::CallBytecodeFunc => 13,
            Instruction::ReturnBytecodeFunc => 14,
            Instruction::Copy => 15,
            Instruction::Exit => 16,
            Instruction::JumpCond => 17,
            Instruction::Jump => 18,
            Instruction::JumpCondNeg => 19,
            Instruction::BytecodeFuncCallback => 20,
            Instruction::PropertySet => 21,

            Instruction::CompEqual => 50,
            Instruction::CompNotEqual => 51,
            Instruction::CompStrictEqual => 52,
            Instruction::CompStrictNotEqual => 53,
            Instruction::CompLessThan => 54,
            Instruction::CompGreaterThan => 55,
            Instruction::CompLessThanEqual => 56,
            Instruction::CompGreaterThanEqual => 57,

            Instruction::Add => 100,
            Instruction::Minus => 102,
            Instruction::Mul => 101,
            Instruction::Div => 103,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Instruction::LoadString => "LoadString",
            Instruction::LoadNum => "LoadNum",
            Instruction::LoadFloatNum => "LoadFloatNum",
            Instruction::LoadLongNum => "LoadLongNum",
            Instruction::LoadArray => "LoadArray",

            Instruction::PropAccess => "PropAccess",
            Instruction::CallFunc => "CallFunc",
            Instruction::Eval => "Eval",
            Instruction::CallBytecodeFunc => "CallBytecodeFunc",
            Instruction::ReturnBytecodeFunc => "ReturnBytecodeFunc",
            Instruction::Copy => "Copy",
            Instruction::Exit => "Exit",
            Instruction::JumpCond => "JumpCond",
            Instruction::Jump => "Jump",
            Instruction::JumpCondNeg => "JumpCondNeg",
            Instruction::BytecodeFuncCallback => "BytecodeFuncCallback",
            Instruction::PropertySet => "PropertySet",

            Instruction::CompEqual => "CompEqual",
            Instruction::CompNotEqual => "CompNotEqual",
            Instruction::CompStrictEqual => "CompStrictEqual",
            Instruction::CompStrictNotEqual => "CompStrictNotEqual",
            Instruction::CompLessThan => "CompLessThan",
            Instruction::CompGreaterThan => "CompGreaterThan",
            Instruction::CompLessThanEqual => "CompLessThanEqual",
            Instruction::CompGreaterThanEqual => "CompGreaterThanEqual",

            Instruction::Add => "Add",
            Instruction::Minus => "Minus",
            Instruction::Mul => "Mul",
            Instruction::Div => "Div",
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct BytecodeAddrToken {
    pub ident: String
}

impl ToBytes for BytecodeAddrToken {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0; 4]
    }

    fn length_in_bytes(&self) -> usize {
        4
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LabelAddrToken {
    pub label: Label
}

impl ToBytes for LabelAddrToken {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0; 4]
    }

    fn length_in_bytes(&self) -> usize {
        4
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionArguments {
    pub args: Vec<Register>
}

impl ToBytes for FunctionArguments {
    fn to_bytes(&self) -> Vec<u8> {
        vec![0; self.args.len()]
    }

    fn length_in_bytes(&self) -> usize {
        1 + 2*self.args.len()
    }
}

/// Represents all literal types of JavaScript
///
/// ``var a = 1.5;`` => ``a => BytecodeLiteral::FloatNum(1.5)``
///
/// ``var b = 100;`` => ``b => BytecodeLiteral::IntNumber(100)``
///
/// # Note
/// JavaScript regex literals are not yet supported.
#[derive(Clone, Debug, PartialEq)]
pub enum BytecodeLiteral
{
    Null,
    String(String),
    FloatNum(f64),
    IntNumber(i64),
    Bool(bool),
    // RegEx(ressa::expr::RegEx)
}

impl BytecodeLiteral {
    pub fn from_lit(lit: Literal) -> CompilerResult<Self> {
        match lit {
            Literal::Null => Ok(BytecodeLiteral::Null),
            Literal::String(string) => Ok({
                if string.len() > 0 {
                    BytecodeLiteral::String(string[1..string.len()-1].to_string())
                } else {
                    BytecodeLiteral::String(string)
                }
            }),
            Literal::Number(num_string) => {
                if let Ok(dec_num) = num_string.parse::<i64>() {
                    Ok(BytecodeLiteral::IntNumber(dec_num))
                } else if let Ok(float_num) = num_string.parse::<f64>() {
                    Ok(BytecodeLiteral::FloatNum(float_num))
                } else if num_string.len() > 2 {
                    if &num_string[..2] == "0x" {
                        if let Ok(hex_num) = i64::from_str_radix(&num_string[2..], 16) {
                            Ok(BytecodeLiteral::IntNumber(hex_num))
                        } else {
                            Err(CompilerError::Custom(format!("Failed to parse hex-numeric literal '{}'", num_string)))
                        }
                    } else if &num_string[..2] == "0o" {
                        if let Ok(oct_num) = i64::from_str_radix(&num_string[2..], 8) {
                            Ok(BytecodeLiteral::IntNumber(oct_num))
                        } else {
                            Err(CompilerError::Custom(format!("Failed to parse oct-numeric literal '{}'", num_string)))
                        }
                    } else if &num_string[..2] == "0b" {
                        if let Ok(oct_num) = i64::from_str_radix(&num_string[2..], 2) {
                            Ok(BytecodeLiteral::IntNumber(oct_num))
                        } else {
                            Err(CompilerError::Custom(format!("Failed to parse bin-numeric literal '{}'", num_string)))
                        }
                    } else {
                        Err(CompilerError::Custom(format!("Failed to parse numeric literal '{}'", num_string)))
                    }
                } else {
                    Err(CompilerError::Custom(format!("Failed to parse numeric literal '{}'", num_string)))
                }
            },
            Literal::Boolean(b) => Ok(BytecodeLiteral::Bool(b)),
            Literal::RegEx(_) |
            Literal::Template(_) => Err(CompilerError::are_unsupported("regex and template literals"))
        }
    }
}

impl std::fmt::Display for BytecodeLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BytecodeLiteral::Null => write!(f, "Null"),
            BytecodeLiteral::String(string) => write!(f, "\"{}\"", string),
            BytecodeLiteral::FloatNum(float) => write!(f, "Float(){})", float),
            BytecodeLiteral::IntNumber(signed_int) => write!(f, "SignedInt({})", signed_int),
            BytecodeLiteral::Bool(bool) => write!(f, "Bool({})", bool),
        }
    }
}

/// Represents variants of bytecode operand
///
/// There are two types of operands. Regular operands(numbers, strings or registers) and token operands.
/// Token operands are only used by the compiler to express a dependency that cannot be calculated
/// at the point of declaration. For example, the target of a jump is an address. However, an address
/// can only be calculated when the entire bytecode is known. Thus, all jump instructions contain a token
/// operand [BranchAddr](enum.Operand.html#Operand::FunctionAddr) which holds the target [label](type.Label.html)
/// of the jump. After the compilation of this, these tokens are then replaced.
/// Thus, token operands cannot be part of a final bytecode.
#[derive(Debug, PartialEq, Clone)]
pub enum Operand
{
    String(String),
    FloatNum(f64),
    LongNum(i32),
    ShortNum(u8),
    Reg(u8),
    RegistersArray(Vec<u8>),

    FunctionAddr(BytecodeAddrToken),
    BranchAddr(LabelAddrToken),
    FunctionArguments(FunctionArguments),
}

impl Operand {
    pub fn from_literal(literal: BytecodeLiteral) -> CompilerResult<Self> {
        match literal {
            BytecodeLiteral::Null => Ok(Operand::Reg(253)), //TODO: Register of predefined void 0,
            BytecodeLiteral::String(string) => Ok(Operand::String(string)),
            BytecodeLiteral::FloatNum(float) => Ok(Operand::FloatNum(float)),
            BytecodeLiteral::IntNumber(int) => {
                if int <= 255 && int >= 0 {
                    Ok(Operand::ShortNum(int as u8))
                } else if int <= std::i32::MAX.into() && int >= std::i32::MIN.into() {
                    Ok(Operand::LongNum(int as i32))
                } else {
                    Err(CompilerError::Custom(
                        format!("Only integers from {} to {} are allowed. Consider using a float instead",
                        std::i32::MIN, std::i32::MAX)))
                }
            },
            BytecodeLiteral::Bool(bool) => Ok(Operand::ShortNum(bool as u8)),
        }
    }

    pub fn str(string: String) -> Self {
        Operand::String(string.to_string())
    }

    pub fn function_addr(ident: String) -> Self {
        Operand::FunctionAddr(BytecodeAddrToken{ ident })
    }

    pub fn branch_addr(label: Label) -> Self {
        Operand::BranchAddr(LabelAddrToken{ label })
    }

    pub fn bc_func_args(arg_regs: Vec<Register>) -> Self {
        Operand::FunctionArguments(FunctionArguments{ args: arg_regs })
    }

    pub fn is_worth_caching(&self) -> bool {
        match *self {
            Operand::String(_) |
            Operand::FloatNum(_) |
            Operand::LongNum(_) |
            Operand::RegistersArray(_) => true,
            _ => false
        }
    }

    fn encode_string(string: String) -> Vec<u8> {
        if string.len() > u16::max_value() as usize {
            panic!("The string '{}' is too long. Encoded string may only have 65536 charachters.");
        }

        let bytes = string.as_bytes();

        let mut encoded = vec![(bytes.len() & 0xff00) as u8, (bytes.len() & 0xff) as u8];
        encoded.extend_from_slice(bytes);
        encoded
    }

    fn encode_registers_array(regs: &[Register]) -> Vec<u8> {
        if regs.len() > u8::max_value() as usize {
            panic!("Too long registers array. Encoded byte arrays may only have 256 elements.");
        }

        let mut encoded = vec![regs.len() as u8];
        encoded.extend_from_slice(regs);
        encoded
    }

    fn encode_num(num: u32) -> Vec<u8> {
        vec![(((num & 0xff000000) >> 24) as u8),
             (((num & 0x00ff0000) >> 16) as u8),
             (((num & 0x0000ff00) >> 8) as u8),
             (((num & 0x000000ff) >> 0) as u8)]
    }

    fn encode_long_num(num: u64) -> Vec<u8> {
        vec![(((num & 0xff000000_00000000) >> 56) as u8),
             (((num & 0x00ff0000_00000000) >> 48) as u8),
             (((num & 0x0000ff00_00000000) >> 40) as u8),
             (((num & 0x000000ff_00000000) >> 32) as u8),
             (((num & 0x00000000_ff000000) >> 24) as u8),
             (((num & 0x00000000_00ff0000) >> 16) as u8),
             (((num & 0x00000000_0000ff00) >> 8) as u8),
             (((num & 0x00000000_000000ff) >> 0) as u8)]
    }

    fn encode_float_num(num: f64) -> Vec<u8> {
        Operand::encode_long_num(num.to_bits())
    }
}

impl ToBytes for Operand {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Operand::String(string) => Operand::encode_string(string.to_string()),
            Operand::FloatNum(float_num) => Operand::encode_float_num(float_num.clone()),
            Operand::LongNum(long_num) => Operand::encode_num(long_num.clone() as u32),
            Operand::ShortNum(num) |
            Operand::Reg(num) => vec![*num],
            Operand::RegistersArray(regs) => Operand::encode_registers_array(&regs),
            Operand::FunctionAddr(token)  => token.to_bytes(),
            Operand::BranchAddr(token) => token.to_bytes(),
            Operand::FunctionArguments(args) => args.to_bytes(),
        }
    }

    fn length_in_bytes(&self) -> usize {
        match self {
            Operand::String(string) => 2 + string.len(),
            Operand::FloatNum(_) => 8,
            Operand::LongNum(_) => 4,
            Operand::ShortNum(_) |
            Operand::Reg(_) => 1,
            Operand::RegistersArray(regs) => 1 + regs.len(),
            Operand::FunctionAddr(token) => token.length_in_bytes(),
            Operand::BranchAddr(token) => token.length_in_bytes(),
            Operand::FunctionArguments(args) => args.length_in_bytes(),
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operand::String(string) => write!(f, "String(\"{}\")", string),
            Operand::FloatNum(float) => write!(f, "Float({})", float),
            Operand::LongNum(long_num) => write!(f, "LongNum({})", long_num),
            Operand::ShortNum(short_num) => write!(f, "ShortNum({})", short_num),
            Operand::Reg(reg) => write!(f, "Reg({})", reg),
            Operand::RegistersArray(reg_array) => write!(f, "RegArray({:?})", reg_array),

            Operand::FunctionAddr(bc_addr_token) => write!(f, "FunctionAddr({:?})", bc_addr_token),
            Operand::BranchAddr(label_addr_token) => write!(f, "BranchAddr({:?})", label_addr_token),
            Operand::FunctionArguments(args) => write!(f, "FunctionArguments({:?})", args),
        }
    }
}

/// Contains an instruction and its operands
///
/// Every operation consists of one [instruction](enum.Instruction.html) and zero or more [operands](enum.Operand.html).
///
///```
/// use jsyc_compiler::{Operation, Instruction, Operand};
///
/// let cmd = Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(100)]);
///```
#[derive(Debug, PartialEq, Clone)]
pub struct Operation
{
    pub instruction: Instruction,
    pub operands: Vec<Operand>
}

impl Operation {
    pub fn new(instruction: Instruction, operands: Vec<Operand>) -> Self {
        Operation {
            instruction,
            operands
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.instruction.to_str())?;
        for operand in self.operands.iter() {
            write!(f, " {}", operand)?;
        }
        Ok(())
    }
}

impl ToBytes for Operation {
    fn to_bytes(&self) -> Vec<u8> {
        let mut line = vec![self.instruction.to_byte()];
        line.append(&mut self.operands.iter().map(|operand| operand.to_bytes()).flatten().collect::<Vec<u8>>());
        line
    }

    fn length_in_bytes(&self) -> usize {
        1 + self.operands.iter().fold(0, |acc, x| acc + x.length_in_bytes())
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum BytecodeElement
{
    Operation(Operation),
    Label(Label)
}

impl ToBytes for BytecodeElement {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BytecodeElement::Operation(cmd) => cmd.to_bytes(),
            BytecodeElement::Label(_) => vec![]
        }
    }

    fn length_in_bytes(&self) -> usize {
        match self {
            BytecodeElement::Operation(cmd) => cmd.length_in_bytes(),
            BytecodeElement::Label(_) => 0
        }
    }
}

/// Represents the bytecode produced by the [compiler](struct.BytecodeCompiler.html).
///
/// Bytecode is a wrapper for a list of [bytecode elements](enum.BytecodeElement.html). It offers
/// an API to extend it by other [bytecode](struct.Bytecode.html), [commands](struct.Operation.html) or [labels](type.Label.html).
/// ```
/// use jsyc_compiler::{Bytecode, Operation, Instruction, Operand};
///
/// let bytecode = Bytecode::new()
///                  .add(Operation::new(Instruction::LoadNum,
///                                    vec![Operand::Reg(10), Operand::ShortNum(10)]))
///                  .add(Operation::new(Instruction::Add,
///                                    vec![Operand::Reg(10), Operand::Reg(9)]));
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    pub elements: Vec<BytecodeElement>,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for element in self.elements.iter() {
            match element {
                BytecodeElement::Label(label) => { write!(f, "\nlabel_{}:\n", label)?; }
                BytecodeElement::Operation(cmd) => {

                    write!(f, "{}\n", cmd)?;

                    if Instruction::ReturnBytecodeFunc == cmd.instruction {
                        write!(f, "\n\n")?;
                    }

                }
            }
        }
        Ok(())
    }
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode {
            elements: vec![]
        }
    }

    pub fn add(mut self, command: Operation) -> Self {
        self.elements.push(BytecodeElement::Operation(command));
        self
    }

    /// Appends a [label](type.Label.html) as [bytecode element](enum.BytecodeElement.html).
    pub fn add_label(mut self, label: Label) -> Self {
        self.elements.push(BytecodeElement::Label(label));
        self
    }

    /// Appends another bytecode onto this bytecode.
    pub fn add_bytecode(mut self, mut other: Bytecode) -> Self {
        self.elements.append(&mut other.elements);
        self
    }

    /// Returns the base64-encoded bytecode as string.
    pub fn encode_base64(&self) -> String {
        base64::encode(&self.to_bytes())
    }

    /// Checks whether the last element is a [return instruction](enum.Instruction.html#Instruction::ReturnBytecodeFunc).
    pub fn last_op_is_return(&self) -> bool {
        match self.elements.last() {
            Some(last_element) => match last_element {
                BytecodeElement::Operation(cmd) => (cmd.instruction == Instruction::ReturnBytecodeFunc),
                _ => false
            },
            None => false
        }
    }

    /// Returns an iterator over all [commands](struct.Operation.html) in the bytecode.
    pub fn commands_iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut Operation> {
        self.elements.iter_mut().filter_map(|element| match element {
            BytecodeElement::Operation(cmd) => Some(cmd),
            BytecodeElement::Label(_) => None
        })
    }
}

impl FromIterator<Bytecode> for Bytecode {
    fn from_iter<I: IntoIterator<Item=Bytecode>>(iter: I) -> Self {
        Bytecode {
            elements: iter.into_iter().flat_map(|bc| bc.elements).collect()
        }
    }
}

impl ToBytes for Bytecode {
    fn to_bytes(&self) -> Vec<u8> {
        self.elements.iter().map(|element| element.to_bytes()).flatten().collect()
    }

    fn length_in_bytes(&self) -> usize {
        self.elements.iter().fold(0, |acc, element| acc + element.length_in_bytes())
    }
}


#[test]
fn test_instrution_to_byte() {
    assert_eq!(Instruction::Add.to_byte(), 100);
}

#[test]
fn test_bytecode_literal_from_literal() {
    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0".into())).unwrap(),
                BytecodeLiteral::IntNumber(0));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("1".into())).unwrap(),
                BytecodeLiteral::IntNumber(1));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0x10".into())).unwrap(),
                BytecodeLiteral::IntNumber(16));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0b10".into())).unwrap(),
                BytecodeLiteral::IntNumber(2));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0o10".into())).unwrap(),
                BytecodeLiteral::IntNumber(8));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0.0".into())).unwrap(),
                BytecodeLiteral::FloatNum(0.0));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("1.1".into())).unwrap(),
                BytecodeLiteral::FloatNum(1.1));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number(".0".into())).unwrap(),
                BytecodeLiteral::FloatNum(0.0));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number(".1".into())).unwrap(),
                BytecodeLiteral::FloatNum(0.1));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("0.0e0".into())).unwrap(),
                BytecodeLiteral::FloatNum(0.0));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number("1.1e2".into())).unwrap(),
                BytecodeLiteral::FloatNum(110.0));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number(".1E0".into())).unwrap(),
                BytecodeLiteral::FloatNum(0.1));

    assert_eq!(BytecodeLiteral::from_lit(Literal::Number(".1E2".into())).unwrap(),
                BytecodeLiteral::FloatNum(10.0));
}

#[test]
fn test_encode_string() {
    assert_eq!(Operand::String("Hello World".into()).to_bytes(),
               vec![0, 11, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]);
}

#[test]
fn test_encode_registers_array() {
    assert_eq!(Operand::RegistersArray(vec![]).to_bytes(),
               vec![0]);
   assert_eq!(Operand::RegistersArray(vec![1, 2, 200]).to_bytes(),
              vec![3, 1, 2, 200]);
}

#[test]
fn test_encode_long_num() {
    assert_eq!(Operand::LongNum(1_234_567_891).to_bytes(),
                vec![0x49, 0x96, 0x02, 0xD3]);

    assert_eq!(Operand::LongNum(-1_234_567_891 as i32).to_bytes(),
                vec![0xB6, 0x69, 0xFD, 0x2D])
}

#[test]
fn test_encode_float_num() {
    assert_eq!(Operand::FloatNum(0.12345).to_bytes(),
                vec![63, 191, 154, 107, 80, 176, 242, 124]);

    assert_eq!(Operand::FloatNum(0.5).to_bytes(),
                vec![0x3f, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(Operand::FloatNum(-1.1234).to_bytes(),
                vec![191, 241, 249, 114, 71, 69, 56, 239])
}

#[test]
fn test_command() {
    assert_eq!(Operation{
        instruction: Instruction::Add,
        operands:vec![
            Operand::Reg(150),
            Operand::Reg(151),
        ]
    }.to_bytes(),
    vec![100, 150, 151]);
}

#[test]
fn test_bytecode_to_bytes() {
    assert_eq!(Bytecode::new().to_bytes().len(), 0);
    assert_eq!(Bytecode{ elements: vec![
        BytecodeElement::Operation(Operation{
            instruction: Instruction::LoadNum,
            operands: vec![
                Operand::Reg(151),
                Operand::ShortNum(2),
            ]
        }),
        BytecodeElement::Operation(Operation{
            instruction: Instruction::LoadNum,
            operands: vec![
                Operand::Reg(150),
                Operand::ShortNum(3),
            ]
        }),
        BytecodeElement::Operation(Operation{
            instruction: Instruction::Mul,
            operands: vec![
                Operand::Reg(150),
                Operand::Reg(151),
            ]
        }),
        ]
    }.to_bytes(), vec![2, 151, 2, 2, 150, 3,101, 150, 151]);
}

#[test]
fn test_last_op_is_return() {
    assert_eq!(Bytecode::new().last_op_is_return(), false);
    assert_eq!(Bytecode::new().add(Operation::new(Instruction::ReturnBytecodeFunc, vec![])).last_op_is_return(), true);
    assert_eq!(Bytecode::new()
                .add(Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)]))
                .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![])).last_op_is_return(), true);
    assert_eq!(Bytecode::new().add(
            Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)])
        ).last_op_is_return(), false);
}
