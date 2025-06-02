// Widow Programming Language
// Bytecode module for compilation and execution

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::ast::{Program, Statement, Expression, Declaration, ExpressionStatement, AssignmentStatement, LiteralExpression, InfixExpression, InfixOperator, PrefixExpression, PrefixOperator, IdentifierExpression, CallExpression};
use crate::error::{Result, WidowError};
use crate::memory::Value;

/// Widow bytecode format version
const BYTECODE_VERSION: u8 = 1;

/// Opcodes for the Widow VM
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Noop = 0,
    Constant = 1,
    Add = 2,
    Subtract = 3,
    Multiply = 4,
    Divide = 5,
    Negate = 6,
    Not = 7,
    Equal = 8,
    NotEqual = 9,
    Greater = 10,
    GreaterEqual = 11,
    Less = 12,
    LessEqual = 13,
    Jump = 14,
    JumpIfFalse = 15,
    Call = 16,
    Return = 17,
    Pop = 18,
    GetLocal = 19,
    SetLocal = 20,
    GetGlobal = 21,
    SetGlobal = 22,
    DefineGlobal = 23,  // Define a global variable
    BorrowShared = 24,  // Create a shared borrow
    BorrowMut = 25,     // Create a mutable borrow
    ReleaseBorrow = 26, // Release a borrow
    PushScope = 27,     // Push a new scope
    PopScope = 28,      // Pop the current scope
    Array = 29,
    GetIndex = 30,
    SetIndex = 31,
    GetField = 32,
    SetField = 33,
    Print = 34,         // Print a value
    Modulo = 35,        // Modulo operation
}

/// Bytecode chunk representing a unit of compiled code
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub line_info: Vec<usize>,
    pub locals: Vec<String>,      // Local variable names
    pub upvalues: Vec<String>,    // Variables from outer scopes
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            line_info: Vec::new(),
            locals: Vec::new(),
            upvalues: Vec::new(),
        }
    }
    
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.line_info.push(line);
    }
    
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }
}

/// Compiled bytecode module
#[derive(Debug, Clone)]
pub struct BytecodeModule {
    pub chunks: Vec<Chunk>,
    pub main_chunk: usize,
}

impl BytecodeModule {
    pub fn new() -> Self {
        let main_chunk = Chunk::new();
        Self {
            chunks: vec![main_chunk],
            main_chunk: 0,
        }
    }
    
    pub fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunks[self.main_chunk]
    }
}

/// Compiler state for generating bytecode from AST
struct Compiler {
    module: BytecodeModule,
    globals: HashMap<String, usize>, // Map global names to their index in constants
    scope_depth: usize,
    locals: Vec<Local>,
}

/// Local variable for tracking
struct Local {
    name: String,
    depth: usize,
    initialized: bool,
}

impl Compiler {
    fn new() -> Self {
        Self {
            module: BytecodeModule::new(),
            globals: HashMap::new(),
            scope_depth: 0,
            locals: Vec::new(),
        }
    }
    
    fn compile(&mut self, program: Program) -> Result<BytecodeModule> {
        // Compile each statement
        for statement in program.statements {
            self.compile_statement(&statement)?;
        }
        
        // Every program must end with a return statement
        let chunk = self.module.current_chunk();
        chunk.write(Opcode::Return as u8, 0);
        
        Ok(self.module.clone())
    }
    
