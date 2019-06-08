use std::collections::*;

pub use resast::prelude::*;

use crate::bytecode::{BytecodeLiteral};
use crate::error::{CompilerError, CompilerResult};

pub type Register = u8;
pub type Reg = Register;


#[derive(Debug, Clone)]
pub enum DeclarationType {
    Variable(VariableKind),
    Function,
    Literal,
    // Intermediate
}

pub type DeclType = DeclarationType;

#[derive(Debug, Clone)]
pub struct Declaration
{
    pub register: Register,
    pub decl_type: DeclarationType,
}


#[derive(Debug, Clone)]
pub struct Scope
{
    pub decls: HashMap<String, Declaration>,
    pub unused_register: VecDeque<Register>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            decls: HashMap::new(),
            unused_register: (0..(Register::max_value() as u16 + 1)).map(|reg: u16| reg as u8).collect(),
        }
    }

    pub fn derive_scope(parent_scope: &Scope) -> Result<Self, CompilerError> {
        Ok(Scope {
            decls: parent_scope.decls.clone(),
            unused_register: parent_scope.unused_register.clone()
        })
    }

    // TODO: is this still required?
    pub fn get_throwaway_register(&self) -> Result<&Register, CompilerError> {
        self.unused_register.front().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers by using less declarations".into())
        )
    }

    pub fn get_unused_register(&mut self) -> Result<Register, CompilerError> {
        self.unused_register.pop_front().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers".into())
        )
    }

    pub fn get_unused_register_back(&mut self) -> Result<Register, CompilerError> {
        self.unused_register.pop_back().ok_or(
            CompilerError::Custom("All registers are in use. Free up some registers".into())
        )
    }

    pub fn try_reserve_specific_reg(&mut self, specific_reg: Register) -> CompilerResult<Register> {
        let maybe_idx = self.unused_register.iter().enumerate().find_map(|(i, &reg)| {
            if reg == specific_reg { Some(i) } else { None }
        });

        match maybe_idx {
            Some(idx) => {
                self.unused_register.remove(idx);
                Ok(specific_reg)
            },
            None => Err(CompilerError::Custom(format!("Failed to reserve specific reg {}", specific_reg)))
        }
    }

    pub fn add_decl(&mut self, decl: String, decl_type: DeclarationType) -> Result<Register, CompilerError> {
        let unused_reg = self.get_unused_register()?;
        self.decls.insert(decl, Declaration {
            register: unused_reg,
            decl_type: decl_type
        });
        Ok(unused_reg)
    }

    pub fn reserve_register(&mut self) -> Result<Register, CompilerError> {
        self.get_unused_register()
    }

    pub fn reserve_register_back(&mut self) -> Result<Register, CompilerError> {
        self.get_unused_register_back()
    }
}


#[derive(Debug, Clone)]
pub struct Scopes
{
    // TODO: It is not trivial to derive the Hash trait from resast::Literal,
    // and thus, it cannot be easily used in HashMap. However, this would be
    // a better choice.
    // pub literals_cache: HashMap<BytecodeLiteral, Declaration>,
    pub literals: Vec<(BytecodeLiteral, Declaration)>,
    pub scopes: Vec<Scope>,
}

impl Scopes
{
    pub fn new() -> Scopes {
        Scopes {
            // literals_cache: HashMap::new(),
            literals: vec![],
            scopes: vec![ Scope::new() ],
        }
    }

    pub fn add_lit_decl(&mut self, literal: BytecodeLiteral, reg: Reg) -> CompilerResult<()> {
        // self.literals.insert(lit, Declaration{
        //     register: reg,
        //     is_function: false
        // }).ok_or(
        //     Err(CompilerError::Custom("Failed to insert literal to hashmap".into()))
        // )

        self.literals.push((literal,
            Declaration {
                register: reg,
                decl_type: DeclarationType::Literal
            }
        ));

        Ok(())
    }

    pub fn add_var_decl(&mut self, decl: String) -> CompilerResult<Register> {
        self.add_decl(decl, DeclarationType::Variable(VariableKind::Var))
    }

    pub fn add_decl(&mut self, decl: String, decl_type: DeclarationType) -> CompilerResult<Register> {
        self.current_scope_mut()?.add_decl(decl, decl_type)
    }

    pub fn reserve_register(&mut self) -> Result<Register, CompilerError> {
        self.current_scope_mut()?.reserve_register()
    }

    pub fn reserve_register_back(&mut self) -> Result<Register, CompilerError> {
        self.current_scope_mut()?.reserve_register_back()
    }

    // TODO: is this still required?
    pub fn get_throwaway_register(&self) -> Result<&Register, CompilerError> {
        self.current_scope()?.get_throwaway_register()
    }

    pub fn get_var(&self, var_name: &str) -> Result<&Declaration, CompilerError> {
        self.current_scope()?.decls.get(var_name).ok_or(
            CompilerError::Custom(format!("The declaration '{}' does not exist", var_name))
        )
    }

    pub fn get_lit_decl(&self, literal: &BytecodeLiteral) -> CompilerResult<&Declaration> {
        // self.literals.get(literal).ok_or(
        //     Err(CompilerError::Custom("The requested literal does not exist".into()))
        // )
        match self.literals.iter().find(|&lit| lit.0 == *literal) {
            Some(lit_and_decl) => Ok(&lit_and_decl.1),
            None => Err(CompilerError::Custom("The requested literal does not exist".into()))
        }
    }

    pub fn enter_new_scope(&mut self) -> Result<(), CompilerError> {
        // Ok(self.scopes.push(Scope::derive_scope(self.current_scope()?)?))
        Ok(())
    }

    pub fn current_scope(&self) -> Result<&Scope, CompilerError> {
        self.scopes.last().ok_or(
            CompilerError::Custom("No current scope".into())
        )
    }

    pub fn current_scope_mut(&mut self) -> Result<&mut Scope, CompilerError> {
        self.scopes.last_mut().ok_or(
            CompilerError::Custom("No current (mut) scope".into())
        )
    }

    pub fn leave_current_scope(&mut self) -> Result<(), CompilerError> {
        // let _scope = self.scopes.pop().ok_or(
        //     CompilerError::Custom("Cannot leave inexisting scope".into())
        // )?;
        Ok(())
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
