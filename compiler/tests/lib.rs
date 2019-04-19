extern crate compiler;
use compiler::*;
use jshelper::{JSSourceCode};
use bytecode::*;

#[cfg(test)]
fn run_test(js_code: &str, mut compiler: compiler::BytecodeCompiler, expected_bc: compiler::Bytecode) {
    let js_source = JSSourceCode {
        source_code: js_code.to_string()
    };

    assert_eq!(compiler.compile(&js_source).unwrap(), expected_bc);
}

#[test]
fn test_compile_empty_js() {
    run_test("", BytecodeCompiler::new(), Bytecode::new());
}

#[test]
fn test_compile_js_var_decls() {
    run_test("var a = 5;", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)])));

    run_test("var a = 5, b = 6;", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(6)]))
            );

    run_test("var s = \"Hello World\";", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(0), Operand::String("\"Hello World\"".into())]))
            );
}

#[test]
fn test_compile_js_func_call() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_decl("test".into()).is_ok());

    run_test("test();", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![])]))
            );

    run_test("test(1);", compiler, Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(2), Operand::Register(1), Operand::RegistersArray(vec![
                        0
                    ])]))
            );


    // let mut compiler1 = BytecodeCompiler::new();
    // assert!(compiler1.add_decl("document".into()).is_ok());
    // // assert!(compiler1.add_decl("document.test".into()).is_ok());
    //
    // run_test("document.test();", compiler1, Bytecode::new()
    //             .add(Command::new(Instruction::PropAccess, vec![Operand::Register(1), Operand::Register(0), Operand::String("test".into())]))
    //             .add(Command::new(Instruction::CallFunc, vec![Operand::Register(2), Operand::Register(1), Operand::RegistersArray(vec![])]))
    //         );
}
