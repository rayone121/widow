use pest::Parser;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

#[derive(pest_derive::Parser)]
#[grammar = "widow.pest"] // relative to src/
pub struct WidowParser;

#[derive(Debug)]
pub enum Expr {
    Literal(String),
    Variable(String),
    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ArrayAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    Grouped(Box<Expr>),
}

lazy_static::lazy_static! {
    static ref PRATT: PrattParser<Rule> = {
        PrattParser::new()
            // Logical OR (lowest precedence)
            .op(Op::infix(Rule::logical_or, Assoc::Left))
            // Logical AND
            .op(Op::infix(Rule::logical_and, Assoc::Left))
            // Equality
            .op(Op::infix(Rule::equality, Assoc::Left))
            // Comparison
            .op(Op::infix(Rule::comparison, Assoc::Left))
            // Range
            .op(Op::infix(Rule::range, Assoc::Left))
            // Addition/Subtraction
            .op(Op::infix(Rule::addition, Assoc::Left))
            // Multiplication/Division/Modulo
            .op(Op::infix(Rule::multiplication, Assoc::Left))
            // Unary operators (highest precedence)
            .op(Op::prefix(Rule::unary))
    };
}

pub fn parse_source(source: &str) -> Result<(), pest::error::Error<Rule>> {
    let mut parsed = WidowParser::parse(Rule::program, source)?;
    let program = parsed.next().unwrap();

    for stmt in program.into_inner() {
        if stmt.as_rule() == Rule::EOI {
            continue;
        }

        println!(
            "DEBUG: Matched pair: {:?} => {:?}",
            stmt.as_rule(),
            stmt.as_str()
        );
        println!("DEBUG: Statement inner pairs:");
        for inner in stmt.clone().into_inner() {
            println!("  {:?} => {:?}", inner.as_rule(), inner.as_str());
        }

        match stmt.as_rule() {
            Rule::variable_decl => {
                println!("Variable declaration: {:?}", stmt.as_str());
                parse_variable_decl(stmt);
            }
            Rule::const_decl => {
                println!("Const declaration: {:?}", stmt.as_str());
                parse_const_decl(stmt);
            }
            Rule::func_decl => {
                println!("Function declaration: {:?}", stmt.as_str());
                parse_func_decl(stmt);
            }
            Rule::struct_decl => {
                println!("Struct declaration: {:?}", stmt.as_str());
                parse_struct_decl(stmt);
            }
            Rule::impl_decl => {
                println!("Implementation declaration: {:?}", stmt.as_str());
                parse_impl_decl(stmt);
            }
            Rule::return_stmt => {
                println!("Return statement: {:?}", stmt.as_str());
                parse_return_stmt(stmt);
            }
            Rule::assignment_stmt => {
                println!("Assignment statement: {:?}", stmt.as_str());
                parse_assignment_stmt(stmt);
            }
            Rule::control_flow => {
                println!("Control flow: {:?}", stmt.as_str());
                parse_control_flow(stmt);
            }
            Rule::expr_stmt => {
                let expression_pair = stmt.into_inner().next().unwrap();
                println!(
                    "DEBUG: expr_stmt contains: {:?} => {:?}",
                    expression_pair.as_rule(),
                    expression_pair.as_str()
                );
                let expr = parse_expression(expression_pair);
                println!("Expression statement: {:?}", expr);
            }
            _ => {
                println!("DEBUG: Unhandled rule: {:?}", stmt.as_rule());
            }
        }
    }
    Ok(())
}

fn parse_expression(pair: Pair<Rule>) -> Expr {
    println!(
        "DEBUG: parse_expression called with: {:?} => {:?}",
        pair.as_rule(),
        pair.as_str()
    );

    match pair.as_rule() {
        Rule::expression => {
            // Expression rule contains the precedence chain
            let inner = pair.into_inner().next().unwrap();
            parse_expression(inner)
        }
        Rule::logical_or => parse_binary_expr(pair),
        Rule::logical_and => parse_binary_expr(pair),
        Rule::equality => parse_binary_expr(pair),
        Rule::comparison => parse_binary_expr(pair),
        Rule::range => parse_binary_expr(pair),
        Rule::addition => parse_binary_expr(pair),
        Rule::multiplication => parse_binary_expr(pair),
        Rule::unary => parse_unary_expr(pair),
        Rule::postfix => parse_postfix_expr(pair),
        Rule::primary => parse_primary(pair),
        _ => {
            // If it's a direct atom, parse it
            parse_primary(pair)
        }
    }
}

fn parse_binary_expr(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut left = parse_expression(inner.next().unwrap());

    while let Some(op_pair) = inner.next() {
        let right = parse_expression(inner.next().unwrap());
        left = Expr::BinaryOp {
            left: Box::new(left),
            op: get_binary_op_string(&op_pair),
            right: Box::new(right),
        };
    }

    left
}

fn parse_unary_expr(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut ops = Vec::new();

    // Collect all unary operators
    while let Some(next) = inner.peek() {
        if matches!(next.as_rule(), Rule::unary) {
            ops.push(inner.next().unwrap().as_str().to_string());
        } else {
            break;
        }
    }

    // Parse the base expression
    let mut expr = parse_expression(inner.next().unwrap());

    // Apply unary operators (right to left)
    for op in ops.into_iter().rev() {
        expr = Expr::UnaryOp {
            op,
            expr: Box::new(expr),
        };
    }

    expr
}

