// Widow Programming Language
// Virtual Machine module for bytecode execution

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use crate::bytecode::{BytecodeModule, Chunk, Opcode};
use crate::error::{Result, WidowError};
use crate::memory::{Value, MemoryManager, Environment};

/// The Widow Virtual Machine
pub struct VM {
    /// Bytecode module being executed
    module: BytecodeModule,
    
    /// Current instruction pointer
    ip: usize,
    
    /// Current chunk being executed
    current_chunk: usize,
    
    /// Value stack
    stack: Vec<Value>,
    
    /// Memory manager (handles variables and scopes)
    memory: MemoryManager,
    
    /// Call stack
    frames: Vec<CallFrame>,
    
    /// Currently active borrows (tracks borrow state for error reporting)
    active_borrows: HashMap<String, BorrowState>,
}

/// Tracks the state of borrows for a variable
enum BorrowState {
    /// No active borrows
    None,
    /// Immutably borrowed, with count of active borrows
    Shared(usize),
    /// Mutably borrowed (exclusive)
    Exclusive,
}

/// Call frame for function calls
struct CallFrame {
    /// Return address
    return_ip: usize,
    
    /// Return chunk
    return_chunk: usize,
    
    /// Base pointer for local variables
    bp: usize,
}

impl VM {
    /// Create a new VM with the given bytecode module
    pub fn new(module: BytecodeModule) -> Self {
        Self {
            module,
            ip: 0,
            current_chunk: 0,
            stack: Vec::with_capacity(256),
            memory: MemoryManager::new(),
            frames: Vec::new(),
            active_borrows: HashMap::new(),
        }
    }
    
    /// Run the VM until completion
    pub fn run(&mut self) -> Result<Value> {
        // Set initial chunk to main chunk
        self.current_chunk = self.module.main_chunk;
        
        // Execute bytecode instructions
        loop {
            // Check if we've reached the end of the chunk
            if self.ip >= self.chunk().code.len() {
                // If there are no frames left, we're done
                if self.frames.is_empty() {
                    // Return the top value on the stack or nil
                    return Ok(self.stack.pop().unwrap_or(Value::Nil));
                }
                
                // Return to the caller
                let frame = self.frames.pop().unwrap();
                self.ip = frame.return_ip;
                self.current_chunk = frame.return_chunk;
                continue;
            }
            
            // Fetch the next instruction
            let instruction = self.read_byte();
            
            // Execute the instruction
            match instruction {
                byte if byte == Opcode::Noop as u8 => {
                    // Do nothing
                }
                
                byte if byte == Opcode::Constant as u8 => {
                    let constant_idx = self.read_byte() as usize;
                    let constant = self.chunk().constants[constant_idx].clone();
                    self.push(constant);
                }
                
                byte if byte == Opcode::GetGlobal as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid global variable name".to_string() 
                        }),
                    };
                    
