extern crate jsyc_compiler;
extern crate resw;
extern crate resast;
extern crate ressa;
extern crate structopt;

mod composer;
mod errors;
mod options;

use std::io::Read;
use std::fs;
use std::path::PathBuf;
use jsyc_compiler::{JSSourceCode, BytecodeCompiler};

use crate::errors::{CompositionResult};
use crate::composer::{Composer, VM};
use crate::options::{Options};
use crate::structopt::StructOpt;


fn load_js_from_file(path: &PathBuf) -> CompositionResult<JSSourceCode> {
    let mut f = fs::File::open(path)?;
    let mut string = String::new();
    f.read_to_string(&mut string)?;

    Ok(JSSourceCode::new(string))
}

fn main() -> CompositionResult<()> {
    let options = Options::from_args();

    if options.verbose {
        println!("Using input file: {}", options.input_path.to_str().unwrap());
        println!("Using vm template file: {}", options.vm_template_path.to_str().unwrap());
        println!("Using output dir: {}", options.output_dir.to_str().unwrap());
    }

    let output_dir = std::path::Path::new(&options.output_dir);

    if !output_dir.exists() {
        if let Some(paren_dir) = output_dir.parent() {
            if !paren_dir.exists() {
                if options.verbose {
                    println!("Creating output parent dir...");
                }
                fs::create_dir(paren_dir)?;
            }
        }

        if options.verbose {
            println!("Creating output dir...");
        }
        fs::create_dir(output_dir)?;
    }

    let js_code = load_js_from_file(&options.input_path)?;
    let vm = VM::from_js_code(load_js_from_file(&options.vm_template_path)?)?;


    println!("Starting to compile bytecode...");

    let mut compiler = BytecodeCompiler::new();
    let bytecode = compiler.compile(&js_code)?;

    println!("Finished bytecode compilation");

    if options.show_dependencies {
        println!("Dependencies: {:?}", &compiler.decl_dependencies());
    }

    if options.show_bytecode {
        println!("Bytecode:\n{}", &bytecode);
    }

    if let Some(index_html_template) = &options.index_html_path {
        let index_html_template_path = std::path::Path::new(&index_html_template);
        println!("Using html template {}", index_html_template_path.display());
        let html_template = fs::read_to_string(index_html_template_path)?;
        let index_html = html_template.replace("Base64EncodedBytecode", &bytecode.encode_base64());

        fs::write(output_dir.join(index_html_template_path.file_name()
                        .expect(&format!("{} is not a valid file path", index_html_template_path.display()))),
                  index_html)?;
    }

    println!("Starting to compose VM and bytecode...");
    let composer = Composer::new(vm, bytecode, &options);

    let (vm, bytecode) = composer.compose(compiler.decl_dependencies())?;
    vm.save_to_file(output_dir.join("vm.js"))?;

    let base64_bytecode = bytecode.encode_base64();
    fs::write(output_dir.join("bytecode.base64"), base64_bytecode)?;

    Ok(())
}
