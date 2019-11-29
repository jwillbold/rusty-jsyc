use jsyc_compiler::{Bytecode, JSSourceCode, JSAst, DeclDepencies};
use resast::prelude::*;

use crate::errors::{CompositionError, CompositionResult};
use crate::options::{Options, VMExportType};


#[derive(Debug, PartialEq)]
pub struct VM {
    vm_template: Vec<resast::ProgramPart>
}

impl VM {
    pub fn from_js_code(vm_code: JSSourceCode) -> CompositionResult<Self> {
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

    pub fn strip_unneeded(self) -> CompositionResult<Self> {
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

    pub fn append_export_stmt(&mut self, export_types: &VMExportType) {
        use resast::*;
        self.vm_template.push(match export_types {
            VMExportType::NodeJS => {
                ProgramPart::Stmt(Stmt::Expr(Expr::Assignment(AssignmentExpr{
                    operator: AssignmentOperator::Equal,
                    left: AssignmentLeft::Expr(Box::new(Expr::Member(MemberExpr{
                        object: Box::new(Expr::Ident("module".into())),
                        property: Box::new(Expr::Ident("exports".into())),
                        computed: false
                    }))),
                    right: Box::new(Expr::Ident("VM".into()))
                })))
            },
            VMExportType::ES6 => {
                ProgramPart::Decl(Decl::Export(Box::new(ModExport::Named(NamedExportDecl::Specifier(
                    vec![ExportSpecifier::new("VM".into(), None)],
                    None
                )))))
            }
        });
    }


}

pub struct Composer<'a> {
    vm: VM,
    bytecode: Bytecode,
    options: &'a Options
}

impl<'a> Composer<'a> {
    pub fn new(vm: VM, bytecode: Bytecode, options: &'a Options) -> Self {
        Composer {
            vm,
            bytecode,
            options
        }
    }

    pub fn compose(self, decls_deps: &DeclDepencies) -> CompositionResult<(VM, Bytecode)> {
        let mut vm = self.vm.inject_decleration_dependencies(decls_deps)?.strip_unneeded()?;

        if let Some(export_type) = &self.options.vm_options.export_type {
            vm.append_export_stmt(export_type);
        }

        Ok((vm, self.bytecode))
    }
}


#[test]
fn test_vm_strip_strip_unneeded() {
    let vm = VM::from_js_code(JSSourceCode::from_str(
        "if(typeof window == \"undefined\") {\
           var window = {};\
        }\
        const REGS = {};
        const OP = {};
        class VM {}\
        module.exports = function() {\
           this.REGS = REGS;\
        }")).unwrap();

    let clean_vm = vm.strip_unneeded().unwrap();
    let expected_vm = VM::from_js_code(JSSourceCode::from_str("
        const REGS = {};\
        const OP = {};\
        class VM {}")).unwrap();

    assert_eq!(clean_vm, expected_vm);
}

#[test]
fn test_vm_inject_decleration_dependencies() {
    let vm = VM::from_js_code(JSSourceCode::from_str(
        "class VM {\
              init(bytecode) {
                  this.setReg(255, 0);

                  this.setReg(FutureDeclerationsPlaceHolder, 0);
              }
        }")).unwrap();

    let mut decl_deps = DeclDepencies::new();
    decl_deps.add_decl_dep("document".into(), 2);
    decl_deps.add_decl_dep("window".into(), 10);

    let injected_vm = vm.inject_decleration_dependencies(&decl_deps).unwrap();

    // Both varinats expected_vm_0 and expected_vm_1 are possible since
    // DeclDepencies uses a HashMap internally so the order of the elements is unpredictable
    let expected_vm_0 = VM::from_js_code(JSSourceCode::from_str(
        "class VM {\
              init(bytecode) {
                  this.setReg(255, 0);

                  this.setReg(2, document);
                  this.setReg(10, window);
              }
        }")).unwrap();

    let expected_vm_1 = VM::from_js_code(JSSourceCode::from_str(
        "class VM {\
              init(bytecode) {
                  this.setReg(255, 0);

                  this.setReg(10, window);
                  this.setReg(2, document);
              }
        }")).unwrap();

    if injected_vm == expected_vm_0 {
        assert_eq!(injected_vm, expected_vm_0);
    } else {
        assert_eq!(injected_vm, expected_vm_1);
    }
}