                    // Get variable from memory system
                    let value = self.memory.get_value(&name)?;
                    self.push(value);
                }
                
                byte if byte == Opcode::SetGlobal as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid global variable name".to_string() 
                        }),
                    };
                    
                    let value = self.peek(0)?;
                    self.memory.assign(&name, value)?;
                }
                
                byte if byte == Opcode::DefineGlobal as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid global variable name".to_string() 
                        }),
                    };
                    
                    let value = self.pop()?;
                    self.memory.define(name, value);
                }
                
                byte if byte == Opcode::PushScope as u8 => {
                    self.memory.push_scope();
                }
                
                byte if byte == Opcode::PopScope as u8 => {
                    self.memory.pop_scope()?;
                }
                
                byte if byte == Opcode::BorrowShared as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid variable name".to_string() 
                        }),
                    };
                    
                    // Create a shared borrow
                    self.create_shared_borrow(&name)?;
                    
                    // Get the value
                    let value = self.memory.get_value(&name)?;
                    self.push(value);
                }
                
                byte if byte == Opcode::BorrowMut as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid variable name".to_string() 
                        }),
                    };
                    
                    // Create a mutable borrow
                    self.create_mutable_borrow(&name)?;
                    
                    // Check if variable is mutable
                    let is_mutable = self.memory.is_mutable(&name)?;
                    if !is_mutable {
                        return Err(WidowError::Runtime {
                            message: format!("Cannot mutably borrow immutable variable '{}'", name)
                        });
                    }
                    
                    // Get the value
                    let value = self.memory.get_value(&name)?;
                    self.push(value);
                }
                
                byte if byte == Opcode::ReleaseBorrow as u8 => {
                    let name_idx = self.read_byte() as usize;
                    let name = match &self.chunk().constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => return Err(WidowError::Runtime { 
                            message: "Invalid variable name".to_string() 
                        }),
                    };
                    
                    // Release the borrow
                    self.release_borrow(&name)?;
                }
                
                byte if byte == Opcode::Print as u8 => {
                    // Pop the value to print from the stack
                    let value = self.pop()?;
                    // Print it
                    println!("{}", value);
                }
                
                byte if byte == Opcode::Add as u8 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    
                    match (&a, &b) {
                        (Value::Int(a_val), Value::Int(b_val)) => {
                            self.push(Value::Int(a_val + b_val));
                        },
                        (Value::Int(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(*a_val as f64 + b_val));
                        },
                        (Value::Float(a_val), Value::Int(b_val)) => {
                            self.push(Value::Float(a_val + *b_val as f64));
                        },
                        (Value::Float(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(a_val + b_val));
                        },
                        (Value::String(a_val), Value::String(b_val)) => {
                            self.push(Value::String(a_val.clone() + b_val));
                        },
                        _ => {
                            return Err(WidowError::Runtime {
                                message: format!("Cannot add values of types {:?} and {:?}", a, b)
                            });
                        }
                    }
                }
                
                byte if byte == Opcode::Subtract as u8 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    
                    match (&a, &b) {
                        (Value::Int(a_val), Value::Int(b_val)) => {
                            self.push(Value::Int(a_val - b_val));
                        },
                        (Value::Int(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(*a_val as f64 - b_val));
                        },
                        (Value::Float(a_val), Value::Int(b_val)) => {
                            self.push(Value::Float(a_val - *b_val as f64));
                        },
                        (Value::Float(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(a_val - b_val));
                        },
                        _ => {
                            return Err(WidowError::Runtime {
                                message: format!("Cannot subtract values of types {:?} and {:?}", a, b)
                            });
                        }
                    }
                }
                
                byte if byte == Opcode::Multiply as u8 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    
                    match (&a, &b) {
                        (Value::Int(a_val), Value::Int(b_val)) => {
                            self.push(Value::Int(a_val * b_val));
                        },
                        (Value::Int(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(*a_val as f64 * b_val));
                        },
                        (Value::Float(a_val), Value::Int(b_val)) => {
                            self.push(Value::Float(a_val * *b_val as f64));
                        },
                        (Value::Float(a_val), Value::Float(b_val)) => {
                            self.push(Value::Float(a_val * b_val));
                        },
                        _ => {
                            return Err(WidowError::Runtime {
                                message: format!("Cannot multiply values of types {:?} and {:?}", a, b)
                            });
                        }
                    }
                }
                
                byte if byte == Opcode::Divide as u8 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    
                    match (&a, &b) {
                        (Value::Int(a_val), Value::Int(b_val)) => {
                            if *b_val == 0 {
                                return Err(WidowError::Runtime {
                                    message: "Division by zero".to_string()
                                });
                            }
                            self.push(Value::Int(a_val / b_val));
                        },
                        (Value::Int(a_val), Value::Float(b_val)) => {
                            if *b_val == 0.0 {
                                return Err(WidowError::Runtime {
                                    message: "Division by zero".to_string()
                                });
                            }
                            self.push(Value::Float(*a_val as f64 / b_val));
                        },
                        (Value::Float(a_val), Value::Int(b_val)) => {
                            if *b_val == 0 {
                                return Err(WidowError::Runtime {
                                    message: "Division by zero".to_string()
                                });
                            }
                            self.push(Value::Float(a_val / *b_val as f64));
                        },
                        (Value::Float(a_val), Value::Float(b_val)) => {
                            if *b_val == 0.0 {
                                return Err(WidowError::Runtime {
                                    message: "Division by zero".to_string()
                                });
                            }
                            self.push(Value::Float(a_val / b_val));
                        },
                        _ => {
                            return Err(WidowError::Runtime {
                                message: format!("Cannot divide values of types {:?} and {:?}", a, b)
                            });
                        }
                    }
                }
                
                byte if byte == Opcode::Modulo as u8 => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    
                    match (&a, &b) {
                        (Value::Int(a_val), Value::Int(b_val)) => {
                            if *b_val == 0 {
                                return Err(WidowError::Runtime {
                                    message: "Modulo by zero".to_string()
                                });
                            }
                            self.push(Value::Int(a_val % b_val));
                        },
                        _ => {
                            return Err(WidowError::Runtime {
                                message: format!("Modulo operation only supported on integers, got {:?} and {:?}", a, b)
                            });
                        }
                    }
                }
                
                byte if byte == Opcode::Return as u8 => {
                    // If there are no frames left, we're done
                    if self.frames.is_empty() {
                        // Return the top value on the stack or nil
                        return Ok(self.stack.pop().unwrap_or(Value::Nil));
                    }
                    
                    // Return to the caller
                    let frame = self.frames.pop().unwrap();
                    self.ip = frame.return_ip;
                    self.current_chunk = frame.return_chunk;
                }
                
                _ => {
                    return Err(WidowError::Runtime { 
                        message: format!("Unknown opcode: {}", instruction) 
                    });
                }
            }
        }
    }
    
    /// Get the current chunk
    fn chunk(&self) -> &Chunk {
        &self.module.chunks[self.current_chunk]
    }
    
    /// Read a byte from the current chunk and advance the ip
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk().code[self.ip];
        self.ip += 1;
        byte
    }
    

    
    /// Pop a value from the stack
    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| WidowError::Runtime {
            message: "Stack underflow".to_string()
        })
    }
    
    /// Push a value onto the stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
}

