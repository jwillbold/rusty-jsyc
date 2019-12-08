extern crate jsyc_compiler;

use jsyc_compiler::*;
use jshelper::{JSSourceCode};

macro_rules! reg {
    ($r:expr) => { Operand::Reg($r) };
}

macro_rules! reg_arr {
    ( $($x:expr),* ) => { Operand::RegistersArray(vec![$($x),*]) };
}

macro_rules! string {
    ($s:expr) => { Operand::String($s.into()) };
}

// macro_rules! float_num {
//     ($f:expr) => { Operand::FloatNum($f) };
// }

macro_rules! short_num {
    ($n:expr) => { Operand::ShortNum($n) };
}

macro_rules! long_num {
    ($n:expr) => { Operand::LongNum($n) };
}

macro_rules! addr {
    ($n:expr) => { Operand::LongNum($n) };
}

macro_rules! op {
    ($instr:ident, $($operands:expr),*) => { Operation::new(Instruction::$instr, vec![$($operands),*]) };
}


#[cfg(test)]
fn run_test(js_code: &str, mut compiler: compiler::BytecodeCompiler, expected_bc: Bytecode) {
    let js_source = JSSourceCode::new(js_code.to_string());

    assert_eq!(compiler.compile(&js_source).unwrap(), expected_bc);
}

#[cfg(test)]
fn run_test_deps(js_code: &str, expected_decl_deps: &[&str], expected_bc: Bytecode) {
    let mut compiler = BytecodeCompiler::new();
    let js_source = JSSourceCode::new(js_code.to_string());

    assert_eq!(compiler.compile(&js_source).unwrap(), expected_bc);

    for (decl_dep, expected_decl_dep) in compiler.decl_dependencies().decls_decps.keys().zip(expected_decl_deps) {
        assert_eq!(&decl_dep.as_str(), expected_decl_dep);
    }
}

#[cfg(test)]
fn check_is_unsupported_error(js_code: &str, mut compiler: compiler::BytecodeCompiler) {
    let js_source = JSSourceCode::new(js_code.to_string());

    assert!(compiler.compile(&js_source).is_err());
    let error = compiler.compile(&js_source).err().unwrap();

    println!("{:?}", error);

    assert!(error.is_unsupported_feature());
}


#[test]
fn test_compiler_api() {
    let mut compiler = BytecodeCompiler::new();
    let js_code = JSSourceCode::from_str("var a = 10");

    let bytecode = compiler.compile(&js_code).unwrap();
    assert_eq!(bytecode.encode_base64(), "AgAK");
}

#[test]
fn test_compile_empty_js() {
    run_test("", BytecodeCompiler::new(), Bytecode::new());
}

#[test]
fn test_compile_js_decls() {
    run_test("var a = 5;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
    );
    run_test("var a = 5, b = 6;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(6)]))
    );

    run_test("var s = \"Hello World\";", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(0), Operand::String("Hello World".into())]))
    );

    // TODO: null !== undefined
    // run_test("var xxx = null;", BytecodeCompiler::new(), Bytecode::new()
    //     .add(op!(LoadNum, reg!(0), reg!(253)))
    // );

    run_test("function foo() {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a) {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a, b) {}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a) {return a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(0), Operand::RegistersArray(vec![])]))
    );
    run_test("function foo(a, b) {a+=b; return a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(0), Operand::RegistersArray(vec![])]))
    );

    run_test_deps("var a = document.cookie;", &["document"], Bytecode::new()
        .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("cookie".into())]))
        .add(Operation::new(Instruction::PropAccess, vec![Operand::Reg(0), Operand::Reg(1), Operand::Reg(2)]))
    );

    check_is_unsupported_error("class C {}", BytecodeCompiler::new());

    check_is_unsupported_error("import foo from \"bar.js;\"", BytecodeCompiler::new());
    check_is_unsupported_error("export {foo}", BytecodeCompiler::new());
}

