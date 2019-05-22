extern crate compiler;
extern crate resw;
extern crate resast;
extern crate ressa;

mod composer;
mod errors;

use std::io::Read;
use std::env;
use std::fs;
use compiler::{Bytecode, JSSourceCode, BytecodeCompiler};
use errors::{CompositionResult};
use composer::{Composer, VM};


fn print_usage() {
    println!("Usage: './composer /path/to/javascript.js /path/to/vm-template.js' /output/dir\n");
}

fn load_vm_template(path: &str) -> CompositionResult<VM> {
    let mut f = fs::File::open(path)?;
    let mut vm_code = String::new();
    f.read_to_string(&mut vm_code)?;

    VM::from_string(JSSourceCode::new(vm_code))
}

fn load_and_compile_js_code(path: &str) -> CompositionResult<Bytecode> {
    let mut f = fs::File::open(path)?;
    let mut js_code = String::new();
    f.read_to_string(&mut js_code)?;

    println!("Starting to compile bytecode...");

    let mut compiler = BytecodeCompiler::new();
    let bytecode = compiler.compile(&JSSourceCode::new(js_code))?;

    println!("Finished bytecode compilation");

    Ok(bytecode)
}

fn main() -> CompositionResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        print_usage();
        return Ok(())
    }

    let code_path = &args[1];
    let vm_path = &args[2];
    let output_dir = std::path::Path::new(&args[3]);

    if !output_dir.exists() {
        if let Some(paren_dir) = output_dir.parent() {
            if !paren_dir.exists() {
                fs::create_dir(paren_dir)?;
            }
        }

        fs::create_dir(output_dir)?;
    }

    let bytecode = load_and_compile_js_code(code_path)?;
    let vm = load_vm_template(vm_path)?;

    println!("Starting to compose VM and bytecode...");
    let composer = Composer::new(vm, bytecode);

    let (vm, bytecode) = composer.compose()?;
    vm.save_to_file(output_dir.join("vm.js"))?;

    let base64_bytecode = bytecode.encode_base64();
    fs::write(output_dir.join("bytecode.base64"), base64_bytecode)?;

    Ok(())
}
