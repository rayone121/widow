// Widow Programming Language
// Interpreter module - AST-based interpreter implementation

use crate::ast;
use crate::memory::{MemoryManager, Value};

/// Simple interpreter implementation
pub fn interpret_program(program: &ast::Program, memory: &mut MemoryManager) -> Result<(), String> {
    for statement in &program.statements {
        interpret_statement(statement, memory)?;
    }
    Ok(())
}

pub fn interpret_statement(statement: &ast::Statement, memory: &mut MemoryManager) -> Result<(), String> {
    match statement {
        ast::Statement::Declaration(decl) => {
            interpret_declaration(decl, memory)?;
        }
        ast::Statement::Expression(expr_stmt) => {
            interpret_expression(&expr_stmt.expression, memory)?;
        }
        ast::Statement::Assignment(assign) => {
            let value = interpret_expression(&assign.value, memory)?;
            
            // Get the identifier from the target expression
            if let ast::Expression::Identifier(ident) = &assign.target {
                // Try to assign to existing variable, if not found, create new one
                match memory.assign(&ident.value, value.clone()) {
                    Ok(_) => println!("📝 Assigned '{}' = {}", ident.value, value),
                    Err(_) => {
                        memory.define(ident.value.clone(), value.clone());
                        println!("📝 Defined and assigned '{}' = {}", ident.value, value);
                    }
                }
            } else {
                return Err("Assignment target must be an identifier".to_string());
            }
        }
        ast::Statement::Block(block) => {
            interpret_block(block, memory)?;
        }
        ast::Statement::If(if_stmt) => {
            interpret_if_statement(if_stmt, memory)?;
        }
        ast::Statement::For(for_stmt) => {
            interpret_for_statement(for_stmt, memory)?;
        }
        ast::Statement::Switch(_) => {
            return Err("Switch statements not yet implemented".to_string());
        }
        ast::Statement::Return(_) => {
            return Err("Return statements not yet implemented".to_string());
        }
        ast::Statement::Break(_) => {
            return Err("Break statements not yet implemented".to_string());
        }
        ast::Statement::Continue(_) => {
            return Err("Continue statements not yet implemented".to_string());
        }
    }
    Ok(())
}

fn interpret_declaration(decl: &ast::Declaration, memory: &mut MemoryManager) -> Result<(), String> {
    match decl {
        ast::Declaration::Variable(var_decl) => {
            let value = if let Some(init) = &var_decl.value {
                interpret_expression(init, memory)?
            } else {
                Value::Nil
            };
            memory.define(var_decl.name.clone(), value);
            println!("📝 Defined variable '{}' = {}", var_decl.name, memory.get_value(&var_decl.name).unwrap());
        }
        ast::Declaration::Function(func_decl) => {
            // Extract parameter names
            let param_names: Vec<String> = func_decl.parameters.iter()
                .map(|p| p.name.clone())
                .collect();
            
            // Store the function in memory
            let function = crate::memory::Function {
                name: func_decl.name.clone(),
                arity: func_decl.parameters.len(),
                parameters: param_names,
                body: func_decl.body.clone(),
            };
            
            let func_value = Value::Function(std::rc::Rc::new(function));
            memory.define(func_decl.name.clone(), func_value);
            println!("📝 Defined function '{}'", func_decl.name);
        }
        ast::Declaration::Struct(_) => {
            return Err("Struct declarations not yet implemented".to_string());
        }
        ast::Declaration::Implementation(_) => {
            return Err("Implementation declarations not yet implemented".to_string());
        }
    }
    Ok(())
}