#[test]
fn test_bytecode_func_calls() {
    run_test("function test() {}; test();", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(8), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );

    run_test("function foo() {}; function bar() {}; foo();bar();", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(15), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(18), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );

    run_test("function foo() {var a = 5;}; function bar() {}; foo();bar();", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(15), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(21), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );

    run_test("var a = 5; function foo() {a = 10;}; foo();", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(5)]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(11), Operand::Reg(202), Operand::RegistersArray(vec![])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![0])]))
    );

    run_test("function testy(a) {} testy(10);", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(13), Operand::Reg(202), Operand::RegistersArray(vec![0, 0])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(253), Operand::RegistersArray(vec![])]))
    );

    run_test("function testy(a) {return a;} testy(10);", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(13), Operand::Reg(202), Operand::RegistersArray(vec![0, 0])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(0), Operand::RegistersArray(vec![])]))
    );

    run_test("var x = 10; function testy(a) {return a;} testy(x);", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::CallBytecodeFunc, vec![Operand::LongNum(13), Operand::Reg(202), Operand::RegistersArray(vec![1, 0])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(1), Operand::RegistersArray(vec![])]))
    );

    run_test_deps("function testy(a) {return a;}; var interval = setInterval(testy, 60);", &["setInterval"], Bytecode::new()
        .add(Operation::new(Instruction::BytecodeFuncCallback, vec![Operand::Reg(2), Operand::LongNum(19), Operand::RegistersArray(vec![0])]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(3), Operand::ShortNum(60)]))
        .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(0), Operand::Reg(1),
                                                      Operand::Reg(253), Operand::RegistersArray(vec![2, 3])]))
        .add(Operation::new(Instruction::Exit, vec![]))
        .add(Operation::new(Instruction::ReturnBytecodeFunc, vec![Operand::Reg(0), Operand::RegistersArray(vec![])]))
    );
}

#[test]
fn test_jump_stmts() {
    run_test("var a = false; if(a){a+=a;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(13)]))
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(0)]))
        .add_label(0)
    );

    run_test("var a = false; if(a){a+=a;}else{a+=2}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(18)]))
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(0)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(25)]))
        .add_label(0)
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(2)]))
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
        .add_label(1)
    );

    run_test("var a = true; while(a){a=false;}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(17)]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(3)]))
        .add_label(1)
    );

    run_test("var a = true; do{a=false;}while(a)", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)]))
        .add(Operation::new(Instruction::JumpCond, vec![Operand::Reg(0), Operand::LongNum(3)]))
        .add_label(1)
    );

    run_test("var a = 10; for(var i = 0; i < 10; ++i){++a} --i;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        // Init
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(1), Operand::Reg(255)]))
        .add_label(0)
        // Comp
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(3), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::CompLessThan, vec![Operand::Reg(2), Operand::Reg(1), Operand::Reg(3)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(2), Operand::LongNum(32)]))
        // Body
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
        // Update
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1)
        // Check that i still exists
        .add(Operation::new(Instruction::Minus, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
    );

    run_test("var a = 10; var i = 0; for(; i < 10; ++i){++a} --i;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        // Init
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(1), Operand::Reg(255)]))
        .add_label(0)
        // Comp
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(3), Operand::ShortNum(10)]))
        .add(Operation::new(Instruction::CompLessThan, vec![Operand::Reg(2), Operand::Reg(1), Operand::Reg(3)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(2), Operand::LongNum(32)]))
        // Body
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
        // Update
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1)
        // Check that i still exists
        .add(Operation::new(Instruction::Minus, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
    );

    run_test("var a = 10; var i = 0; for(;;){++a} --i;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(10)]))
        // Init
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(1), Operand::Reg(255)]))
        .add_label(0)
        // Body
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
        // Update
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1)
        // Check that i still exists
        .add(Operation::new(Instruction::Minus, vec![Operand::Reg(1), Operand::Reg(1), Operand::Reg(254)]))
    );

    run_test("var i = 0; for(;;){++i}", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(255)]))
        // Init
        .add_label(0)
        // Body
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
        // Update
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(3)]))
        .add_label(1)
    );

    let break_testcase_bytecode = Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(1), Operand::LongNum(28)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(23)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(28)]))
        .add_label(2) // If block end label
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1); // while block end label

    // Test 'break'
    run_test("var b = true; while(true) { if(b) {break;} }", BytecodeCompiler::new(),
             break_testcase_bytecode.clone());

    // Test labeled 'break', should be equal to last testcase
    run_test("var b = true; foo: while(true) { if(b) {break;} }", BytecodeCompiler::new(),
             break_testcase_bytecode);

    // Test 'continue'
    run_test("var b = true; while(true) { if(b) {continue;} }", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(1), Operand::LongNum(28)]))
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(23)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(2) // If block end label
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1) // while block end label
    );

    // Test labeled 'continue'
    run_test("var b = true; foo: while(true) { var x = 0; for(;;) { if(b) {continue;} } }", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(1)]))
        .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(1)]))
        .add_label(0)
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(1), Operand::LongNum(36)]))
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(2), Operand::Reg(255)]))
        .add_label(2)
        .add(Operation::new(Instruction::JumpCondNeg, vec![Operand::Reg(0), Operand::LongNum(26)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(15)]))
        .add_label(4) // If block end label
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(15)]))
        .add_label(3)
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(6)]))
        .add_label(1) // while block end label
    );

    run_test("outer: for(;;){ for(;;) {break outer;} }", BytecodeCompiler::new(), Bytecode::new()
        .add_label(0)
        .add_label(2)
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(10)])) // break jump
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(0)]))
        .add_label(3)
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(0)]))
        .add_label(1)
    );
}

