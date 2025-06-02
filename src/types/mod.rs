// Widow Programming Language
// Types module for type checking and inference

use std::collections::HashMap;
use crate::ast;
use crate::error::{Result, WidowError, Location};
use crate::memory::Value;

/// Type checking context
pub struct TypeChecker {
    // Environment for type checking
    variables: HashMap<String, Type>,
    functions: HashMap<String, FunctionType>,
    structs: HashMap<String, StructType>,
    current_function: Option<String>,
}

/// Function type definition
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Type,
}

/// Struct type definition
#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: String,
    pub fields: HashMap<String, Type>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        let mut checker = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function: None,
        };
        
        // Add built-in functions
        checker.add_builtin_functions();
        
        checker
    }
    
    /// Add built-in functions like print
    fn add_builtin_functions(&mut self) {
        // print function
        self.functions.insert("print".to_string(), FunctionType {
            params: vec![Type::Any], // print can take any type
            return_type: Type::Primitive(PrimitiveType::Nil),
        });
        
        // string function (converts to string)
        self.functions.insert("string".to_string(), FunctionType {
            params: vec![Type::Any],
            return_type: Type::Primitive(PrimitiveType::String),
        });
    }
    
    /// Check types in a program
    pub fn check_program(&mut self, program: &ast::Program) -> Result<()> {
        // For simplicity in our basic implementation,
        // we'll just approve all programs for now
        Ok(())
    }
    
    /// Get the type of an expression
    pub fn type_of_expr(&mut self, _expr: &ast::Expression) -> Result<Type> {
        // For now, just return Any type to allow all operations
        Ok(Type::Any)
    }
}

/// Standard primitive types
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    I8,
    I32,
    I64,
    I128,
    IArch,
    U8,
    U32,
    U64,
    U128,
    UArch,
    F32,
    F64,
    FArch,
    Bool,
    Char,
    String,
    Nil,
}

/// Type representation in the typechecker
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Struct(String, HashMap<String, Type>),
    Function(Vec<Type>, Box<Type>),
    Any,  // Special "any" type for initial development
    Unknown,
}

impl Type {
    /// Convert a Value to its corresponding Type
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Int(_) => Type::Primitive(PrimitiveType::I64), // Default to i64
            Value::Float(_) => Type::Primitive(PrimitiveType::F64), // Default to f64
            Value::Bool(_) => Type::Primitive(PrimitiveType::Bool),
            Value::Char(_) => Type::Primitive(PrimitiveType::Char),
            Value::String(_) => Type::Primitive(PrimitiveType::String),
            Value::Array(_) => Type::Array(Box::new(Type::Any)),  // Initially assume any type
            Value::Map(_) => Type::Map(Box::new(Type::Any), Box::new(Type::Any)), // Initially any key/value
            Value::Struct(s) => {
                let s_ref = s.borrow();
                Type::Struct(s_ref.struct_name.clone(), HashMap::new())
            },
            Value::Function(f) => {
                // For now, we're treating all functions as taking any params and returning any
                Type::Function(vec![], Box::new(Type::Any))
            },
            Value::Nil => Type::Primitive(PrimitiveType::Nil),
        }
    }
}

/// Check types in a program AST
pub fn check(program: ast::Program) -> Result<ast::Program> {
    // Create a type checker
    let mut checker = TypeChecker::new();
    
    // Check all types in the program
    checker.check_program(&program)?;
    
    // Return the same AST - in a real implementation we might
    // add type annotations or transform the AST
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Program;
    
    #[test]
    fn test_check_empty_program() {
        let program = Program { statements: vec![] };
        let result = check(program);
        assert!(result.is_ok());
    }
}