use crate::error::{CompilerError};
use std::{fmt, u16};
use base64::encode;


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
    Number(u8),
    Register(u8),
    RegistersArray(Vec<u8>)
}

impl Operand {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Operand::Str(string) => Operand::encode_string(string.to_string()),
            Operand::Number(num) | Operand::Register(num) => vec![*num],
            Operand::RegistersArray(regs) => Operand::encode_bytes_array(&regs)
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
            panic!("Too long byte array. Encoded byte arrays may only have 65536 charachters.");
        }

        let mut encoded = vec![(bytes.len() & 0xff00) as u8, (bytes.len() & 0xff) as u8];
        encoded.extend_from_slice(bytes);
        encoded
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

#[derive(Debug, PartialEq)]
pub struct Command
{
    pub instruction: Instruction,
    pub operands: Vec<Operand>
}

impl Command {
    fn to_bytes(&self) -> Vec<u8> {
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
    fn to_bytes(&self) -> Vec<u8> {
        self.commands.iter().map(|line| line.to_bytes()).flatten().collect()
    }

    fn encode(&self) -> String {
        base64::encode(&self.to_bytes())
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
                    Operand::Number(2),
                ]
            },
            Command{
                instruction: Instruction::LoadNum,
                operands: vec![
                    Operand::Register(150),
                    Operand::Number(3),
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