#[test]
fn test_assigmnet_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("a".into()).is_ok());
    assert!(compiler.add_var_decl("b".into()).is_ok());

    run_test("a+=b;", compiler.clone(), Bytecode::new()
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
    );

    run_test("a*=b;", compiler.clone(), Bytecode::new()
        .add(Operation::new(Instruction::Mul, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(1)]))
    );
}

#[test]
fn test_member_expr() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("document".into()).is_ok());

    run_test("var t = document.test", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("test".into())]))
                .add(Operation::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(2)])));

    run_test("var t = document.test; var a = document.test", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("test".into())]))
                .add(Operation::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(2)]))
                .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(4), Operand::String("test".into())]))
                .add(Operation::new(Instruction::PropAccess, vec![Operand::Reg(3), Operand::Reg(0), Operand::Reg(4)])));

    // Assignment expression 'equal'
    let mut assignments_compiler = BytecodeCompiler::new();
    assert!(assignments_compiler.add_var_decl("test".into()).is_ok());
    assert!(assignments_compiler.add_var_decl("foo".into()).is_ok());

    run_test("test = 0;", assignments_compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(0), Operand::ShortNum(0)])));
    run_test("test = foo;", assignments_compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(1)])));
}

#[test]
fn test_cond_expr() {
    let mut compiler = BytecodeCompiler::new();
    compiler.add_var_decl("test".into()).unwrap();
    compiler.add_var_decl("a".into()).unwrap();
    compiler.add_var_decl("b".into()).unwrap();

    run_test("var result = (test > 0) ? a : b;", compiler, Bytecode::new()
        .add(Operation::new(Instruction::CompGreaterThan, vec![Operand::Reg(4), Operand::Reg(0), Operand::Reg(255)]))
        .add(Operation::new(Instruction::JumpCond, vec![Operand::Reg(4), Operand::LongNum(18)]))
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(3), Operand::Reg(1)]))
        .add(Operation::new(Instruction::Jump, vec![Operand::LongNum(21)]))
        .add_label(0)
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(3), Operand::Reg(2)]))
        .add_label(1)
    );
}

#[test]
fn test_unary_expr() {
    run_test("var a = void 0", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(253)]))
    );

    run_test("var a = 0; ++a;", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::Copy, vec![Operand::Reg(0), Operand::Reg(255)]))
        .add(Operation::new(Instruction::Add, vec![Operand::Reg(0), Operand::Reg(0), Operand::Reg(254)]))
    );

    // Suffix update expressions
    check_is_unsupported_error("a++;", BytecodeCompiler::new());
}