pub fn interpret_expression(expr: &ast::Expression, memory: &mut MemoryManager) -> Result<Value, String> {
    match expr {
        ast::Expression::Literal(lit) => {
            Ok(interpret_literal(lit))
        }
        ast::Expression::Identifier(var) => {
            memory.get_value(&var.value)
                .map_err(|e| format!("Variable access error: {}", e))
        }
        ast::Expression::Call(call) => {
            if let ast::Expression::Identifier(func_name) = &call.function.as_ref() {
                // Handle built-in functions
                if func_name.value == "print" {
                    if call.arguments.len() != 1 {
                        return Err("print() expects exactly 1 argument".to_string());
                    }
                    let arg_value = interpret_expression(&call.arguments[0], memory)?;
                    println!("{}", arg_value);
                    return Ok(Value::Nil);
                }
                
                // Try to get user-defined function
                match memory.get_value(&func_name.value) {
                    Ok(Value::Function(function)) => {
                        // Check argument count
                        if call.arguments.len() != function.arity {
                            return Err(format!(
                                "Function '{}' expects {} arguments, got {}",
                                function.name, function.arity, call.arguments.len()
                            ));
                        }
                        
                        // Evaluate arguments
                        let mut arg_values = Vec::new();
                        for arg_expr in &call.arguments {
                            arg_values.push(interpret_expression(arg_expr, memory)?);
                        }
                        
                        // Call the function
                        interpret_function_call(&function, arg_values, memory)
                    }
                    Ok(_) => {
                        Err(format!("'{}' is not a function", func_name.value))
                    }
                    Err(_) => {
                        Err(format!("Unknown function: {}", func_name.value))
                    }
                }
            } else {
                Err("Complex function calls not yet supported".to_string())
            }
        }
        ast::Expression::Infix(infix) => {
            let left = interpret_expression(&infix.left, memory)?;
            let right = interpret_expression(&infix.right, memory)?;
            interpret_infix_expression(&left, &infix.operator, &right)
        }
        ast::Expression::Prefix(prefix) => {
            let operand = interpret_expression(&prefix.right, memory)?;
            interpret_prefix_expression(&prefix.operator, &operand)
        }
        ast::Expression::Index(index) => {
            let left_val = interpret_expression(&index.left, memory)?;
            let index_val = interpret_expression(&index.index, memory)?;
            interpret_index_expression(left_val, index_val)
        }
        ast::Expression::Dot(_) => {
            Err("Dot expressions not yet implemented".to_string())
        }
        ast::Expression::Array(array) => {
            let mut elements = Vec::new();
            for element_expr in &array.elements {
                let element_val = interpret_expression(element_expr, memory)?;
                elements.push(element_val);
            }
            Ok(Value::Array(std::rc::Rc::new(std::cell::RefCell::new(elements))))
        }
        ast::Expression::HashMap(hashmap) => {
            let mut map = std::collections::HashMap::new();
            for (key_expr, value_expr) in &hashmap.pairs {
                let key_val = interpret_expression(key_expr, memory)?;
                let value_val = interpret_expression(value_expr, memory)?;
                
                // Convert key to string
                let key_str = match key_val {
                    Value::String(s) => s,
                    Value::Int(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Char(c) => c.to_string(),
                    _ => return Err("Invalid hashmap key type".to_string()),
                };
                
                map.insert(key_str, value_val);
            }
            Ok(Value::Map(std::rc::Rc::new(std::cell::RefCell::new(map))))
        }
        ast::Expression::StructInit(_) => {
            Err("Struct initialization not yet implemented".to_string())
        }
    }
}

fn interpret_infix_expression(left: &Value, operator: &ast::InfixOperator, right: &Value) -> Result<Value, String> {
    use ast::InfixOperator;
    
    match (left, operator, right) {
        // Arithmetic operations
        (Value::Int(l), InfixOperator::Plus, Value::Int(r)) => Ok(Value::Int(l + r)),
        (Value::Float(l), InfixOperator::Plus, Value::Float(r)) => Ok(Value::Float(l + r)),
        (Value::Int(l), InfixOperator::Plus, Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
        (Value::Float(l), InfixOperator::Plus, Value::Int(r)) => Ok(Value::Float(l + *r as f64)),
        
        (Value::Int(l), InfixOperator::Minus, Value::Int(r)) => Ok(Value::Int(l - r)),
        (Value::Float(l), InfixOperator::Minus, Value::Float(r)) => Ok(Value::Float(l - r)),
        (Value::Int(l), InfixOperator::Minus, Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
        (Value::Float(l), InfixOperator::Minus, Value::Int(r)) => Ok(Value::Float(l - *r as f64)),
        
        (Value::Int(l), InfixOperator::Multiply, Value::Int(r)) => Ok(Value::Int(l * r)),
        (Value::Float(l), InfixOperator::Multiply, Value::Float(r)) => Ok(Value::Float(l * r)),
        (Value::Int(l), InfixOperator::Multiply, Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
        (Value::Float(l), InfixOperator::Multiply, Value::Int(r)) => Ok(Value::Float(l * *r as f64)),
        
        (Value::Int(l), InfixOperator::Divide, Value::Int(r)) => {
            if *r == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(*l as f64 / *r as f64))
            }
        },
        (Value::Float(l), InfixOperator::Divide, Value::Float(r)) => {
            if *r == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(l / r))
            }
        },
        (Value::Int(l), InfixOperator::Divide, Value::Float(r)) => {
            if *r == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(*l as f64 / r))
            }
        },
        (Value::Float(l), InfixOperator::Divide, Value::Int(r)) => {
            if *r == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(l / *r as f64))
            }
        },
        
        (Value::Int(l), InfixOperator::Modulo, Value::Int(r)) => {
            if *r == 0 {
                Err("Modulo by zero".to_string())
            } else {
                Ok(Value::Int(l % r))
            }
        },
        
        // Comparison operations
        (Value::Int(l), InfixOperator::Equal, Value::Int(r)) => Ok(Value::Bool(l == r)),
        (Value::Float(l), InfixOperator::Equal, Value::Float(r)) => Ok(Value::Bool(l == r)),
        (Value::String(l), InfixOperator::Equal, Value::String(r)) => Ok(Value::Bool(l == r)),
        (Value::Bool(l), InfixOperator::Equal, Value::Bool(r)) => Ok(Value::Bool(l == r)),
        
        (Value::Int(l), InfixOperator::NotEqual, Value::Int(r)) => Ok(Value::Bool(l != r)),
        (Value::Float(l), InfixOperator::NotEqual, Value::Float(r)) => Ok(Value::Bool(l != r)),
        (Value::String(l), InfixOperator::NotEqual, Value::String(r)) => Ok(Value::Bool(l != r)),
        (Value::Bool(l), InfixOperator::NotEqual, Value::Bool(r)) => Ok(Value::Bool(l != r)),
        
        (Value::Int(l), InfixOperator::LessThan, Value::Int(r)) => Ok(Value::Bool(l < r)),
        (Value::Float(l), InfixOperator::LessThan, Value::Float(r)) => Ok(Value::Bool(l < r)),
        (Value::Int(l), InfixOperator::LessThan, Value::Float(r)) => Ok(Value::Bool((*l as f64) < *r)),
        (Value::Float(l), InfixOperator::LessThan, Value::Int(r)) => Ok(Value::Bool(*l < (*r as f64))),
        
        (Value::Int(l), InfixOperator::GreaterThan, Value::Int(r)) => Ok(Value::Bool(l > r)),
        (Value::Float(l), InfixOperator::GreaterThan, Value::Float(r)) => Ok(Value::Bool(l > r)),
        (Value::Int(l), InfixOperator::GreaterThan, Value::Float(r)) => Ok(Value::Bool((*l as f64) > *r)),
        (Value::Float(l), InfixOperator::GreaterThan, Value::Int(r)) => Ok(Value::Bool(*l > (*r as f64))),
        
        (Value::Int(l), InfixOperator::LessEqual, Value::Int(r)) => Ok(Value::Bool(l <= r)),
        (Value::Float(l), InfixOperator::LessEqual, Value::Float(r)) => Ok(Value::Bool(l <= r)),
        (Value::Int(l), InfixOperator::LessEqual, Value::Float(r)) => Ok(Value::Bool((*l as f64) <= *r)),
        (Value::Float(l), InfixOperator::LessEqual, Value::Int(r)) => Ok(Value::Bool(*l <= (*r as f64))),
        
        (Value::Int(l), InfixOperator::GreaterEqual, Value::Int(r)) => Ok(Value::Bool(l >= r)),
        (Value::Float(l), InfixOperator::GreaterEqual, Value::Float(r)) => Ok(Value::Bool(l >= r)),
        (Value::Int(l), InfixOperator::GreaterEqual, Value::Float(r)) => Ok(Value::Bool((*l as f64) >= *r)),
        (Value::Float(l), InfixOperator::GreaterEqual, Value::Int(r)) => Ok(Value::Bool(*l >= (*r as f64))),
        
        // Logical operations
        (Value::Bool(l), InfixOperator::And, Value::Bool(r)) => Ok(Value::Bool(*l && *r)),
        (Value::Bool(l), InfixOperator::Or, Value::Bool(r)) => Ok(Value::Bool(*l || *r)),
        
        // String concatenation
        (Value::String(l), InfixOperator::Plus, Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
        
        _ => Err(format!("Unsupported operation: {:?} {} {:?}", left, operator, right))
    }
}

