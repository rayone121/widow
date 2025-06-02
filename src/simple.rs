// Widow Programming Language
// Simple module for basic Hello World example

use std::path::Path;
use std::fs;
use crate::error::Result;
use crate::memory::Value;
use crate::bytecode::{BytecodeModule, Chunk, Opcode};
use crate::vm;

/// Run a simple Hello World program without using the full parser/compiler stack
pub fn run_hello_world<P: AsRef<Path>>(path: P) -> Result<()> {
    let source = fs::read_to_string(path)?;
    
    // Create a simple bytecode program that just prints "Hello, World!"
    let module = create_hello_world_bytecode(&source)?;
    
    // Execute the bytecode
    vm::execute(module)
}

/// Create bytecode for simple print statements
fn create_hello_world_bytecode(source: &str) -> Result<BytecodeModule> {
    let mut module = BytecodeModule::new();
    
    // Process each line
    for line in source.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse simple print("Hello, World!") statements
        if line.starts_with("print(\"") && line.ends_with("\")") {
            // Extract the string content between quotes
            let content = &line[7..line.len()-2]; // Skip print(" and ")
            
            // Create a constant for the string
            let constant_idx = module.current_chunk().add_constant(Value::String(content.to_string()));
            
            // Emit CONSTANT instruction to load the string
            module.current_chunk().write(Opcode::Constant as u8, 1);
            module.current_chunk().write(constant_idx, 1);
            
            // Emit PRINT instruction
            module.current_chunk().write(Opcode::Print as u8, 1);
        }
    }
    
    // Always end with a RETURN instruction
    module.current_chunk().write(Opcode::Return as u8, 1);
    
    Ok(module)
}