    fn compile_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.compile_expression(&expr_stmt.expression)?;
                // Pop the value if it's not used
                self.emit_byte(Opcode::Pop as u8, expr_stmt.node.line);
            },
            Statement::Declaration(decl) => {
                self.compile_declaration(decl)?;
            },
            Statement::Assignment(assign) => {
                self.compile_assignment(assign)?;
            },
            // For now, we'll implement just the basics needed for a Hello World
            _ => {
                return Err(WidowError::Runtime {
                    message: format!("Statement type not yet implemented for compilation")
                });
            }
        }
        
        Ok(())
    }
    
    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Variable(var_decl) => {
                // If there's an initializer, compile it
                if let Some(init) = &var_decl.value {
                    self.compile_expression(init)?;
                } else {
                    // Default to nil if no initializer
                    self.emit_constant(Value::Nil, var_decl.node.line)?;
                }
                
                // Define the variable in the appropriate scope
                if self.scope_depth > 0 {
                    // Local variable
                    self.add_local(&var_decl.name);
                    // The variable's value is already on the stack
                } else {
                    // Global variable
                    let name_idx = self.make_constant(Value::String(var_decl.name.clone()), var_decl.node.line)?;
                    self.emit_bytes(Opcode::DefineGlobal as u8, name_idx, var_decl.node.line);
                }
            },
            // For now, we'll implement just the basics needed for a Hello World
            _ => {
                return Err(WidowError::Runtime {
                    message: format!("Declaration type not yet implemented for compilation")
                });
            }
        }
        
        Ok(())
    }
    
    fn compile_assignment(&mut self, assignment: &AssignmentStatement) -> Result<()> {
        // Compile the value to be assigned
        self.compile_expression(&assignment.value)?;
        
        // Handle the assignment target
        match &assignment.target {
            Expression::Identifier(ident) => {
                // Check if it's a local variable first
                if let Some(local_idx) = self.resolve_local(&ident.value) {
                    self.emit_bytes(Opcode::SetLocal as u8, local_idx as u8, assignment.node.line);
                } else {
                    // Global variable
                    let name_idx = self.make_constant(Value::String(ident.value.clone()), assignment.node.line)?;
                    self.emit_bytes(Opcode::SetGlobal as u8, name_idx, assignment.node.line);
                }
            },
            // For now, we'll implement just the basics needed for a Hello World
            _ => {
                return Err(WidowError::Runtime {
                    message: format!("Assignment target type not yet implemented for compilation")
                });
            }
        }
        
        Ok(())
    }
    
    fn compile_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::Literal(lit) => {
                self.compile_literal(lit)?;
            },
            Expression::Identifier(ident) => {
                self.compile_identifier(ident)?;
            },
            Expression::Infix(infix) => {
                self.compile_infix(infix)?;
            },
            Expression::Prefix(prefix) => {
                self.compile_prefix(prefix)?;
            },
            Expression::Call(call) => {
                self.compile_call(call)?;
            },
            // For now, we'll implement just the basics needed for a Hello World
            _ => {
                return Err(WidowError::Runtime {
                    message: format!("Expression type not yet implemented for compilation")
                });
            }
        }
        
        Ok(())
    }
    
    fn compile_literal(&mut self, literal: &LiteralExpression) -> Result<()> {
        match literal {
            LiteralExpression::Int { value, node } => {
                self.emit_constant(Value::Int(*value), node.line)?;
            },
            LiteralExpression::Float { value, node } => {
                self.emit_constant(Value::Float(*value), node.line)?;
            },
            LiteralExpression::String { value, node } => {
                self.emit_constant(Value::String(value.clone()), node.line)?;
            },
            LiteralExpression::Bool { value, node } => {
                self.emit_constant(Value::Bool(*value), node.line)?;
            },
            LiteralExpression::Char { value, node } => {
                self.emit_constant(Value::Char(*value), node.line)?;
            },
            LiteralExpression::Nil { node } => {
                self.emit_constant(Value::Nil, node.line)?;
            },
        }
        
        Ok(())
    }
    
    fn compile_identifier(&mut self, identifier: &IdentifierExpression) -> Result<()> {
        // Check if it's a local variable first
        if let Some(local_idx) = self.resolve_local(&identifier.value) {
            self.emit_bytes(Opcode::GetLocal as u8, local_idx as u8, identifier.node.line);
        } else {
            // Look for a global variable
            let name_idx = self.make_constant(Value::String(identifier.value.clone()), identifier.node.line)?;
            self.emit_bytes(Opcode::GetGlobal as u8, name_idx, identifier.node.line);
        }
        
        Ok(())
    }
    
    fn compile_infix(&mut self, infix: &InfixExpression) -> Result<()> {
        // Compile left and right expressions
        self.compile_expression(&infix.left)?;
        self.compile_expression(&infix.right)?;
        
        // Emit the operation
        let opcode = match infix.operator {
            InfixOperator::Plus => Opcode::Add,
            InfixOperator::Minus => Opcode::Subtract,
            InfixOperator::Multiply => Opcode::Multiply,
            InfixOperator::Divide => Opcode::Divide,
            InfixOperator::Modulo => Opcode::Modulo,
            InfixOperator::Equal => Opcode::Equal,
            InfixOperator::NotEqual => Opcode::NotEqual,
            InfixOperator::LessThan => Opcode::Less,
            InfixOperator::GreaterThan => Opcode::Greater,
            InfixOperator::LessEqual => Opcode::LessEqual,
            InfixOperator::GreaterEqual => Opcode::GreaterEqual,
            InfixOperator::And => Opcode::JumpIfFalse,  // We'd need more complex handling for short-circuiting
            InfixOperator::Or => Opcode::JumpIfFalse,   // We'd need more complex handling for short-circuiting
        };
        
        self.emit_byte(opcode as u8, infix.node.line);
        
        Ok(())
    }
    
    fn compile_prefix(&mut self, prefix: &PrefixExpression) -> Result<()> {
        // Compile the operand
        self.compile_expression(&prefix.right)?;
        
        // Emit the operation
        let opcode = match prefix.operator {
            PrefixOperator::Minus => Opcode::Negate,
            PrefixOperator::Not => Opcode::Not,
        };
        
        self.emit_byte(opcode as u8, prefix.node.line);
        
        Ok(())
    }
    
    fn compile_call(&mut self, call: &CallExpression) -> Result<()> {
        // Handle special case for print function
        if let Expression::Identifier(ident) = &call.function as &Expression {
            if ident.value == "print" {
                // Compile arguments
                for arg in &call.arguments {
                    self.compile_expression(arg)?;
                }
                
                // Emit print opcode
                self.emit_byte(Opcode::Print as u8, call.node.line);
                return Ok(());
            }
        }
        
        // Compile the function expression
        self.compile_expression(&call.function)?;
        
        // Compile each argument
        for arg in &call.arguments {
            self.compile_expression(arg)?;
        }
        
        // Emit the call instruction with argument count
        self.emit_bytes(Opcode::Call as u8, call.arguments.len() as u8, call.node.line);
        
        Ok(())
    }
    
    // Helper methods for bytecode emission
    
    fn emit_byte(&mut self, byte: u8, line: usize) {
        let chunk = self.module.current_chunk();
        chunk.write(byte, line);
    }
    
    fn emit_bytes(&mut self, byte1: u8, byte2: u8, line: usize) {
        self.emit_byte(byte1, line);
        self.emit_byte(byte2, line);
    }
    
    fn emit_constant(&mut self, value: Value, line: usize) -> Result<()> {
        let idx = self.make_constant(value, line)?;
        self.emit_bytes(Opcode::Constant as u8, idx, line);
        Ok(())
    }
    
    fn make_constant(&mut self, value: Value, line: usize) -> Result<u8> {
        let chunk = self.module.current_chunk();
        let idx = chunk.add_constant(value);
        
        if idx > u8::MAX as u8 {
            return Err(WidowError::Runtime {
                message: format!("Too many constants in one chunk at line {}", line)
            });
        }
        
        Ok(idx)
    }
    
    // Scope management
    
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }
    
    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        
        // Remove all local variables from this scope
        while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit_byte(Opcode::Pop as u8, 0); // Line info not important for pops
            self.locals.pop();
        }
    }
    
    fn add_local(&mut self, name: &str) {
        self.locals.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
            initialized: false,
        });
    }
    
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        
        None
    }
}