/// Execute a bytecode module
pub fn execute(bytecode: BytecodeModule) -> Result<()> {
    let mut vm = VM::new(bytecode);
    vm.run()?;
    Ok(())
}

/// Helper functions for manipulating the stack
impl VM {
    /// Peek at a value in the stack without removing it
    /// 0 is the top of the stack, 1 is the next value, etc.
    fn peek(&self, distance: usize) -> Result<Value> {
        let idx = self.stack.len().checked_sub(1 + distance);
        match idx {
            Some(i) if i < self.stack.len() => Ok(self.stack[i].clone()),
            _ => Err(WidowError::Runtime {
                message: "Stack underflow".to_string()
            }),
        }
    }
    
    /// Create a shared (immutable) borrow of a variable
    fn create_shared_borrow(&mut self, name: &str) -> Result<()> {
        let entry = self.active_borrows.entry(name.to_string()).or_insert(BorrowState::None);
        
        match entry {
            BorrowState::None => {
                *entry = BorrowState::Shared(1);
                Ok(())
            },
            BorrowState::Shared(count) => {
                *count += 1;
                Ok(())
            },
            BorrowState::Exclusive => {
                Err(WidowError::Runtime {
                    message: format!("Cannot borrow '{}' as immutable because it is already borrowed as mutable", name)
                })
            }
        }
    }
    
    /// Create a mutable (exclusive) borrow of a variable
    fn create_mutable_borrow(&mut self, name: &str) -> Result<()> {
        let entry = self.active_borrows.entry(name.to_string()).or_insert(BorrowState::None);
        
        match entry {
            BorrowState::None => {
                *entry = BorrowState::Exclusive;
                Ok(())
            },
            BorrowState::Shared(_) => {
                Err(WidowError::Runtime {
                    message: format!("Cannot borrow '{}' as mutable because it is already borrowed as immutable", name)
                })
            },
            BorrowState::Exclusive => {
                Err(WidowError::Runtime {
                    message: format!("Cannot borrow '{}' as mutable because it is already borrowed as mutable", name)
                })
            }
        }
    }
    
    /// Release a borrow (either shared or exclusive)
    fn release_borrow(&mut self, name: &str) -> Result<()> {
        if let Some(borrow_state) = self.active_borrows.get_mut(name) {
            match borrow_state {
                BorrowState::None => {
                    // No borrow to release
                    Ok(())
                },
                BorrowState::Shared(1) => {
                    // Last shared borrow
                    *borrow_state = BorrowState::None;
                    Ok(())
                },
                BorrowState::Shared(count) => {
                    // Decrement count
                    *count -= 1;
                    Ok(())
                },
                BorrowState::Exclusive => {
                    // Release exclusive borrow
                    *borrow_state = BorrowState::None;
                    Ok(())
                }
            }
        } else {
            // No borrow to release
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeModule, Chunk};
    
    #[test]
    fn test_execute_empty_module() {
        let module = BytecodeModule::new();
        let result = execute(module);
        assert!(result.is_ok());
    }
}