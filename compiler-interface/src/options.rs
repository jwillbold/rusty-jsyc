use structopt::StructOpt;

use crate::errors::{CompositionError, CompositionResult};


#[derive(StructOpt)]
#[structopt(name = "Rusty JSYC bytecode compiler",
            about = "A tool to compile JavaScript code into bytecode to be used in virtualization obfuscation.",
            author = "Johannes Willbold <johannes.willbold@gmail.com>",
            rename_all = "verbatim")]
pub struct Options {
    #[structopt(parse(from_os_str), name = "/path/to/javascript.js")]
    pub input_path: std::path::PathBuf,

    #[structopt(parse(from_os_str), name = "/path/to/vm-template.js")]
    pub vm_template_path: std::path::PathBuf,

    #[structopt(parse(from_os_str), name = "/output/dir")]
    pub output_dir: std::path::PathBuf,

    #[structopt(parse(from_os_str), name = "/path/to/index.html")]
    pub index_html_path: Option<std::path::PathBuf>,

    #[structopt(short = "s", long = "show-bytecode")]
    pub show_bytecode: bool,

    #[structopt(short = "d", long = "show-depedencies")]
    pub show_dependencies: bool,

    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    #[structopt(flatten)]
    pub vm_options: VMOptions
}

#[derive(StructOpt)]
pub enum VMExportType {
    ES6,
    NodeJS
}

impl std::str::FromStr for VMExportType {
    type Err = CompositionError;

    fn from_str(s: &str) -> CompositionResult<Self> {
        match s {
            "ES6" => Ok(VMExportType::ES6),
            "NodeJS" => Ok(VMExportType::NodeJS),
            _ => Err(CompositionError::Custom("".into()))
        }
    }
}

#[derive(StructOpt)]
#[structopt(rename_all = "verbatim")]
pub struct VMOptions {
    #[structopt(long="vm-export-type", name="ES6/NodeJS")]
    pub export_type: Option<VMExportType>,

    // #[structopt(long)]
    // keep_unused_instructions: bool
}