fn interpret_prefix_expression(operator: &ast::PrefixOperator, operand: &Value) -> Result<Value, String> {
    use ast::PrefixOperator;
    
    match (operator, operand) {
        (PrefixOperator::Minus, Value::Int(val)) => Ok(Value::Int(-val)),
        (PrefixOperator::Minus, Value::Float(val)) => Ok(Value::Float(-val)),
        (PrefixOperator::Not, Value::Bool(val)) => Ok(Value::Bool(!val)),
        _ => Err(format!("Unsupported prefix operation: {} {:?}", operator, operand))
    }
}

fn interpret_literal(lit: &ast::LiteralExpression) -> Value {
    match lit {
        ast::LiteralExpression::Int { value, .. } => Value::Int(*value),
        ast::LiteralExpression::Float { value, .. } => Value::Float(*value),
        ast::LiteralExpression::String { value, .. } => Value::String(value.clone()),
        ast::LiteralExpression::Bool { value, .. } => Value::Bool(*value),
        ast::LiteralExpression::Char { value, .. } => Value::Char(*value),
        ast::LiteralExpression::Nil { .. } => Value::Nil,
    }
}

fn interpret_index_expression(left: Value, index: Value) -> Result<Value, String> {
    match (left, index) {
        // Array indexing with integer
        (Value::Array(arr), Value::Int(i)) => {
            let arr_ref = arr.borrow();
            if i < 0 {
                return Err("Array index cannot be negative".to_string());
            }
            let index = i as usize;
            if index >= arr_ref.len() {
                return Err(format!("Array index {} out of bounds (length {})", index, arr_ref.len()));
            }
            Ok(arr_ref[index].clone())
        }
        // Hashmap access with string key
        (Value::Map(map), Value::String(key)) => {
            let map_ref = map.borrow();
            match map_ref.get(&key) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Key '{}' not found in hashmap", key)),
            }
        }
        // Hashmap access with other key types (convert to string)
        (Value::Map(map), key_val) => {
            let key_str = match key_val {
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Char(c) => c.to_string(),
                _ => return Err("Invalid hashmap key type".to_string()),
            };
            
            let map_ref = map.borrow();
            match map_ref.get(&key_str) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Key '{}' not found in hashmap", key_str)),
            }
        }
        _ => Err("Invalid index operation".to_string()),
    }
}

