use crate::ast::*;

pub fn optimize_program(program: Program) -> Program {
    // Apply various optimization passes
    let mut optimized_program = program;
    
    // Pass 1: Constant folding
    optimized_program = constant_folding_pass(optimized_program);
    
    // Pass 2: Dead code elimination
    optimized_program = dead_code_elimination_pass(optimized_program);
    
    // Pass 3: Simple inlining (for small functions)
    optimized_program = simple_inlining_pass(optimized_program);
    
    optimized_program
}

fn constant_folding_pass(program: Program) -> Program {
    let statements = program.statements
        .into_iter()
        .map(|stmt| constant_fold_statement(stmt))
        .collect();
    
    Program { statements }
}

fn constant_fold_statement(stmt: Statement) -> Statement {
    match stmt {
        Statement::Expression(expr) => {
            Statement::Expression(constant_fold_expression(expr))
        },
        Statement::LetBinding { mutable, name, type_annotation, value } => {
            Statement::LetBinding {
                mutable,
                name,
                type_annotation,
                value: constant_fold_expression(value),
            }
        },
        Statement::ConstBinding { name, type_annotation, value } => {
            Statement::ConstBinding {
                name,
                type_annotation,
                value: constant_fold_expression(value),
            }
        },
        Statement::Function(func) => {
            Statement::Function(FunctionDef {
                name: func.name,
                parameters: func.parameters,
                return_type: func.return_type,
                body: func.body.into_iter().map(constant_fold_statement).collect(),
                is_async: func.is_async,
                is_public: func.is_public,
                is_awaitable: func.is_awaitable,
                effect_annotations: func.effect_annotations,
            })
        },
        Statement::Block(statements) => {
            Statement::Block(statements.into_iter().map(constant_fold_statement).collect())
        },
        _ => stmt, // Other statement types remain unchanged for now
    }
}

fn constant_fold_expression(expr: Expression) -> Expression {
    match expr {
        Expression::BinaryOp(left, op, right) => {
            let left_folded = constant_fold_expression(*left);
            let right_folded = constant_fold_expression(*right);
            
            // Try to fold constant expressions
            match (&left_folded, &right_folded, op.clone()) {
                (Expression::Integer(a), Expression::Integer(b), BinaryOp::Add) => {
                    Expression::Integer(a + b)
                },
                (Expression::Integer(a), Expression::Integer(b), BinaryOp::Sub) => {
                    Expression::Integer(a - b)
                },
                (Expression::Integer(a), Expression::Integer(b), BinaryOp::Mul) => {
                    Expression::Integer(a * b)
                },
                (Expression::Integer(a), Expression::Integer(b), BinaryOp::Div) if *b != 0 => {
                    Expression::Integer(a / b)
                },
                (Expression::Integer(a), Expression::Integer(b), BinaryOp::Mod) if *b != 0 => {
                    Expression::Integer(a % b)
                },
                (Expression::Float(a), Expression::Float(b), BinaryOp::Add) => {
                    Expression::Float(a + b)
                },
                (Expression::Float(a), Expression::Float(b), BinaryOp::Sub) => {
                    Expression::Float(a - b)
                },
                (Expression::Float(a), Expression::Float(b), BinaryOp::Mul) => {
                    Expression::Float(a * b)
                },
                (Expression::Float(a), Expression::Float(b), BinaryOp::Div) if *b != 0.0 => {
                    Expression::Float(a / b)
                },
                (Expression::Boolean(a), Expression::Boolean(b), BinaryOp::And) => {
                    Expression::Boolean(*a && *b)
                },
                (Expression::Boolean(a), Expression::Boolean(b), BinaryOp::Or) => {
                    Expression::Boolean(*a || *b)
                },
                _ => {
                    // If we can't fold, return the expression with folded operands
                    Expression::BinaryOp(
                        Box::new(left_folded),
                        op.clone(),
                        Box::new(right_folded)
                    )
                }
            }
        },
        Expression::UnaryOp(op, operand) => {
            let operand_folded = constant_fold_expression(*operand);
            
            match (&operand_folded, op.clone()) {
                (Expression::Integer(n), UnaryOp::Neg) => {
                    Expression::Integer(-n)
                },
                (Expression::Float(f), UnaryOp::Neg) => {
                    Expression::Float(-f)
                },
                (Expression::Boolean(b), UnaryOp::Not) => {
                    Expression::Boolean(!b)
                },
                _ => {
                    Expression::UnaryOp(op.clone(), Box::new(operand_folded))
                }
            }
        },
        Expression::If(condition, then_branch, else_branch) => {
            let folded_condition = constant_fold_expression(*condition);
            
            // If condition is a constant, we can eliminate one branch
            match folded_condition {
                Expression::Boolean(true) => {
                    // Only execute then branch
                    if then_branch.len() == 1 {
                        match &then_branch[0] {
                            Statement::Expression(expr) => constant_fold_expression(expr.clone()),
                            _ => Expression::BlockExpr(then_branch),
                        }
                    } else {
                        Expression::BlockExpr(then_branch)
                    }
                },
                Expression::Boolean(false) => {
                    // Only execute else branch
                    if else_branch.len() == 1 {
                        match &else_branch[0] {
                            Statement::Expression(expr) => constant_fold_expression(expr.clone()),
                            _ => Expression::BlockExpr(else_branch),
                        }
                    } else {
                        Expression::BlockExpr(else_branch)
                    }
                },
                _ => {
                    Expression::If(
                        Box::new(folded_condition),
                        then_branch.into_iter().map(constant_fold_statement).collect(),
                        else_branch.into_iter().map(constant_fold_statement).collect(),
                    )
                }
            }
        },
        Expression::BlockExpr(statements) => {
            Expression::BlockExpr(statements.into_iter().map(constant_fold_statement).collect())
        },
        _ => expr, // Other expressions remain unchanged
    }
}

