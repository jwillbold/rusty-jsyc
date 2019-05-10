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
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)])));

    run_test("var a = 5, b = 6;", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(6)]))
            );

    run_test("var s = \"Hello World\";", BytecodeCompiler::new(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Reg(0), Operand::String("\"Hello World\"".into())]))
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
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![0])]))
    );
    // run_test("function foo(a, b) {return a+b;}", BytecodeCompiler::new(), Bytecode::new()
        // .add(Command::new(Instruction::Exit, vec![]))
    // );
    run_test("function foo(a, b) {a+=b; return a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![0])]))
    );

    check_is_error("class C {}", BytecodeCompiler::new());

    check_is_error("import foo from \"bar.js;\"", BytecodeCompiler::new());
    check_is_error("export {foo}", BytecodeCompiler::new());
}

#[test]
fn test_bytecode_func_calls() {
    run_test("function test() {}; test();", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(11), Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );

    run_test("function foo() {}; function bar() {}; foo();bar();", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(21), Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(23), Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );

    run_test("function foo() {var a = 5;}; function bar() {}; foo();bar();", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(21), Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(26), Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );

    run_test("function testy(a) {} testy(10);", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(15), Operand::RegistersArray(vec![0])]))
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![])]))
    );

    run_test("function testy(a) {return a;} testy(10);", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Command::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(15), Operand::RegistersArray(vec![0])]))
        .add(Command::new(Instruction::Exit, vec![]))
        .add(Command::new(Instruction::ReturnBytecodeFunc, vec![Operand::RegistersArray(vec![0])]))
    );
}

#[test]
fn test_jump_stmts() {
    run_test("var a = false; if(a){a+=a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(0), Operand::LongNum(17)]))
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(0)]))
        .add_label(0)
    );

    run_test("var a = false; if(a){a+=a;}else{a+=2}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(0), Operand::LongNum(26)]))
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(0)]))
        .add(Command::new(Instruction::Jump, vec![Operand::LongNum(33)]))
        .add_label(0)
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(2)]))
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
        .add_label(1)
    );

    run_test("var a = true; while(a){a=false;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(0), Operand::LongNum(25)]))
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Command::new(Instruction::Jump, vec![Operand::LongNum(3)]))
        .add_label(1)
    );

     run_test("var a = true; do{a=false;}while(a)", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(0), Operand::LongNum(3)]))
    );

    run_test("var a = 10; for(var i = 0; i < 10; ++i){++a} --i;", BytecodeCompiler::new(), Bytecode::new()
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        // Init
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(0)]))
        .add_label(0)
        // Comp
        .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(3), Operand::ShortNum(10)]))
        .add(Command::new(Instruction::CompLessThan, vec![Operand::Reg(2), Operand::Reg(1), Operand::Reg(3)]))
        .add(Command::new(Instruction::JumpCond, vec![Operand::Reg(2), Operand::LongNum(40)]))
        // Body
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
        // Update
        .add(Command::new(Instruction::Add, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
        .add(Command::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1)
        // Check that i still exists
        .add(Command::new(Instruction::Minus, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
    );
}

#[test]
fn test_assigmnet_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("a".into()).is_ok());
    assert!(compiler.add_var_decl("b".into()).is_ok());

    run_test("a+=b;", compiler.clone(), Bytecode::new()
        .add(Command::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
    );

    run_test("a*=b;", compiler.clone(), Bytecode::new()
        .add(Command::new(Instruction::Mul, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
    );
}

#[test]
fn test_member_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("document".into()).is_ok());

    run_test("var t = document.test", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(2)])));

    // TODO: The second LoadString can be eleiminated check the Expr::Member_Expr::Ident&Expr::Ident code branch in maybe_compile_expr
    run_test("var t = document.test; var a = document.test", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(2)]))
                .add(Command::new(Instruction::LoadString, vec![Operand::Reg(4), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Reg(3), Operand::Reg(0), Operand::Reg(4)])));

    // Assignment expression 'equal'
    let mut assignments_compiler = BytecodeCompiler::new();
    assert!(assignments_compiler.add_var_decl("test".into()).is_ok());
    assert!(assignments_compiler.add_var_decl("foo".into()).is_ok());

    run_test("test = 0;", assignments_compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)])));
    run_test("test = foo;", assignments_compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)])));
}

#[test]
fn test_compile_js_func_call() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("test".into()).is_ok());

    run_test("test();", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![])]))
            );

    run_test("test(1);", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![1])]))
            );

    // TODO: get rid of the second LoadNum
    run_test("test(1);test(1);", compiler.clone(), Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![1])]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(2), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(2), Operand::Reg(0), Operand::RegistersArray(vec![2])]))
            );

    run_test("test(1, 2);", compiler, Bytecode::new()
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
                .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(2), Operand::ShortNum(2)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![1, 2])]))
            );

    // run_test("test(1, 2); test(2, 1);", compiler, Bytecode::new()
    //             .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
    //             .add(Command::new(Instruction::LoadNum, vec![Operand::Reg(2), Operand::ShortNum(2)]))
    //             .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![1, 2])]))
    //         );


    let mut compiler_doc = BytecodeCompiler::new();
    assert!(compiler_doc.add_var_decl("document".into()).is_ok());
    // assert!(compiler1.add_var_decl("document.test".into()).is_ok());

    run_test("document.test();", compiler_doc, Bytecode::new()
                .add(Command::new(Instruction::LoadString, vec![Operand::Reg(1), Operand::String("test".into())]))
                .add(Command::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(1)]))
                .add(Command::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(1), Operand::RegistersArray(vec![])]))
            );
}
