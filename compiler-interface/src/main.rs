extern crate jsyc_compiler;
extern crate resw;
extern crate resast;
extern crate ressa;
extern crate clap;

mod composer;
mod errors;

use std::io::Read;
use std::fs;
use jsyc_compiler::{JSSourceCode, BytecodeCompiler};
use clap::{Arg, App};

use errors::{CompositionResult};
use composer::{Composer, VM};


fn load_vm_template(path: &str) -> CompositionResult<VM> {
    let mut f = fs::File::open(path)?;
    let mut vm_code = String::new();
    f.read_to_string(&mut vm_code)?;

    VM::from_string(JSSourceCode::new(vm_code))
}

fn load_js_code(path: &str) -> CompositionResult<JSSourceCode> {
    let mut f = fs::File::open(path)?;
    let mut js_code = String::new();
    f.read_to_string(&mut js_code)?;

    Ok(JSSourceCode::new(js_code))
}

fn main() -> CompositionResult<()> {
    let matches = App::new("Rusty JSYC bytecode compiler")
                    .version("1.0")
                    .author("Johannes Willbold <johannes.willbold@rub.de>")
                    .about("A tool to compile JavaScript code into bytecode to be used in virtualization obfuscation.")
                    .arg(Arg::with_name("INPUT")
                            .required(true)
                            .value_name("/path/to/javascript.js"))
                    .arg(Arg::with_name("VM")
                            .value_name("/path/to/vm-template.js")
                            .required(true))
                    .arg(Arg::with_name("OUTPUT_DIR")
                            .value_name("/output/dir")
                            .required(true))
                    .arg(Arg::with_name("INDEX_HTML")
                            .value_name("/path/to/index.html")
                            .required(false))
                    .arg(Arg::with_name("s")
                            .short("s")
                            .long("show-bytecode"))
                    .arg(Arg::with_name("d")
                            .short("d")
                            .long("show-depedencies"))
                    .arg(Arg::with_name("v")
                            .short("v")
                            .long("verbose"))
                    .get_matches();

    let code_path =  matches.value_of("INPUT").unwrap();
    let vm_path =  matches.value_of("VM").unwrap();
    let output_dir = matches.value_of("OUTPUT_DIR").unwrap();

    if matches.is_present("v") {
        println!("Using input file: {}", code_path);
        println!("Using vm template file: {}", vm_path);
        println!("Using output dir: {}", output_dir);
    }

    let output_dir = std::path::Path::new(output_dir);

    if !output_dir.exists() {
        if let Some(paren_dir) = output_dir.parent() {
            if !paren_dir.exists() {
                if matches.is_present("v") {
                    println!("Creating output parent dir...");
                }
                fs::create_dir(paren_dir)?;
            }
        }

        if matches.is_present("v") {
            println!("Creating output dir...");
        }
        fs::create_dir(output_dir)?;
    }

    let js_code = load_js_code(code_path)?;
    let vm = load_vm_template(vm_path)?;


    println!("Starting to compile bytecode...");

    let mut compiler = BytecodeCompiler::new();
    let bytecode = compiler.compile(&js_code)?;

    println!("Finished bytecode compilation");

    if matches.is_present("d") {
        println!("Dependencies: {:?}", &compiler.decl_dependencies());
    }

    if matches.is_present("s") {
        println!("Bytecode:\n{}", &bytecode);
    }

    if let Some(index_html_template) = matches.value_of("INDEX_HTML") {
        let index_html_template_path = std::path::Path::new(index_html_template);
        println!("Using html template {}", index_html_template_path.display());
        let html_template = fs::read_to_string(index_html_template_path)?;
        let index_html = html_template.replace("base64EncodedBytecode", &bytecode.encode_base64());

        fs::write(output_dir.join(index_html_template_path.file_name()
                        .expect(&format!("{} is not a valid file path", index_html_template_path.display()))),
                  index_html)?;
    }

    println!("Starting to compose VM and bytecode...");
    let composer = Composer::new(vm, bytecode);

    let (vm, bytecode) = composer.compose(compiler.decl_dependencies())?;
    vm.save_to_file(output_dir.join("vm.js"))?;

    let base64_bytecode = bytecode.encode_base64();
    fs::write(output_dir.join("bytecode.base64"), base64_bytecode)?;

    Ok(())
}
