use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;
use widow_lib::memory::MemoryManager;
use widow_lib::{interpreter, lexer, parser};

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file to run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Widow source file
    Run {
        /// Input file
        file: PathBuf,
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Compile source to bytecode
    Compile {
        /// Input file
        file: PathBuf,
        /// Output file (defaults to input.wdb)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Run compiled bytecode
    Execute {
        /// Bytecode file
        file: PathBuf,
    },
    /// Compile to native executable
    Native {
        /// Input file
        file: PathBuf,
        /// Output executable name
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { file, verbose }) => {
            run_file(file, verbose);
        }
        Some(Commands::Compile { file, output }) => {
            compile_to_bytecode(file, output);
        }
        Some(Commands::Execute { file }) => {
            execute_bytecode(file);
        }
        Some(Commands::Native { file, output }) => {
            compile_to_native(file, output);
        }
        None => {
            if let Some(file) = cli.file {
                run_file(file, cli.verbose);
            } else {
                eprintln!(
                    "{} No file specified. Use --help for usage information.",
                    "Error:".bright_red()
                );
                process::exit(1);
            }
        }
    }
}

fn run_file(file_path: PathBuf, verbose: bool) {
    if verbose {
        println!(
            "{} Running Widow file: {}",
            "Info:".bright_blue(),
            file_path.display()
        );
    }

    if !file_path.exists() {
        eprintln!(
            "{} File not found: {}",
            "Error:".bright_red(),
            file_path.display()
        );
        process::exit(1);
    }

    // Read the source code
    let source = match fs::read_to_string(&file_path) {
        Ok(content) => {
            if verbose {
                println!("📄 File read successfully, {} bytes", content.len());
                println!("📄 Source code:");
                println!("'{}'", content);
                println!("{}", "─".repeat(50));
            }
            content
        }
        Err(err) => {
            eprintln!("{} Failed to read file: {}", "Error:".bright_red(), err);
            process::exit(1);
        }
    };

    // Tokenize the source code
    let tokens = match lexer::tokenize(&source) {
        Ok(tokens) => {
            if verbose {
                println!("✓ Tokenization successful ({} tokens)", tokens.len());
            }
            tokens
        }
        Err(err) => {
            eprintln!("{} Tokenization failed: {}", "Error:".bright_red(), err);
            process::exit(1);
        }
    };

    // Parse tokens into AST
    let ast = match parser::parse(tokens) {
        Ok(ast) => {
            if verbose {
                println!("✓ Parsing successful ({} statements)", ast.statements.len());
            }
            ast
        }
        Err(err) => {
            eprintln!("{} Parsing failed: {}", "Error:".bright_red(), err);
            process::exit(1);
        }
    };

    // Create memory manager for interpretation
    let mut memory = MemoryManager::new();

    // Execute the program
    if verbose {
        println!("🚀 Executing program...");
    }

    match interpreter::interpret_program(&ast, &mut memory) {
        Ok(_) => {
            if verbose {
                println!("{} Program executed successfully", "Success:".green());
            }
        }
        Err(err) => {
            eprintln!("{} {}", "Runtime error:".bright_red(), err);
            process::exit(1);
        }
    }
}

fn compile_to_bytecode(file_path: PathBuf, output: Option<PathBuf>) {
    eprintln!(
        "{} Bytecode compilation not yet implemented",
        "Error:".bright_red()
    );
    process::exit(1);
}

fn execute_bytecode(file_path: PathBuf) {
    eprintln!(
        "{} Bytecode execution not yet implemented",
        "Error:".bright_red()
    );
    process::exit(1);
}

fn compile_to_native(file_path: PathBuf, output: Option<PathBuf>) {
    eprintln!(
        "{} Native compilation not yet implemented",
        "Error:".bright_red()
    );
    process::exit(1);
}
