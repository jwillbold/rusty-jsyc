use std::collections::*;

use crate::error::{CompilerError};

pub type Register = u8;
pub type Reg = Register;

#[derive(Debug, Clone)]
pub struct Declaration
{
    // pub resast::Decl& ressa_decl,
    pub register: Register,
    pub is_function: bool,
}

impl Declaration {
    pub fn is_function(&self) -> bool {
        self.is_function
    }
}

// #[derive(Hash, Debug, Clone, PartialEq, Eq)]
// pub enum CommonLiterals {
//     Zero,
//     One,
//     Undefined,
//     EmptyString
// }

#[derive(Debug, Clone)]
pub struct Scope
{
    pub decls: HashMap<String, Declaration>,
    pub unnamed_reserved_registers: VecDeque<Register>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            decls: HashMap::new(),
            unnamed_reserved_registers: VecDeque::new(),
        }
    }

    pub fn derive_scope(parent_scope: &Scope) -> Result<Self, CompilerError> {
        Ok(Scope {
            decls: parent_scope.decls.clone(),
            unnamed_reserved_registers: VecDeque::new(),
        })
    }

    pub fn used_registers(self) -> VecDeque<Register> {
        let mut uses_regs = self.unnamed_reserved_registers;
        uses_regs.append(&mut self.decls.iter().map(|(_, decl)| {
            decl.register
        }).collect());
        uses_regs
    }

    // pub fn name_by_register(&self, register: Register) -> Result<String, CompilerError> {
    //     for (name, decl) in &self.decls {
    //         if decl.register == register {
    //             return Ok(name.to_string());
    //         }
    //     }
    //
    //     Err(CompilerError::Custom(format!("Failed to find name for regitser: {}", register)))
    // }
}

#[derive(Debug, Clone)]
pub struct Scopes
{
    pub scopes: Vec<Scope>,
    pub unused_register: VecDeque<Register>,
    // pub common_literals: HashMap<CommonLiterals, Register>
}

impl Scopes
{
    pub fn new() -> Scopes {
        Scopes {
            scopes: vec![ Scope::new() ],
            unused_register: (0..Register::max_value()).collect(),
            // common_literals: HashMap::new()
        }
    }

    pub fn add_var_decl(&mut self, decl: String) -> Result<Register, CompilerError> {
        self.add_decl(decl, false)
    }

    pub fn add_func_decl(&mut self, decl: String) -> Result<Register, CompilerError> {
        self.add_decl(decl, true)
    }

    pub fn add_decl(&mut self, decl: String, is_function: bool) -> Result<Register, CompilerError> {
        let unused_reg = self.get_unused_register()?;
        self.current_scope_mut()?.decls.insert(decl, Declaration {
            register: unused_reg,
            is_function: is_function
        });
        Ok(unused_reg)
    }

    pub fn reserve_register(&mut self) -> Result<Register, CompilerError> {
        self.get_unused_register()
    }

    pub fn reserve_register_back(&mut self) -> Result<Register, CompilerError> {
        self.get_unused_register_back()
    }

    pub fn get_throwaway_register(&self) -> Result<&Register, CompilerError> {
        self.unused_register.front().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers by using less declarations".into())
        )
    }

    pub fn get_var(&self, var_name: &str) -> Result<&Declaration, CompilerError> {
        self.current_scope()?.decls.get(var_name).ok_or(
            CompilerError::Custom(format!("The declaration '{}' does not exist", var_name))
        )
    }

    pub fn enter_new_scope(&mut self) -> Result<(), CompilerError> {
        Ok(self.scopes.push(Scope::derive_scope(self.current_scope()?)?))
    }

    pub fn current_scope(&self) -> Result<&Scope, CompilerError> {
        self.scopes.last().ok_or(
            CompilerError::Custom("No current scope".into())
        )
    }

    fn current_scope_mut(&mut self) -> Result<&mut Scope, CompilerError> {
        self.scopes.last_mut().ok_or(
            CompilerError::Custom("No current (mut) scope".into())
        )
    }

    pub fn leave_current_scope(&mut self) -> Result<(), CompilerError> {
        let scope = self.scopes.pop().ok_or(
            CompilerError::Custom("Cannot leave inextsiting scope".into())
        )?;
        Ok(self.unused_register.append(&mut scope.used_registers()))
    }

    fn get_unused_register(&mut self) -> Result<Register, CompilerError> {
        self.unused_register.pop_front().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers".into())
        )
    }

    fn get_unused_register_back(&mut self) -> Result<Register, CompilerError> {
        self.unused_register.pop_back().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers".into())
        )
    }
}

#[test]
fn test_scopes() {
    let mut scopes = Scopes::new();

    let r0 = scopes.add_var_decl("globalVar".into()).unwrap();

    scopes.enter_new_scope().unwrap();
        let r1 = scopes.add_var_decl("testVar".into()).unwrap();
        let r2 = scopes.add_var_decl("anotherVar".into()).unwrap();
        assert_ne!(r0, r1);
        assert_ne!(r1, r2);
        assert_eq!(scopes.get_var("testVar").unwrap().register, r1);
        assert_eq!(scopes.get_var("anotherVar").unwrap().register, r2);
    assert!(scopes.leave_current_scope().is_ok());

    assert_eq!(scopes.get_var("globalVar").unwrap().register, r0);
    assert!(scopes.get_var("testVar").is_err());
    assert!(scopes.get_var("anotherVar").is_err());

    assert!(scopes.leave_current_scope().is_ok());

    assert!(scopes.current_scope().is_err());
}
