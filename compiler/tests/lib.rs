extern crate compiler;
use compiler::*;
use jshelper::{JSSourceCode};
use bytecode::*;

#[cfg(test)]
fn run_test(js_code: &str, expected_bc: compiler::Bytecode) {
    let js_source = JSSourceCode {
        source_code: js_code.to_string()
    };

    assert_eq!(compiler::BytecodeCompiler::new().compile(&js_source).unwrap(), expected_bc);
}

#[test]
fn test_compile_empty_js() {
    run_test("", Bytecode::new());
}

#[test]
fn test_compile_js_var_decls() {
    run_test("var a = 5;", Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)])));

    run_test("var a = 5, b = 6;", Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(6)]))
            );

    run_test("var s = \"Hello World\";", Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(0), Operand::Str("\"Hello World\"".into())]))
            );
}
