extern crate compiler;

use compiler::{BytecodeCompiler, JSSourceCode};


#[test]
fn test_compiler_api() {
    let mut compiler = BytecodeCompiler::new();
    let js_code = JSSourceCode::from_str("var a = 10");

    let bytecode = compiler.compile(&js_code).unwrap();
    assert_eq!(bytecode.encode_base64(), "AgAK");
}
