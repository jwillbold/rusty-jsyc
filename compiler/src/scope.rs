use std::collections::*;

use crate::error::{CompilerError};

pub type Register = u8;

#[derive(Debug, Clone)]
pub struct Declaration
{
    // pub resast::Decl& ressa_decl,
    pub register: Register,
}

#[derive(Debug)]
pub struct Scope
{
    pub decls: HashMap<String, Declaration>,
}

impl Scope
{
    pub fn new() -> Self {
        Scope{
            decls: HashMap::new(),
        }
    }

    pub fn used_registers(&self) -> Vec<Register> {
        self.decls.iter().map(|(_, decl)| {
            decl.register
        }).collect()
    }
}

#[derive(Debug)]
pub struct Scopes
{
    scopes: Vec<Scope>,
    unused_register: Vec<Register>
}

impl Scopes
{
    pub fn new() -> Scopes {
        Scopes {
            scopes: vec![ Scope::new() ],
            unused_register: (0..Register::max_value()).collect()
        }
    }

    pub fn add_decl(&mut self, decl: String) -> Result<Register, CompilerError> {
        let unused_reg = self.get_unused_register()?;
        self.current_scope_mut()?.decls.insert(decl, Declaration {
            // ressa_decl: decl,
            register: unused_reg,
        });
        Ok(unused_reg)
    }

    pub fn get_var(&self, var_name: &str) -> Result<&Declaration, CompilerError> {
        self.current_scope()?.decls.get(var_name).ok_or(
            CompilerError::Custom(format!("The declaration '{}' does not exist", var_name))
        )
    }

    pub fn enter_new_scope(&mut self) -> Result<(), CompilerError> {
        Ok(self.scopes.push(Scope {
            decls: self.current_scope()?.decls.clone()
        }))
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

    pub fn leave_current_scope(&mut self) -> Result<Scope, CompilerError> {
        let scope = self.scopes.pop().ok_or(
            CompilerError::Custom("Cannot leave inextsiting scope".into())
        )?;
        self.unused_register.append(&mut scope.used_registers());

        Ok(scope)
    }

    fn get_unused_register(&mut self) -> Result<Register, CompilerError> {
        self.unused_register.pop().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers by using less declarations".into())
        )
    }
}

#[test]
fn test_scopes() {
    let mut scopes = Scopes::new();

    let r0 = scopes.add_decl("globalVar".into()).unwrap();

    scopes.enter_new_scope();
        let r1 = scopes.add_decl("testVar".into()).unwrap();
        let r2 = scopes.add_decl("anotherVar".into()).unwrap();
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