/// Compile AST to bytecode
pub fn compile(ast: Program) -> Result<BytecodeModule> {
    let mut compiler = Compiler::new();
    compiler.compile(ast)
}

/// Save bytecode to a file
pub fn save<P: AsRef<Path>>(bytecode: &BytecodeModule, path: P) -> Result<()> {
    let mut file = File::create(path)?;
    
    // Write magic number "WDBC" (Widow ByteCode)
    file.write_all(b"WDBC")?;
    
    // Write version
    file.write_all(&[BYTECODE_VERSION])?;
    
    // Write the main chunk index
    file.write_all(&(bytecode.main_chunk as u32).to_le_bytes())?;
    
    // Write number of chunks
    file.write_all(&(bytecode.chunks.len() as u32).to_le_bytes())?;
    
    // Write each chunk
    for chunk in &bytecode.chunks {
        // Write code length
        file.write_all(&(chunk.code.len() as u32).to_le_bytes())?;
        
        // Write code
        file.write_all(&chunk.code)?;
        
        // Write constants count
        file.write_all(&(chunk.constants.len() as u32).to_le_bytes())?;
        
        // TODO: Write constants
        // This would require serializing Value objects
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Program;
    
    #[test]
    fn test_compile_empty_program() {
        let program = Program { statements: vec![] };
        let result = compile(program);
        assert!(result.is_ok());
    }
}