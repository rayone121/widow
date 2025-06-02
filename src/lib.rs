//! Widow Programming Language
//! 
//! Widow is a programming language combining Python/Go-like syntax with Rust safety.
//! This library implements the lexer, parser, type checker, and VM for Widow.

pub mod ast;
pub mod bytecode;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod memory;
pub mod parser;
pub mod simple;
pub mod types;
pub mod vm;

use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use crate::simple::run_hello_world;

/// Version of the Widow language implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compile and run a Widow program from a file
pub fn run_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let source = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;
    
    run(&source)
}

/// Compile and run Widow code from a string
pub fn run(source: &str) -> Result<()> {
    // Phase 1: Lexical analysis
    let tokens = lexer::tokenize(source).context("Lexical analysis failed")?;
    
    // Phase 2: Parsing
    let ast = parser::parse(tokens).context("Parsing failed")?;
    
    // Phase 3: Type checking
    let typed_ast = types::check(ast).context("Type checking failed")?;
    
    // Phase 4: Bytecode generation
    let bytecode = bytecode::compile(typed_ast).context("Bytecode compilation failed")?;
    
    // Phase 5: Execution
    vm::execute(bytecode).context("Execution failed")?;
    
    Ok(())
}

/// Run a file using the simplified Hello World processor
/// This is used for demonstration before the full compiler is ready
pub fn run_hello<P: AsRef<Path>>(path: P) -> Result<()> {
    run_hello_world(path).context("Failed to run hello world program")?;
    Ok(())
}

/// Compile a Widow program to bytecode without executing
pub fn compile_file<P: AsRef<Path>>(path: P, output: Option<P>) -> Result<()> {
    let source = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;
    
    let tokens = lexer::tokenize(&source).context("Lexical analysis failed")?;
    let ast = parser::parse(tokens).context("Parsing failed")?;
    let typed_ast = types::check(ast).context("Type checking failed")?;
    let bytecode = bytecode::compile(typed_ast).context("Bytecode compilation failed")?;
    
    // Determine the output path
    let output_path = match output {
        Some(path) => path.as_ref().to_path_buf(),
        None => {
            let mut path = path.as_ref().to_path_buf();
            path.set_extension("wdb");
            path
        }
    };
    
    bytecode::save(&bytecode, output_path).context("Failed to save bytecode")?;
    
    Ok(())
}