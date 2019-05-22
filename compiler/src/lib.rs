#[warn(unused_imports)]

extern crate ressa;
extern crate resast;
extern crate base64;
#[macro_use]
extern crate log;

pub mod error;
pub mod bytecode;
pub mod jshelper;
pub mod compiler;
pub mod scope;
pub mod instruction_set;

pub use crate::bytecode::{Bytecode, BytecodeElement, Command, Instruction, Operand, ToBytes};
pub use crate::compiler::{BytecodeCompiler};
pub use crate::error::{CompilerResult, CompilerError};
pub use crate::instruction_set::{InstructionSet};
pub use crate::jshelper::{JSSourceCode, JSAst};
pub use crate::scope::{Register};