fn interpret_block(block: &ast::BlockStatement, memory: &mut MemoryManager) -> Result<(), String> {
    // Push a new scope for the block
    memory.push_scope();
    
    // Execute all statements in the block
    let result = (|| {
        for statement in &block.statements {
            interpret_statement(statement, memory)?;
        }
        Ok(())
    })();
    
    // Pop the scope when done (even if there was an error)
    memory.pop_scope().map_err(|e| format!("Scope error: {}", e))?;
    
    result
}

fn interpret_if_statement(if_stmt: &ast::IfStatement, memory: &mut MemoryManager) -> Result<(), String> {
    // Evaluate the condition
    let condition_value = interpret_expression(&if_stmt.condition, memory)?;
    
    // Check if condition is truthy
    let is_truthy = match condition_value {
        Value::Bool(b) => b,
        Value::Nil => false,
        Value::Int(i) => i != 0,
        Value::Float(f) => f != 0.0,
        Value::String(s) => !s.is_empty(),
        _ => true, // Arrays, maps, etc. are truthy
    };
    
    if is_truthy {
        // Execute the consequence block
        interpret_block(&if_stmt.consequence, memory)?;
    } else if let Some(alternative) = &if_stmt.alternative {
        // Execute the alternative (else or elif)
        interpret_statement(alternative, memory)?;
    }
    
    Ok(())
}