#[test]
fn test_array_expr() {
    run_test("var a = [0, 12.5, \"String\"]", BytecodeCompiler::new(), Bytecode::new()
        .add(Operation::new(Instruction::LoadFloatNum, vec![Operand::Reg(1), Operand::FloatNum(12.5)]))
        .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("String".into())]))
        .add(Operation::new(Instruction::LoadArray, vec![Operand::Reg(0), Operand::RegistersArray(vec![255, 1, 2])]))
    );
}

#[test]
fn test_compile_js_func_call() {
    let mut compiler = BytecodeCompiler::new();
    assert!(compiler.add_var_decl("test".into()).is_ok());

    run_test("test();", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(0),
                                                              Operand::Reg(253), Operand::RegistersArray(vec![])]))
            );

    run_test("test(1);", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(0),
                                                              Operand::Reg(253),Operand::RegistersArray(vec![254])]))
            );

    run_test("test(10);test(10);", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(10)]))
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(0),
                                                              Operand::Reg(253),Operand::RegistersArray(vec![1])]))
                .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(2), Operand::ShortNum(10)]))
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(0),
                                                              Operand::Reg(253),Operand::RegistersArray(vec![2])]))
            );

    run_test("test(1, 20);", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::LoadNum, vec![Operand::Reg(1), Operand::ShortNum(20)]))
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(0),
                                                              Operand::Reg(253),Operand::RegistersArray(vec![254, 1])]))
            );

    run_test("var a = test(1);", compiler.clone(), Bytecode::new()
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(1), Operand::Reg(0),
                                                              Operand::Reg(253),Operand::RegistersArray(vec![254])]))
            );



    let mut compiler_doc = BytecodeCompiler::new();
    assert!(compiler_doc.add_var_decl("document".into()).is_ok());
    // assert!(compiler1.add_var_decl("document.test".into()).is_ok());

    run_test("document.test();", compiler_doc, Bytecode::new()
                .add(Operation::new(Instruction::LoadString, vec![Operand::Reg(2), Operand::String("test".into())]))
                .add(Operation::new(Instruction::PropAccess, vec![Operand::Reg(1), Operand::Reg(0), Operand::Reg(2)]))
                .add(Operation::new(Instruction::CallFunc, vec![Operand::Reg(202), Operand::Reg(1), Operand::Reg(0), Operand::RegistersArray(vec![])]))
            );
}

