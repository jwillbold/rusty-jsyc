extern crate compiler;
use compiler::*;


#[test]
fn test_compile_empty_js_code() {
    let mut compiler = compiler::BytecodeCompiler::new();
    let js_code = compiler::JSSourceCode{
        source_code: "".into()
    };

    assert_eq!(compiler.compile(&js_code).unwrap(),
               compiler::Bytecode{commands: vec![]});
}

#[test]
fn test_compile_assign_int() {
    let compiler = compiler::BytecodeCompiler::new();
    let js_code = compiler::JSSourceCode{
        source_code: "var a = 5;".into()
    };


}

// #[test]
// fn test_hello_world() {
//     let compiler = compiler::BytecodeCompiler{};
//     let js_code = compiler::JSSourceCode{
//         source_code: "function helloWorld() { alert('Hello world'); }".into()
//     };
//
//
// }