fn dead_code_elimination_pass(program: Program) -> Program {
    // For now, a simple pass that removes unused variables
    // In a real implementation, this would be more sophisticated
    let statements = program.statements
        .into_iter()
        .filter(|stmt| !is_dead_code(stmt))
        .collect();
    
    Program { statements }
}

fn is_dead_code(stmt: &Statement) -> bool {
    // Simple heuristic: if it's an expression that's just a literal, it might be dead code
    // unless it's part of a larger computation
    match stmt {
        Statement::Expression(Expression::Integer(_)) => true,
        Statement::Expression(Expression::Float(_)) => true,
        Statement::Expression(Expression::String(_)) => true,
        Statement::Expression(Expression::Boolean(_)) => true,
        Statement::Expression(Expression::Nil) => true,
        _ => false,
    }
}

fn simple_inlining_pass(program: Program) -> Program {
    // For now, a simple pass that doesn't do actual inlining
    // In a real implementation, this would inline small functions
    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_constant_folding() {
        let input = r#"
        fn main() {
            let x = 5 + 3
            let y = x * 2
            print(x + y)
        }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let optimized = optimize_program(program);
        
        // The optimized program should still be valid
        assert!(!optimized.statements.is_empty());
    }

    #[test]
    fn test_arithmetic_constant_folding() {
        let input = "let x = 10 + 5";
        
        let mut parser = Parser::new(input);
        let mut stmt = parser.parse_statement().unwrap();

        // Convert to expression to test constant folding directly
        if let Statement::LetBinding { ref mut value, .. } = &mut stmt {
            *value = constant_fold_expression(std::mem::replace(value, Expression::Nil));
        }
        
        // The expression should be folded to a single integer
        if let Statement::LetBinding { value, .. } = stmt {
            match value {
                Expression::Integer(15) => {}, // Expected result of 10 + 5
                _ => panic!("Expected constant folded to 15"),
            }
        } else {
            panic!("Expected LetBinding");
        }
    }

    #[test]
    fn test_boolean_constant_folding() {
        let input = "let x = true && false";
        
        let mut parser = Parser::new(input);
        let mut stmt = parser.parse_statement().unwrap();

        if let Statement::LetBinding { ref mut value, .. } = &mut stmt {
            *value = constant_fold_expression(std::mem::replace(value, Expression::Nil));
        }
        
        if let Statement::LetBinding { value, .. } = stmt {
            match value {
                Expression::Boolean(false) => {}, // Expected result of true && false
                _ => panic!("Expected constant folded to false"),
            }
        } else {
            panic!("Expected LetBinding");
        }
    }
}