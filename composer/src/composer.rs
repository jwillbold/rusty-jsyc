use crate::errors::{CompositionError, CompositionResult};

use compiler::{Bytecode, Register, JSSourceCode, JSAst};
use resast::prelude::*;


pub struct VM {
    vm_template: Vec<resast::ProgramPart>
}

impl VM {
    pub fn from_string(vm_code: JSSourceCode) -> CompositionResult<Self> {
        match JSAst::parse(&vm_code) {
            Ok(ast_helper) => match ast_helper.ast {
                Program::Mod(parts) |
                Program::Script(parts) => Ok(VM {
                    vm_template: parts
                })
            },
            Err(_) => Err(CompositionError::from_str("failed to parse vm code"))
        }
    }

    pub fn add_decl(reg: Register, ident: String) {

    }
    
    pub fn save_to_file<P>(self, filepath: P) -> CompositionResult<()>
        where P: AsRef<std::path::Path>
    {
        let f = std::fs::File::create(filepath)?;
        let mut writer = resw::Writer::new(f);

        for part in self.vm_template.iter() {
            writer.write_part(part).expect(&format!("failed to write part {:?}", part));
        }

        Ok(())
    }

    pub fn strip_uneeded(self) -> CompositionResult<Self> {
        Ok(VM {
            vm_template: self.vm_template.into_iter().filter(|part| {
                match part {
                    ProgramPart::Decl(decl) => match decl {
                        Decl::Class(class) => match &class.id {
                            Some(id) => id == "VM",
                            None => false,
                        },
                        _ => false
                    },
                    _ => false
                }
            }).collect()
        })
    }
}

pub struct Composer {
    vm: VM,
    bytecode: Bytecode
}

impl Composer {
    pub fn new(vm: VM, bytecode: Bytecode) -> Self {
        Composer {
            vm,
            bytecode
        }
    }

    pub fn compose(self) -> CompositionResult<(VM, Bytecode)> {
        Ok((self.vm.strip_uneeded()?, self.bytecode))
    }
}
