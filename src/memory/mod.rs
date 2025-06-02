// Widow Programming Language
// Memory module - Leverages Rust's borrow checker for memory safety

use std::cell::{RefCell, Ref, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use crate::error::{Result, WidowError};
use crate::ast;

/// The core value type in Widow's memory system
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<HashMap<String, Value>>>),
    Struct(Rc<RefCell<StructInstance>>),
    Function(Rc<Function>),
    Nil,
}

/// Structure instance with fields
#[derive(Debug)]
pub struct StructInstance {
    pub struct_name: String,
    pub fields: HashMap<String, Value>,
}

/// Function representation
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub parameters: Vec<String>,
    pub body: crate::ast::BlockStatement,
}

/// Memory environment for a scope
pub struct Environment {
    /// Variables in the current scope
    variables: HashMap<String, RefCell<Value>>,
    /// Parent environment for closures and nested scopes
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// Create a new environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            enclosing: None,
        }
    }
    
    /// Create a new environment with the given parent
    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            variables: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }
    
    /// Define a new variable in the current environment
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, RefCell::new(value));
    }
    
    /// Get a copy of a variable's value
    pub fn get_value(&self, name: &str) -> Result<Value> {
        // Try to find the variable in the current scope
        if let Some(value) = self.variables.get(name) {
            return Ok(value.borrow().clone());
        }
        
        // If not found in current scope, check the enclosing scope
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get_value(name);
        }
        
        // Variable not found
        Err(WidowError::Runtime {
            message: format!("Undefined variable '{}'", name)
        })
    }
    
    /// Check if a variable is mutable
    pub fn is_mutable(&self, name: &str) -> Result<bool> {
        // Try to find the variable in the current scope
        if let Some(value) = self.variables.get(name) {
            // All variables are mutable in this implementation
            return Ok(true);
        }
        
        // If not found in current scope, check the enclosing scope
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().is_mutable(name);
        }
        
        // Variable not found
        Err(WidowError::Runtime {
            message: format!("Undefined variable '{}'", name)
        })
    }
    
    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return true;
        }
        
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().contains(name);
        }
        
        false
    }
    
    /// Assign a value to an existing variable
    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        // Check if variable exists in current scope
        if let Some(var_cell) = self.variables.get(name) {
            // Replace the value
            *var_cell.borrow_mut() = value;
            return Ok(());
        }
        
        // If not in current scope, try enclosing scope
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        
        // Variable not found
        Err(WidowError::Runtime {
            message: format!("Undefined variable '{}'", name)
        })
    }
}

/// The Memory Manager handles the creation and management of environments
pub struct MemoryManager {
    /// Current environment
    current: Rc<RefCell<Environment>>,
    /// Global environment
    globals: Rc<RefCell<Environment>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        Self {
            current: Rc::clone(&globals),
            globals,
        }
    }
    
    /// Push a new scope
    pub fn push_scope(&mut self) {
        let new_env = Environment::with_enclosing(Rc::clone(&self.current));
        self.current = Rc::new(RefCell::new(new_env));
    }
    
    /// Pop the current scope
    pub fn pop_scope(&mut self) -> Result<()> {
        // Get the parent environment
        let parent = match &self.current.borrow().enclosing {
            Some(env) => Rc::clone(env),
            None => return Err(WidowError::Runtime {
                message: "Cannot pop global scope".to_string()
            }),
        };
        
        // Set current to parent
        self.current = parent;
        Ok(())
    }
    
    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, value: Value) {
        self.current.borrow_mut().define(name, value);
    }
    
    /// Get a copy of a variable's value
    pub fn get_value(&self, name: &str) -> Result<Value> {
        self.current.borrow().get_value(name)
    }
    
    /// Check if a variable is mutable
    pub fn is_mutable(&self, name: &str) -> Result<bool> {
        self.current.borrow().is_mutable(name)
    }
    
    /// Assign a value to an existing variable
    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        self.current.borrow_mut().assign(name, value)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implement conversion traits for Value

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                let arr_ref = arr.borrow();
                write!(f, "[")?;
                let mut first = true;
                for item in arr_ref.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            Value::Map(map) => {
                let map_ref = map.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map_ref.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            },
            Value::Struct(s) => {
                let s_ref = s.borrow();
                write!(f, "{}{{", s_ref.struct_name)?;
                let mut first = true;
                for (k, v) in s_ref.fields.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            },
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Nil => write!(f, "nil"),
        }
    }
}

