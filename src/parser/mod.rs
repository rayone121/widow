// Widow Programming Language
// Parser module for translating tokens into AST

use crate::ast;
use crate::error::{WidowError, Result};
use crate::lexer::{Token, TokenKind};

/// Parse tokens into an AST
pub fn parse(tokens: Vec<Token>) -> Result<ast::Program> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<ast::Program> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if self.match_token(&[TokenKind::Newline]) {
                continue; // Skip newlines
            }
            
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.synchronize();
                    return Err(e);
                }
            }
        }
        
        Ok(ast::Program { statements })
    }
    
    fn declaration(&mut self) -> Result<ast::Statement> {
        if self.match_token(&[TokenKind::Func]) {
            return self.function_declaration();
        } else if self.match_token(&[TokenKind::Struct]) {
            return self.struct_declaration();
        } else if self.match_token(&[TokenKind::Const]) {
            return self.var_declaration(true); // Const declaration
        }
        
        self.statement()
    }
    
    fn function_declaration(&mut self) -> Result<ast::Statement> {
        let name_token = self.consume(|kind| {
            if let TokenKind::Identifier(_) = kind {
                true
            } else {
                false
            }
        }, "Expected function name")?;
        
        let name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => unreachable!(),
        };
        
        self.consume_specific(TokenKind::LeftParen, "Expected '(' after function name")?;
        
        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                // Parse parameter
                let param_name_token = self.consume(|kind| {
                    if let TokenKind::Identifier(_) = kind {
                        true
                    } else {
                        false
                    }
                }, "Expected parameter name")?;
                
                let param_name = match &param_name_token.kind {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };
                
                // Check for type annotation (optional)
                let mut type_ann = None;
                if self.match_token(&[TokenKind::Colon]) {
                    type_ann = Some(self.parse_type()?);
                }
                
                parameters.push(ast::Parameter {
                    name: param_name,
                    type_annotation: type_ann,
                    node: ast::Node::new(param_name_token.line, param_name_token.column),
                });
                
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume_specific(TokenKind::RightParen, "Expected ')' after parameters")?;
        
        // Return type (optional)
        let return_type = None; // We'll add support for return types later
        
        // Function body
        self.consume_specific(TokenKind::Colon, "Expected ':' after function declaration")?;
        self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
        
        let body = self.block()?;
        
        Ok(ast::Statement::Declaration(ast::Declaration::Function(ast::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body: ast::BlockStatement { 
                statements: body,
                node: ast::Node::new(name_token.line, name_token.column),
            },
            node: ast::Node::new(name_token.line, name_token.column),
        })))
    }
    
    fn struct_declaration(&mut self) -> Result<ast::Statement> {
        let name_token = self.consume(|kind| {
            if let TokenKind::Identifier(_) = kind {
                true
            } else {
                false
            }
        }, "Expected struct name")?;
        
        let name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => unreachable!(),
        };
        
        self.consume_specific(TokenKind::Colon, "Expected ':' after struct name")?;
        self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
        
        let mut fields = Vec::new();
        
        // Parse fields
        while !self.is_at_end() && !self.check(&TokenKind::Func) && !self.check(&TokenKind::Struct) {
            if self.match_token(&[TokenKind::Newline]) {
                continue; // Skip empty lines
            }
            
            // Parse field name
            let field_token = self.consume(|kind| {
                if let TokenKind::Identifier(_) = kind {
                    true
                } else {
                    false
                }
            }, "Expected field name")?;
            
            let field_name = match &field_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => unreachable!(),
            };
            
            // Field must have type annotation
            self.consume_specific(TokenKind::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            
            // Optional default value
            let mut default_value = None;
            if self.match_token(&[TokenKind::Assign]) {
                default_value = Some(self.expression()?);
            }
            
            fields.push(ast::StructField {
                name: field_name,
                type_annotation: field_type,
                default_value,
                node: ast::Node::new(field_token.line, field_token.column),
            });
            
            self.consume_specific(TokenKind::Newline, "Expected newline after field declaration")?;
        }
        
        Ok(ast::Statement::Declaration(ast::Declaration::Struct(ast::StructDeclaration {
            name,
            fields,
            node: ast::Node::new(name_token.line, name_token.column),
        })))
    }
    
    fn var_declaration(&mut self, is_const: bool) -> Result<ast::Statement> {
        let name_token = self.consume(|kind| {
            if let TokenKind::Identifier(_) = kind {
                true
            } else {
                false
            }
        }, "Expected variable name")?;
        
        let name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => unreachable!(),
        };
        
        // Check for type annotation (optional)
        let mut type_ann = None;
        if self.match_token(&[TokenKind::Colon]) {
            type_ann = Some(self.parse_type()?);
        }
        
        // Variable declarations may or may not have an initializer
        let mut initializer = None;
        if self.match_token(&[TokenKind::Assign]) {
            initializer = Some(self.expression()?);
        }
        
        // Const declarations must have an initializer
        if is_const && initializer.is_none() {
            return Err(self.error("Const declarations must have an initializer"));
        }
        
        self.consume_specific(TokenKind::Newline, "Expected newline after variable declaration")?;
        
        Ok(ast::Statement::Declaration(ast::Declaration::Variable(ast::VariableDeclaration {
            name,
            type_annotation: type_ann,
            value: initializer,
            is_const,
            node: ast::Node::new(name_token.line, name_token.column),
        })))
    }
    
    fn statement(&mut self) -> Result<ast::Statement> {
        if self.match_token(&[TokenKind::If]) {
            return self.if_statement();
        } else if self.match_token(&[TokenKind::For]) {
            return self.for_statement();
        } else if self.match_token(&[TokenKind::Switch]) {
            return self.switch_statement();
        } else if self.match_token(&[TokenKind::Ret]) {
            return self.return_statement();
        } else if self.match_token(&[TokenKind::Break]) {
            let token = self.previous().unwrap().clone();
            self.consume_specific(TokenKind::Newline, "Expected newline after 'break'")?;
            return Ok(ast::Statement::Break(ast::Node::new(token.line, token.column)));
        } else if self.match_token(&[TokenKind::Continue]) {
            let token = self.previous().unwrap().clone();
            self.consume_specific(TokenKind::Newline, "Expected newline after 'continue'")?;
            return Ok(ast::Statement::Continue(ast::Node::new(token.line, token.column)));
        }
        
        // If we haven't found a statement yet, it must be an expression statement or assignment
        let expr = self.expression()?;
        
        // Check if it's an assignment
        if self.match_token(&[TokenKind::Assign]) {
            let value = self.expression()?;
            
            // Check if the target expression is a valid lvalue (identifier, dot or index expression)
            let target = match expr {
                ast::Expression::Identifier(_) => expr,
                ast::Expression::Dot(_) => expr,
                ast::Expression::Index(_) => expr,
                _ => return Err(self.error("Invalid assignment target")),
            };
            
            self.consume_specific(TokenKind::Newline, "Expected newline after expression")?;
            return Ok(ast::Statement::Assignment(ast::AssignmentStatement {
                target,
                value,
                node: ast::Node::new(self.previous().unwrap().line, self.previous().unwrap().column),
            }));
        }
        
        // It's an expression statement
        self.consume_specific(TokenKind::Newline, "Expected newline after expression")?;
        Ok(ast::Statement::Expression(ast::ExpressionStatement {
            expression: expr,
            node: ast::Node::new(self.previous().unwrap().line, self.previous().unwrap().column),
        }))
    }
    
    fn if_statement(&mut self) -> Result<ast::Statement> {
        let if_token = self.previous().unwrap().clone();
        let condition = self.expression()?;
        
        self.consume_specific(TokenKind::Colon, "Expected ':' after if condition")?;
        self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
        
        let then_branch = self.block()?;
        
        let mut else_branch = None;
        if self.match_token(&[TokenKind::Elif]) {
            // If we have an elif, treat it as a nested if in the else branch
            else_branch = Some(Box::new(self.if_statement()?));
        } else if self.match_token(&[TokenKind::Else]) {
            self.consume_specific(TokenKind::Colon, "Expected ':' after 'else'")?;
            self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
            
            let else_statements = self.block()?;
            else_branch = Some(Box::new(ast::Statement::Block(ast::BlockStatement {
                statements: else_statements,
                node: ast::Node::new(if_token.line, if_token.column),
            })));
        }
        
        Ok(ast::Statement::If(ast::IfStatement {
            condition,
            consequence: ast::BlockStatement {
                statements: then_branch,
                node: ast::Node::new(if_token.line, if_token.column),
            },
            alternative: else_branch,
            node: ast::Node::new(if_token.line, if_token.column),
        }))
    }
    
    fn for_statement(&mut self) -> Result<ast::Statement> {
        let for_token = self.previous().unwrap().clone();
        
        // Check the type of for loop by looking ahead for an identifier followed by "in"
        let save_pos = self.current;
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(_) = token.kind {
                self.advance();
                if self.match_token(&[TokenKind::In]) {
                    // For-in loop (iteration over collection)
                    let identifier = match &self.tokens[save_pos].kind {
                        TokenKind::Identifier(name) => name.clone(),
                        _ => unreachable!(),
                    };
                    
                    let collection = self.expression()?;
                    
                    self.consume_specific(TokenKind::Colon, "Expected ':' after for-in loop header")?;
                    self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
                    
                    let body = self.block()?;
                    
                    return Ok(ast::Statement::For(ast::ForStatement::Iteration {
                        node: ast::Node::new(for_token.line, for_token.column),
                        variable: identifier,
                        collection,
                        body: ast::BlockStatement {
                            statements: body,
                            node: ast::Node::new(for_token.line, for_token.column),
                        },
                    }));
                } else {
                    // Put the identifier back for regular condition loop
                    self.current -= 1;
                }
            }
        }
        
        // Check for range-based loop
        let save_pos = self.current;
        let expr = self.expression()?;
        
        if self.match_token(&[TokenKind::DotDot]) {
            // Range-based for loop
            let start = expr;
            let end = self.expression()?;
            
            self.consume_specific(TokenKind::Colon, "Expected ':' after for-range loop header")?;
            self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
            
            let body = self.block()?;
            
            // Generate a temporary variable name for range loop
            let var_name = format!("_i_{}", for_token.line);
            
            return Ok(ast::Statement::For(ast::ForStatement::Range {
                node: ast::Node::new(for_token.line, for_token.column),
                variable: var_name,
                start,
                end,
                body: ast::BlockStatement {
                    statements: body,
                    node: ast::Node::new(for_token.line, for_token.column),
                },
            }));
        } else {
            // Condition-based for loop
            self.current = save_pos;
            let condition = self.expression()?;
            
            self.consume_specific(TokenKind::Colon, "Expected ':' after for condition")?;
            self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
            
            let body = self.block()?;
            
            return Ok(ast::Statement::For(ast::ForStatement::Condition {
                node: ast::Node::new(for_token.line, for_token.column),
                condition,
                body: ast::BlockStatement {
                    statements: body,
                    node: ast::Node::new(for_token.line, for_token.column),
                },
            }));
        }
    }
    
    fn switch_statement(&mut self) -> Result<ast::Statement> {
        let switch_token = self.previous().unwrap().clone();
        let value = self.expression()?;
        
        self.consume_specific(TokenKind::Colon, "Expected ':' after switch expression")?;
        self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
        
        let mut cases = Vec::new();
        let mut default = None;
        
        while self.match_token(&[TokenKind::Case, TokenKind::Default]) {
            let token = self.previous().unwrap().clone();
            
            if let TokenKind::Case = token.kind {
                let mut case_values = Vec::new();
                
                // Parse case values
                loop {
                    case_values.push(self.expression()?);
                    
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
                
                self.consume_specific(TokenKind::Colon, "Expected ':' after case values")?;
                self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
                
                let case_body = self.block()?;
                
                cases.push(ast::CaseClause {
                    values: case_values,
                    body: ast::BlockStatement {
                        statements: case_body,
                        node: ast::Node::new(token.line, token.column),
                    },
                    node: ast::Node::new(token.line, token.column),
                });
            } else { // Default case
                self.consume_specific(TokenKind::Colon, "Expected ':' after 'default'")?;
                self.consume_specific(TokenKind::Newline, "Expected newline after ':'")?;
                
                let default_body = self.block()?;
                
                default = Some(ast::BlockStatement {
                    statements: default_body,
                    node: ast::Node::new(token.line, token.column),
                });
            }
        }
        
        Ok(ast::Statement::Switch(ast::SwitchStatement {
            value,
            cases,
            default,
            node: ast::Node::new(switch_token.line, switch_token.column),
        }))
    }
    
    fn return_statement(&mut self) -> Result<ast::Statement> {
        let ret_token = self.previous().unwrap().clone();
        
        // Check if there's any value to return
        let mut values = Vec::new();
        if !self.check(&TokenKind::Newline) {
            // Parse return values
            loop {
                values.push(self.expression()?);
                
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume_specific(TokenKind::Newline, "Expected newline after return statement")?;
        
        Ok(ast::Statement::Return(ast::ReturnStatement {
            values,
            node: ast::Node::new(ret_token.line, ret_token.column),
        }))
    }
    
    fn block(&mut self) -> Result<Vec<ast::Statement>> {
        let mut statements = Vec::new();
        
        // Keep parsing statements until we reach a potential block terminator
        while !self.is_at_end() && 
              !self.check(&TokenKind::Elif) && 
              !self.check(&TokenKind::Else) &&
              !self.check(&TokenKind::Case) && 
              !self.check(&TokenKind::Default) {
            
            if self.match_token(&[TokenKind::Newline]) {
                continue; // Skip empty lines
            }
            
            // Check for end of indented block
            if self.check(&TokenKind::Func) || self.check(&TokenKind::Struct) {
                break;
            }
            
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.synchronize();
                    return Err(e);
                }
            }
        }
        
        Ok(statements)
    }

    fn parse_type(&mut self) -> Result<ast::TypeAnnotation> {
        // For now, we'll simplify the type parsing to avoid errors with missing token types
        // Just look for custom type identifiers
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(name) = &token.kind {
                let name = name.clone();
                self.advance();
                return Ok(ast::TypeAnnotation::Struct(name));
            }
        }
        
        // Default to a string type for simplicity during development
        Ok(ast::TypeAnnotation::String)
    }
    
    fn expression(&mut self) -> Result<ast::Expression> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<ast::Expression> {
        let expr = self.logical_or()?;
        
        // Assignment is handled in the statement parser
        // Because we allow assignment as a statement but not as an expression
        
        Ok(expr)
    }
    
    fn logical_or(&mut self) -> Result<ast::Expression> {
        let mut expr = self.logical_and()?;
        
        while self.match_token(&[TokenKind::Or]) {
            let operator = ast::InfixOperator::Or;
            let right = self.logical_and()?;
            let token = self.previous().unwrap();
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn logical_and(&mut self) -> Result<ast::Expression> {
        let mut expr = self.equality()?;
        
        while self.match_token(&[TokenKind::And]) {
            let operator = ast::InfixOperator::And;
            let right = self.equality()?;
            let token = self.previous().unwrap();
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<ast::Expression> {
        let mut expr = self.comparison()?;
        
        while self.match_token(&[TokenKind::Equal, TokenKind::NotEqual]) {
            let token = self.previous().unwrap().clone();
            let operator = match token.kind {
                TokenKind::Equal => ast::InfixOperator::Equal,
                TokenKind::NotEqual => ast::InfixOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<ast::Expression> {
        let mut expr = self.term()?;
        
        while self.match_token(&[TokenKind::Less, TokenKind::LessEqual, TokenKind::Greater, TokenKind::GreaterEqual]) {
            let token = self.previous().unwrap().clone();
            let operator = match token.kind {
                TokenKind::Less => ast::InfixOperator::LessThan,
                TokenKind::LessEqual => ast::InfixOperator::LessEqual,
                TokenKind::Greater => ast::InfixOperator::GreaterThan,
                TokenKind::GreaterEqual => ast::InfixOperator::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<ast::Expression> {
        let mut expr = self.factor()?;
        
        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let token = self.previous().unwrap().clone();
            let operator = match token.kind {
                TokenKind::Plus => ast::InfixOperator::Plus,
                TokenKind::Minus => ast::InfixOperator::Minus,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<ast::Expression> {
        let mut expr = self.unary()?;
        
        while self.match_token(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let token = self.previous().unwrap().clone();
            let operator = match token.kind {
                TokenKind::Star => ast::InfixOperator::Multiply,
                TokenKind::Slash => ast::InfixOperator::Divide,
                TokenKind::Percent => ast::InfixOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = ast::Expression::Infix(Box::new(ast::InfixExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<ast::Expression> {
        if self.match_token(&[TokenKind::Minus, TokenKind::Not]) {
            let token = self.previous().unwrap().clone();
            let operator = match token.kind {
                TokenKind::Minus => ast::PrefixOperator::Minus,
                TokenKind::Not => ast::PrefixOperator::Not,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            return Ok(ast::Expression::Prefix(Box::new(ast::PrefixExpression {
                operator,
                right: Box::new(right),
                node: ast::Node::new(token.line, token.column),
            })));
        }
        
        self.call()
    }
    
    fn call(&mut self) -> Result<ast::Expression> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&[TokenKind::LeftParen]) {
                // Function call
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                // Property access
                let token = self.previous().unwrap().clone();
                let name = self.consume(|kind| {
                    if let TokenKind::Identifier(_) = kind {
                        true
                    } else {
                        false
                    }
                }, "Expected property name after '.'")?;
                
                let identifier = match &name.kind {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };
                
                expr = ast::Expression::Dot(Box::new(ast::DotExpression {
                    left: Box::new(expr),
                    identifier,
                    node: ast::Node::new(token.line, token.column),
                }));
            } else if self.match_token(&[TokenKind::LeftBracket]) {
                // Array/map indexing
                let token = self.previous().unwrap().clone();
                let index = self.expression()?;
                self.consume_specific(TokenKind::RightBracket, "Expected ']' after index")?;
                
                expr = ast::Expression::Index(Box::new(ast::IndexExpression {
                    left: Box::new(expr),
                    index: Box::new(index),
                    node: ast::Node::new(token.line, token.column),
                }));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: ast::Expression) -> Result<ast::Expression> {
        let token = self.previous().unwrap().clone();
        let mut arguments = Vec::new();
        
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume_specific(TokenKind::RightParen, "Expected ')' after arguments")?;
        
        Ok(ast::Expression::Call(Box::new(ast::CallExpression {
            function: Box::new(callee),
            arguments,
            node: ast::Node::new(token.line, token.column),
        })))
    }
    
    fn primary(&mut self) -> Result<ast::Expression> {
        let token = self.peek().ok_or_else(|| self.error("Expected expression"))?.clone();
        
        if self.match_token(&[TokenKind::Nil]) {
            return Ok(ast::Expression::Literal(ast::LiteralExpression::Nil {
                node: ast::Node::new(token.line, token.column),
            }));
        }
        
        if self.match_token(&[TokenKind::True, TokenKind::False]) {
            let value = match self.previous().unwrap().kind {
                TokenKind::True => true,
                TokenKind::False => false,
                _ => unreachable!(),
            };
            return Ok(ast::Expression::Literal(ast::LiteralExpression::Bool {
                node: ast::Node::new(token.line, token.column),
                value,
            }));
        }
        
        if let Some(current_token) = self.peek() {
            let current_token = current_token.clone();
            match &current_token.kind {
                TokenKind::IntLiteral(value) => {
                    let value = *value;
                    self.advance();
                    return Ok(ast::Expression::Literal(ast::LiteralExpression::Int {
                        node: ast::Node::new(current_token.line, current_token.column),
                        value,
                    }));
                }
                TokenKind::FloatLiteral(value) => {
                    let value = *value;
                    self.advance();
                    return Ok(ast::Expression::Literal(ast::LiteralExpression::Float {
                        node: ast::Node::new(current_token.line, current_token.column),
                        value,
                    }));
                }
                TokenKind::StringLiteral(value) => {
                    let value = value.clone();
                    self.advance();
                    return Ok(ast::Expression::Literal(ast::LiteralExpression::String {
                        node: ast::Node::new(current_token.line, current_token.column),
                        value,
                    }));
                }
                TokenKind::CharLiteral(value) => {
                    let value = *value;
                    self.advance();
                    return Ok(ast::Expression::Literal(ast::LiteralExpression::Char {
                        node: ast::Node::new(current_token.line, current_token.column),
                        value,
                    }));
                }
                TokenKind::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    return Ok(ast::Expression::Identifier(ast::IdentifierExpression {
                        node: ast::Node::new(current_token.line, current_token.column),
                        value: name,
                    }));
                }
                TokenKind::LeftParen => {
                    self.advance();
                    let expr = self.expression()?;
                    self.consume_specific(TokenKind::RightParen, "Expected ')' after expression")?;
                    return Ok(expr);
                }
                TokenKind::LeftBracket => {
                    // Array literal
                    self.advance();
                    let mut elements = Vec::new();
                    
                    if !self.check(&TokenKind::RightBracket) {
                        loop {
                            elements.push(self.expression()?);
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }
                    
                    self.consume_specific(TokenKind::RightBracket, "Expected ']' after array elements")?;
                    
                    return Ok(ast::Expression::Array(ast::ArrayExpression {
                        node: ast::Node::new(current_token.line, current_token.column),
                        elements,
                    }));
                }
                TokenKind::LeftBrace => {
                    // HashMap literal
                    self.advance();
                    let mut pairs = Vec::new();
                    
                    if !self.check(&TokenKind::RightBrace) {
                        loop {
                            let key = self.expression()?;
                            self.consume_specific(TokenKind::Colon, "Expected ':' after map key")?;
                            let value = self.expression()?;
                            
                            pairs.push((key, value));
                            
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }
                    
                    self.consume_specific(TokenKind::RightBrace, "Expected '}' after map entries")?;
                    
                    return Ok(ast::Expression::HashMap(ast::HashMapExpression {
                        node: ast::Node::new(current_token.line, current_token.column),
                        pairs,
                    }));
                }
                _ => {}
            }
        }
        
        Err(self.error("Expected expression"))
    }
    
    // Helper methods for token matching and consumption
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }
    
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        let current = &self.peek().unwrap().kind;
        
        match (current, kind) {
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            _ => std::mem::discriminant(current) == std::mem::discriminant(kind),
        }
    }
    
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            // Special handling for Identifier
            if let TokenKind::Identifier(_) = kind {
                if let Some(token) = self.peek() {
                    if let TokenKind::Identifier(_) = token.kind {
                        self.advance();
                        return true;
                    }
                }
                continue;
            }
            
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        
        false
    }
    
    fn consume<F>(&mut self, predicate: F, error_message: &str) -> Result<Token>
    where
        F: FnOnce(&TokenKind) -> bool,
    {
        if let Some(token) = self.peek() {
            if predicate(&token.kind) {
                // Clone the token before advancing
                let result = token.clone();
                self.advance();
                return Ok(result);
            }
        }
        
        Err(self.error(error_message))
    }
    
    fn consume_specific(&mut self, kind: TokenKind, error_message: &str) -> Result<Token> {
        let matches = |token_kind: &TokenKind| -> bool {
            match (token_kind, &kind) {
                (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
                _ => std::mem::discriminant(token_kind) == std::mem::discriminant(&kind),
            }
        };
        self.consume(matches, error_message)
    }
    
    fn error(&self, message: &str) -> WidowError {
        if let Some(token) = self.peek() {
            WidowError::Parser {
                line: token.line,
                column: token.column,
                message: message.to_string(),
            }
        } else if let Some(token) = self.previous() {
            WidowError::Parser {
                line: token.line,
                column: token.column,
                message: message.to_string(),
            }
        } else {
            WidowError::Generic(message.to_string())
        }
    }
    
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if let Some(token) = self.previous() {
                if matches!(token.kind, TokenKind::Newline) {
                    return;
                }
            }
            
            if let Some(token) = self.peek() {
                if matches!(token.kind, 
                    TokenKind::Func | 
                    TokenKind::Struct |
                    TokenKind::If | 
                    TokenKind::For |
                    TokenKind::Switch |
                    TokenKind::Ret
                ) {
                    return;
                }
            }
            
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    
    #[test]
    fn test_parse_empty() {
        let tokens = tokenize("").unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 0);
    }
    
    #[test]
    fn test_parse_simple_expression() {
        let tokens = tokenize("5 + 3\n").unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}