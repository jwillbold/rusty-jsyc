use crate::errors::{CompositionError, CompositionResult};

use jsyc_compiler::{Bytecode, JSSourceCode, JSAst, DeclDepencies};
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

    pub fn inject_decleration_dependencies(self, decls_deps: &DeclDepencies) -> CompositionResult<Self> {
        // The first part of this (until the next big comment) is to find function declerations
        // inside the class "VM"
        Ok(VM{
            vm_template: self.vm_template.into_iter().map(|part| {
                match part {
                    ProgramPart::Decl(Decl::Class(mut class)) => ProgramPart::Decl(Decl::Class(match &class.id {
                        Some(id) => match id.as_str() == "VM" {
                            true => {
                                class.body = class.body.into_iter().map(|mut property| {
                                    property.value = match property.value {
                                        PropertyValue::Expr(Expr::Function(mut function)) => {

                                            // In this part it gets checked whether any of the functions
                                            // function calls contains the idetifier argument
                                            // FutureDeclerationsPlaceHolder. If this is the case,
                                            // This call is used as template. It's arguments get
                                            // replaced by the detected dependencies
                                            let maybe_template_call = function.body.iter().enumerate().find_map(|(i, part)| {
                                                match part {
                                                    ProgramPart::Stmt(Stmt::Expr(Expr::Call(call_expr))) => {
                                                        call_expr.arguments.iter().find(|&arg_expr| {
                                                            arg_expr == &Expr::Ident("FutureDeclerationsPlaceHolder".into())
                                                        }).map(|_| (i, call_expr))
                                                    },
                                                    _ => None
                                                }
                                            });

                                            if let Some((i, template_call)) = maybe_template_call {
                                                let dep_stmts = decls_deps.decls_decps.iter().map(|(ident, reg)| {
                                                    let mut call = template_call.clone();

                                                    call.arguments = vec![
                                                        Expr::Literal(Literal::Number(reg.to_string())),
                                                        Expr::Ident(ident.to_string()),
                                                    ];

                                                    ProgramPart::Stmt(Stmt::Expr(Expr::Call(call)))
                                                }).collect::<Vec<ProgramPart>>();

                                                function.body.extend(dep_stmts);
                                                function.body.remove(i);
                                            }

                                            PropertyValue::Expr(Expr::Function(function))
                                        },
                                        _ => property.value
                                    };
                                    property
                                }).collect();
                                class
                            },
                            false => class
                        },
                        None => class,
                    })),
                    _ => part
                }
            }).collect()
        })
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
                        Decl::Variable(kind, _) => kind == &VariableKind::Const,
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

    pub fn compose(self, decls_deps: &DeclDepencies) -> CompositionResult<(VM, Bytecode)> {
        Ok((self.vm.inject_decleration_dependencies(decls_deps)?.strip_uneeded()?, self.bytecode))
    }
}