/// Execute a user-defined function call
fn interpret_function_call(
    function: &crate::memory::Function,
    arguments: Vec<Value>,
    memory: &mut MemoryManager,
) -> Result<Value, String> {
    // Create new scope for function execution
    memory.push_scope();
    
    // Bind parameters to arguments
    for (param_name, arg_value) in function.parameters.iter().zip(arguments.iter()) {
        memory.define(param_name.clone(), arg_value.clone());
    }
    
    println!("🔧 Calling function '{}' with {} arguments", function.name, function.arity);
    
    // Execute function body
    let result = match interpret_block(&function.body, memory) {
        Ok(_) => Ok(Value::Nil), // No explicit return, return nil
        Err(err) => {
            // Check if this was a return statement (we'll implement this later)
            if err.starts_with("RETURN:") {
                // Extract return value from error message (temporary hack)
                // TODO: Implement proper return statement handling
                Ok(Value::Nil)
            } else {
                Err(err)
            }
        }
    };
    
    // Pop function scope
    memory.pop_scope().map_err(|e| format!("Scope error: {}", e))?;
    
    result
}

fn interpret_for_statement(for_stmt: &ast::ForStatement, memory: &mut MemoryManager) -> Result<(), String> {
    match for_stmt {
        ast::ForStatement::Condition { condition, body, .. } => {
            // While-style loop
            loop {
                let condition_value = interpret_expression(condition, memory)?;
                let is_truthy = match condition_value {
                    Value::Bool(b) => b,
                    Value::Nil => false,
                    Value::Int(i) => i != 0,
                    Value::Float(f) => f != 0.0,
                    Value::String(s) => !s.is_empty(),
                    _ => true,
                };
                
                if !is_truthy {
                    break;
                }
                
                interpret_block(body, memory)?;
            }
        }
        ast::ForStatement::Range { variable, start, end, body, .. } => {
            // Range-based loop: for i in 1..5
            let start_val = interpret_expression(start, memory)?;
            let end_val = interpret_expression(end, memory)?;
            
            let (start_int, end_int) = match (start_val, end_val) {
                (Value::Int(s), Value::Int(e)) => (s, e),
                _ => return Err("Range bounds must be integers".to_string()),
            };
            
            for i in start_int..end_int {
                memory.push_scope();
                memory.define(variable.clone(), Value::Int(i));
                let result = interpret_block(body, memory);
                memory.pop_scope().map_err(|e| format!("Scope error: {}", e))?;
                result?;
            }
        }
        ast::ForStatement::Iteration { variable, collection, body, .. } => {
            // Collection iteration: for item in array
            let collection_val = interpret_expression(collection, memory)?;
            
            match collection_val {
                Value::Array(arr) => {
                    let arr_ref = arr.borrow();
                    for item in arr_ref.iter() {
                        memory.push_scope();
                        memory.define(variable.clone(), item.clone());
                        let result = interpret_block(body, memory);
                        memory.pop_scope().map_err(|e| format!("Scope error: {}", e))?;
                        result?;
                    }
                }
                Value::Map(map) => {
                    let map_ref = map.borrow();
                    for key in map_ref.keys() {
                        memory.push_scope();
                        memory.define(variable.clone(), Value::String(key.clone()));
                        let result = interpret_block(body, memory);
                        memory.pop_scope().map_err(|e| format!("Scope error: {}", e))?;
                        result?;
                    }
                }
                _ => return Err("Can only iterate over arrays and maps".to_string()),
            }
        }
    }
    
    Ok(())
}