fn parse_postfix_expr(pair: Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut expr = parse_expression(inner.next().unwrap());

    for postfix_op in inner {
        match postfix_op.as_rule() {
            Rule::function_call_op => {
                let args = if let Some(args_inner) = postfix_op.into_inner().next() {
                    args_inner.into_inner().map(parse_expression).collect()
                } else {
                    Vec::new()
                };

                // Extract function name from current expression
                let name = match expr {
                    Expr::Variable(n) => n,
                    _ => "unknown".to_string(), // This shouldn't happen with proper grammar
                };

                expr = Expr::FuncCall { name, args };
            }
            Rule::field_access_op => {
                let field = postfix_op.into_inner().next().unwrap().as_str().to_string();
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                };
            }
            Rule::array_access_op => {
                let index = postfix_op.into_inner().next().unwrap();
                expr = Expr::ArrayAccess {
                    object: Box::new(expr),
                    index: Box::new(parse_expression(index)),
                };
            }
            _ => unreachable!("Unexpected postfix op: {:?}", postfix_op.as_rule()),
        }
    }

    expr
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::literal => Expr::Literal(pair.as_str().to_string()),
        Rule::identifier => Expr::Variable(pair.as_str().to_string()),
        Rule::grouped_expr => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Grouped(Box::new(parse_expression(inner)))
        }
        Rule::array_literal => {
            let elements: Vec<Expr> = pair.into_inner().map(parse_expression).collect();
            Expr::ArrayLiteral(elements)
        }
        Rule::map_literal => {
            let entries: Vec<(Expr, Expr)> = pair
                .into_inner()
                .map(|entry_pair| {
                    let mut entry_inner = entry_pair.into_inner();
                    let key = parse_expression(entry_inner.next().unwrap());
                    let value = parse_expression(entry_inner.next().unwrap());
                    (key, value)
                })
                .collect();
            Expr::MapLiteral(entries)
        }
        _ => unreachable!("Unexpected primary rule: {:?}", pair.as_rule()),
    }
}

fn get_binary_op_string(pair: &Pair<Rule>) -> String {
    // The binary operators are now embedded in the grammar rules
    // We need to extract the actual operator string
    match pair.as_str() {
        s if s.contains("||") => "||".to_string(),
        s if s.contains("&&") => "&&".to_string(),
        s if s.contains("==") => "==".to_string(),
        s if s.contains("!=") => "!=".to_string(),
        s if s.contains("<=") => "<=".to_string(),
        s if s.contains(">=") => ">=".to_string(),
        s if s.contains("<") => "<".to_string(),
        s if s.contains(">") => ">".to_string(),
        s if s.contains("..") => "..".to_string(),
        s if s.contains("+") => "+".to_string(),
        s if s.contains("-") => "-".to_string(),
        s if s.contains("*") => "*".to_string(),
        s if s.contains("/") => "/".to_string(),
        s if s.contains("%") => "%".to_string(),
        _ => pair.as_str().to_string(),
    }
}

// Helper functions for parsing different statement types
fn parse_variable_decl(pair: Pair<Rule>) {
    println!("Parsing variable declaration:");
    for inner in pair.into_inner() {
        println!("  {:?} => {:?}", inner.as_rule(), inner.as_str());
    }
}

fn parse_const_decl(pair: Pair<Rule>) {
    println!("Parsing const declaration:");
    for inner in pair.into_inner() {
        println!("  {:?} => {:?}", inner.as_rule(), inner.as_str());
    }
}

fn parse_func_decl(pair: Pair<Rule>) {
    println!("Parsing function declaration:");
    for inner in pair.clone().into_inner() {
        println!("  func part: {:?} => {:?}", inner.as_rule(), inner.as_str());
        if inner.as_rule() == Rule::block {
            println!("    block contents:");
            for block_stmt in inner.into_inner() {
                println!(
                    "      {:?} => {:?}",
                    block_stmt.as_rule(),
                    block_stmt.as_str()
                );
            }
        }
    }
}

fn parse_struct_decl(pair: Pair<Rule>) {
    println!("Parsing struct declaration:");
    for inner in pair.into_inner() {
        println!("  {:?} => {:?}", inner.as_rule(), inner.as_str());
    }
}

fn parse_impl_decl(pair: Pair<Rule>) {
    println!("Parsing impl declaration:");
    for inner in pair.into_inner() {
        println!("  {:?} => {:?}", inner.as_rule(), inner.as_str());
    }
}

fn parse_return_stmt(pair: Pair<Rule>) {
    println!("Parsing return statement:");
    for inner in pair.into_inner() {
        let expr = parse_expression(inner);
        println!("  return expr: {:?}", expr);
    }
}

fn parse_assignment_stmt(pair: Pair<Rule>) {
    println!("Parsing assignment statement:");
    let mut inner = pair.into_inner();
    let target = inner.next().unwrap();
    let value = inner.next().unwrap();

    println!("  target: {:?} => {:?}", target.as_rule(), target.as_str());
    let value_expr = parse_expression(value);
    println!("  value: {:?}", value_expr);
}

fn parse_control_flow(pair: Pair<Rule>) {
    println!("Parsing control flow:");
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::if_stmt => println!("  if statement: {:?}", inner.as_str()),
        Rule::for_loop => println!("  for loop: {:?}", inner.as_str()),
        Rule::while_loop => println!("  while loop: {:?}", inner.as_str()),
        Rule::switch_stmt => println!("  switch statement: {:?}", inner.as_str()),
        _ => println!("  unknown control flow: {:?}", inner.as_rule()),
    }
}
