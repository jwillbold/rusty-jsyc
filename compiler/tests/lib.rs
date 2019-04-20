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

#[cfg(test)]
fn check_is_error(js_code: &str, mut compiler: compiler::BytecodeCompiler) {
    let js_source = JSSourceCode {
        source_code: js_code.to_string()
    };

    assert!(compiler.compile(&js_source).is_err());
}

#[test]
fn test_compile_empty_js() {
    run_test("", BytecodeCompiler::new(), Bytecode::new());
}

#[test]
fn test_compile_js_decls() {
    run_test("var a = 5;", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)])));

    run_test("var a = 5, b = 6;", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(5)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(6)]))
            );

    run_test("var s = \"Hello World\";", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(0), Operand::String("\"Hello World\"".into())]))
            );

    run_test("function foo() {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a) {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a, b) {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a) {return a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![1])]))
    );
    // run_test("function foo(a, b) {return a+b;}", BytecodeCompiler::new(), Bytecode::new()
        // .add(Command::new(Instruction::Exit, vec![]))
    // );
    run_test("function foo(a, b) {a+=b; return a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::Add, vec![Operand::Register(1), Operand::Register(1), Operand::Register(2)]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![1])]))
    );

    check_is_error("class C {}", BytecodeCompiler::new());

    check_is_error("import foo from \"bar.js;\"", BytecodeCompiler::new());
    check_is_error("export {foo}", BytecodeCompiler::new());
}

#[test]
fn test_assigmnet_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_decl("a".into()).is_ok());
    assert!(compiler.add_decl("b".into()).is_ok());

    run_test("a+=b;", compiler.clone(), Bytecode::new()
        .add(Command::new(Instruction::Add, vec![Operand::Register(0), Operand::Register(0), Operand::Register(1)]))
    );

    run_test("a*=b;", compiler.clone(), Bytecode::new()
        .add(Command::new(Instruction::Mul, vec![Operand::Register(0), Operand::Register(0), Operand::Register(1)]))
    );
}

#[test]
fn test_member_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_decl("document".into()).is_ok());

    run_test("var t = document.test", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(2), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Register(1), Operand::Register(0), Operand::Register(2)])));

    // TODO: The second LoadString can be eleiminated check the Expr::Member_Expr::Ident&Expr::Ident code branch in maybe_compile_expr
    run_test("var t = document.test; var a = document.test", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(2), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Register(1), Operand::Register(0), Operand::Register(2)]))
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(4), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Register(3), Operand::Register(0), Operand::Register(4)])));

    // Assignment expression 'equal'
    let mut assignments_compiler = BytecodeCompiler::new();
    assert!(assignments_compiler.add_decl("test".into()).is_ok());
    assert!(assignments_compiler.add_decl("foo".into()).is_ok());

    run_test("test = 0;", assignments_compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(0), Operand::ShortNum(0)])));
    run_test("test = foo;", assignments_compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::Copy, vec![Operand::Register(0), Operand::Register(1)])));
}

#[test]
fn test_compile_js_func_call() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_decl("test".into()).is_ok());

    run_test("test();", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![])]))
            );

    run_test("test(1);", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![1])]))
            );

    // TODO: get rid of the second LoadNum
    run_test("test(1);test(1);", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![1])]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(2), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(2), Operand::Register(0), Operand::RegistersArray(vec![2])]))
            );

    run_test("test(1, 2);", compiler, Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Register(2), Operand::ShortNum(2)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![1, 2])]))
            );

    // run_test("test(1, 2); test(2, 1);", compiler, Bytecode::new()
    //             .add(Command::new(Instruction::LoadNum, vec![Operand::Register(1), Operand::ShortNum(1)]))
    //             .add(Command::new(Instruction::LoadNum, vec![Operand::Register(2), Operand::ShortNum(2)]))
    //             .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(0), Operand::RegistersArray(vec![1, 2])]))
    //         );


    let mut compiler_doc = BytecodeCompiler::new();
    assert!(compiler_doc.add_decl("document".into()).is_ok());
    // assert!(compiler1.add_decl("document.test".into()).is_ok());

    run_test("document.test();", compiler_doc, Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Register(1), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Register(1), Operand::Register(0), Operand::Register(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Register(1), Operand::Register(1), Operand::RegistersArray(vec![])]))
            );
}