#[test]
fn test_try_throw() {
    // let mut compiler_with_json = BytecodeCompiler::new();
    // assert!(compiler_with_json.add_var_decl("console".into()).is_ok());
    // assert!(compiler_with_json.add_var_decl("JSON".into()).is_ok());
    // assert!(compiler_with_json.add_var_decl("Object".into()).is_ok());
    //
    // run_test("var j; try{ var s = '{}'; j = JSON.parse(s); }\
    //          catch(e){ console.log(e); }\
    //          finally{ j = Object.create(null); }",
    //          compiler_with_json.clone(),
    //          Bytecode::new()
    //             .add(op!(Try, reg!(7), long_num!(41), long_num!(64)))
    //             .add(op!(LoadString, reg!(4), string!("{}")))
    //             .add(op!(LoadString, reg!(6), string!("parse")))
    //             .add(op!(PropAccess, reg!(5), reg!(1), reg!(6)))
    //             .add(op!(CallFunc, reg!(3), reg!(5), reg!(1), reg_arr!(4)))
    //             .add(op!(LoadNum, reg!(200), long_num!(90)))
    //             .add_label(0)
    //             .add(op!(LoadString, reg!(9), string!("log")))
    //             .add(op!(PropAccess, reg!(8), reg!(0), reg!(9)))
    //             .add(op!(CallFunc, reg!(202), reg!(8), reg!(0), reg_arr!(7)))
    //             .add(op!(LoadNum, reg!(200), long_num!(90)))
    //             .add_label(1)
    //             .add(op!(LoadString, reg!(8), string!("create")))
    //             .add(op!(PropAccess, reg!(7), reg!(2), reg!(8)))
    //             .add(op!(CallFunc, reg!(3), reg!(7), reg!(2), reg_arr!(9)))
    //             .add(op!(LoadNum, reg!(200), long_num!(90)))
    // );

    let mut compiler_with_json = BytecodeCompiler::new();
    assert!(compiler_with_json.add_var_decl("console".into()).is_ok());
    assert!(compiler_with_json.add_var_decl("JSON".into()).is_ok());
    assert!(compiler_with_json.add_var_decl("empty_object".into()).is_ok());

    run_test("var x; try{ var s = '{\"x\": 100}'; x = JSON.parse(s); }\
             catch(e){ x = empty_object; }\
             finally{ console.log(x); }",
             compiler_with_json.clone(),
             Bytecode::new()
                .add(op!(Try, reg!(7), long_num!(49), long_num!(58)))
                .add(op!(LoadString, reg!(4), string!("{\"x\": 100}")))
                .add(op!(LoadString, reg!(6), string!("parse")))
                .add(op!(PropAccess, reg!(5), reg!(1), reg!(6)))
                .add(op!(CallFunc, reg!(3), reg!(5), reg!(1), reg_arr!(4)))
                .add(op!(LoadLongNum, reg!(200), long_num!(81)))
                .add_label(0)
                .add(op!(Copy, reg!(3), reg!(2)))
                .add(op!(LoadLongNum, reg!(200), long_num!(81)))
                .add_label(1)
                .add(op!(LoadString, reg!(8), string!("log")))
                .add(op!(PropAccess, reg!(7), reg!(0), reg!(8)))
                .add(op!(CallFunc, reg!(202), reg!(7), reg!(0), reg_arr!(3)))
                .add(op!(LoadLongNum, reg!(200), long_num!(81)))
    );

    run_test("var x = 10; try { throw x*2; }", BytecodeCompiler::new(), Bytecode::new()
        .add(op!(LoadNum, reg!(0), short_num!(10)))
        .add(op!(Try, reg!(202), addr!(28), addr!(34)))
        .add(op!(LoadNum, reg!(2), short_num!(2)))
        .add(op!(Mul, reg!(1), reg!(0), reg!(2)))
        .add(op!(Throw, reg!(1)))
        .add(op!(LoadLongNum, reg!(200), long_num!(40)))
        .add_label(0)
        .add(op!(LoadLongNum, reg!(200), long_num!(40)))
        .add_label(1)
        .add(op!(LoadLongNum, reg!(200), long_num!(40)))
    );
}

#[test]
fn test_unsupported_exprs() {
    // Arrow functions
    check_is_unsupported_error("() => 0;", BytecodeCompiler::new());
    // Arrow function placeholder
    check_is_unsupported_error("_ => 0;", BytecodeCompiler::new());

    // Await, FIXME, this seems to be buggy in RESSA
    // check_is_unsupported_error("var x = await something();", BytecodeCompiler::new());

    // Class expressions
    check_is_unsupported_error("var x = class X {};", BytecodeCompiler::new());
    // Function expressions
    check_is_unsupported_error("var x = function X() {};", BytecodeCompiler::new());

    // Object related stuff
    check_is_unsupported_error("var x = new X();", BytecodeCompiler::new());
    check_is_unsupported_error("var x = {};", BytecodeCompiler::new());
    check_is_unsupported_error("var x = this;", BytecodeCompiler::new());

    // yield, FIXME
    // check_is_unsupported_error("var index; while (index < 2) { yield index++; }", BytecodeCompiler::new());

    // Spread
    check_is_unsupported_error("const nums = [1, 2, 3]; func(...nums);", BytecodeCompiler::new());

    // Seqeunce
    check_is_unsupported_error("var x = 10; var b = (x+=10, x==20);", BytecodeCompiler::new());

    // TaggedTemplate, super, meta properties
}

#[test]
fn test_unsupported_stmts() {
    check_is_unsupported_error("switch (x) { case 0: ;}", BytecodeCompiler::new());

    check_is_unsupported_error("for (x in X) {}", BytecodeCompiler::new());
    check_is_unsupported_error("for (x of X) {}", BytecodeCompiler::new());

    check_is_unsupported_error("with(x) {}", BytecodeCompiler::new());
    check_is_unsupported_error("debugger;", BytecodeCompiler::new());
}