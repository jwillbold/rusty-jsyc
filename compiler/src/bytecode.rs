use crate::error::{CompilerError};
use std::{fmt, u16};
use std::iter::FromIterator;
use base64::encode;

pub use resast::prelude::*;


#[derive(Debug, PartialEq)]
pub enum Instruction
{
    LoadString,
    LoadNum,

    PropAccess,
    CallFunc,
    Eval,
    CallBytecodeFunc,
    ReturnBytecodeFunc,
    Copy,
    Exit,
    JumpCond,

    Add,
    Mul,
}

impl Instruction {
    fn to_byte(&self) -> u8 {
        match self {
            Instruction::LoadString => 1,
            Instruction::LoadNum => 2,

            Instruction::PropAccess => 10,
            Instruction::CallFunc => 11,
            Instruction::Eval => 12,
            Instruction::CallBytecodeFunc => 13,
            Instruction::ReturnBytecodeFunc => 14,
            Instruction::Copy => 15,
            Instruction::Exit => 16,
            Instruction::JumpCond => 17,

            Instruction::Add => 100,
            Instruction::Mul => 101,
        }
    }
}

impl Into<u8> for Instruction {
    fn into(self) -> u8 {
        self.to_byte()
    }
}

#[test]
fn test_instrution_to_byte() {
    assert_eq!(Instruction::Add.to_byte(), 100);
}

#[derive(Debug, PartialEq)]
pub enum Operand
{
    Str(String),
    FloatNum(f64),
    LongNum(i64),
    ShortNum(u8),
    Register(u8),
    RegistersArray(Vec<u8>),
}

impl Operand {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Operand::Str(string) => Operand::encode_string(string.to_string()),
            Operand::FloatNum(float_num) => Operand::encode_float_num(float_num.clone()),
            Operand::LongNum(long_num) => Operand::encode_long_num(long_num.clone() as u64),
            Operand::ShortNum(num) | Operand::Register(num) => vec![*num],
            Operand::RegistersArray(regs) => Operand::encode_bytes_array(&regs)
        }
    }

    pub fn from_literal(lit: Literal) -> Result<Self, CompilerError> {
        match lit {
            Literal::Null => Ok(Operand::Register(0)), //TODO: Register of predefined void 0,
            Literal::String(string) => Ok(Operand::Str(string)),
            Literal::Number(num) => Ok(Operand::ShortNum(num.parse().unwrap())), //TODO
            Literal::Boolean(bool) => Ok(Operand::ShortNum(bool as u8)),
            Literal::RegEx(_) | Literal::Template(_) => Err(CompilerError::Custom("regex and template literals are not supported".into()))
        }
    }

    fn encode_string(string: String) -> Vec<u8> {
        if string.len() > u16::max_value() as usize {
            panic!("The string '{}' is too long. Encoded string may only have 65536 charachters.");
        }

        Operand::encode_bytes_array(string.as_bytes())
    }

    fn encode_bytes_array(bytes: &[u8]) -> Vec<u8> {
        if bytes.len() > u16::max_value() as usize {
            panic!("Too long byte array. Encoded byte arrays may only have 65536 elements.");
        }

        let mut encoded = vec![(bytes.len() & 0xff00) as u8, (bytes.len() & 0xff) as u8];
        encoded.extend_from_slice(bytes);
        encoded
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

impl Into<Vec<u8>> for Operand {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}

#[test]
fn test_encode_string() {
    assert_eq!(Operand::Str("Hello World".into()).to_bytes(),
               vec![0, 11, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]);
}

#[test]
fn test_encode_bytes_array() {
    assert_eq!(Operand::RegistersArray(vec![]).to_bytes(),
               vec![0, 0]);
   assert_eq!(Operand::RegistersArray(vec![1, 2, 200]).to_bytes(),
              vec![0, 3, 1, 2, 200]);
}

#[test]
fn test_encode_long_num() {
    assert_eq!(Operand::LongNum(1234567890123456789).to_bytes(),
                vec![0x11, 0x22, 0x10, 0xf4, 0x7d, 0xe9, 0x81, 0x15]);

    assert_eq!(Operand::LongNum(-1234567890123456789 as i64).to_bytes(),
                vec![0xEE, 0xDD, 0xEF, 0x0B, 0x82, 0x16, 0x7E, 0xEB])
}

#[test]
fn test_encode_float_num() {
    assert_eq!(Operand::FloatNum(0.12345).to_bytes(),
                vec![63, 191, 154, 107, 80, 176, 242, 124]);

    assert_eq!(Operand::FloatNum(-1.1234).to_bytes(),
                vec![191, 241, 249, 114, 71, 69, 56, 239])
}

#[derive(Debug, PartialEq)]
pub struct Command
{
    pub instruction: Instruction,
    pub operands: Vec<Operand>
}

impl Command {

    pub fn new(instruction: Instruction, operands: Vec<Operand>) -> Self {
        Command {
            instruction,
            operands
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut line = vec![self.instruction.to_byte()];
        line.append(&mut self.operands.iter().map(|operand| operand.to_bytes()).flatten().collect::<Vec<u8>>());
        line
    }
}

impl Into<Vec<u8>> for Command {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}

#[test]
fn test_command() {
    assert_eq!(Command{
        instruction: Instruction::Add,
        operands:vec![
            Operand::Register(150),
            Operand::Register(151),
        ]
    }.to_bytes(),
    vec![100, 150, 151]);
}


#[derive(Debug, PartialEq)]
pub struct Bytecode
{
    pub commands: Vec<Command>
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TO{0}", "DO")
    }
}

impl Bytecode {

    pub fn new() -> Self {
        Bytecode {
            commands: vec![]
        }
    }

    pub fn add(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.commands.iter().map(|line| line.to_bytes()).flatten().collect()
    }

    fn encode(&self) -> String {
        base64::encode(&self.to_bytes())
    }
}

impl FromIterator<Bytecode> for Bytecode {
    fn from_iter<I: IntoIterator<Item=Bytecode>>(iter: I) -> Self {
        Bytecode {
            commands: iter.into_iter().flat_map(|bc| bc.commands).collect()
        }
    }
}


#[test]
fn test_bytecode_to_bytes() {
    assert_eq!(Bytecode{ commands: vec![] }.to_bytes().len(), 0);
    assert_eq!(Bytecode{ commands: vec![
            Command{
                instruction: Instruction::LoadNum,
                operands: vec![
                    Operand::Register(151),
                    Operand::ShortNum(2),
                ]
            },
            Command{
                instruction: Instruction::LoadNum,
                operands: vec![
                    Operand::Register(150),
                    Operand::ShortNum(3),
                ]
            },
            Command{
                instruction: Instruction::Mul,
                operands: vec![
                    Operand::Register(150),
                    Operand::Register(151),
                ]
            },
        ]}.to_bytes(), vec![2, 151, 2, 2, 150, 3,101, 150, 151]);
}
