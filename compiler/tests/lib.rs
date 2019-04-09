extern crate compiler;
use compiler::*;

#[test]
fn test_bytecode_genertor() {
    let compiler = compiler::BytecodeCompiler{

    };
    assert_eq!(compiler.compile(&compiler::JSSourceCode{}).unwrap(),
               compiler::Bytecode{});
}