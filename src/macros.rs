//! Macro system implementation for the Logos programming language
//! Provides compile-time code generation and metaprogramming capabilities

use std::collections::HashMap;
use crate::ast::*;

/// Represents a macro definition with its expansion rules
#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
    pub is_hygienic: bool,
}

/// The macro expansion system
pub struct MacroSystem {
    macros: HashMap<String, Macro>,
}

impl MacroSystem {
    /// Creates a new macro system instance
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Registers a new macro definition
    pub fn register_macro(&mut self, macro_def: &MacroDef) -> Result<(), String> {
        let macro_obj = Macro {
            name: macro_def.name.clone(),
            parameters: macro_def.parameters.clone(),
            body: macro_def.body.clone(),
            is_hygienic: macro_def.is_hygienic,
        };

        if self.macros.contains_key(&macro_obj.name) {
            return Err(format!("Macro '{}' already defined", macro_obj.name));
        }

        self.macros.insert(macro_obj.name.clone(), macro_obj);
        Ok(())
    }

    /// Expands a macro invocation by substituting parameters with provided arguments
    pub fn expand_macro(&self, name: &str, args: &[Expression]) -> Result<Vec<Statement>, String> {
        let macro_def = self.macros.get(name)
            .ok_or_else(|| format!("Macro '{}' not found", name))?;

        if args.len() != macro_def.parameters.len() {
            return Err(format!(
                "Macro '{}' expects {} parameters, got {}",
                name,
                macro_def.parameters.len(),
                args.len()
            ));
        }

        // Create a substitution map from parameter names to argument values
        let mut substitutions = HashMap::new();
        for (param, arg) in macro_def.parameters.iter().zip(args.iter()) {
            substitutions.insert(param.clone(), arg.clone());
        }

        // Expand the macro body by substituting parameters with arguments
        let mut expanded_body = Vec::new();
        for stmt in &macro_def.body {
            let expanded_stmt = self.substitute_parameters(stmt, &substitutions)?;
            expanded_body.push(expanded_stmt);
        }

        Ok(expanded_body)
    }

    /// Recursively substitutes parameter references with actual arguments in a statement
    fn substitute_parameters(&self, stmt: &Statement, substitutions: &HashMap<String, Expression>) -> Result<Statement, String> {
        match stmt {
            Statement::Expression(expr) => {
                let new_expr = self.substitute_expr_parameters(expr, substitutions)?;
                Ok(Statement::Expression(new_expr))
            },
            Statement::LetBinding { mutable, name, type_annotation, value, ownership_modifier, lifetime_annotation } => {
                let new_value = self.substitute_expr_parameters(value, substitutions)?;
                Ok(Statement::LetBinding {
                    mutable: *mutable,
                    name: name.clone(),
                    type_annotation: type_annotation.clone(),
                    value: new_value,
                    ownership_modifier: ownership_modifier.clone(),
                    lifetime_annotation: lifetime_annotation.clone(),
                })
            },
            Statement::Block(statements) => {
                let mut new_statements = Vec::new();
                for stmt in statements {
                    new_statements.push(self.substitute_parameters(stmt, substitutions)?);
                }
                Ok(Statement::Block(new_statements))
            },
            // For now, we'll handle other statement types by returning them unchanged
            // In a full implementation, we'd recursively substitute in all statement types
            _ => Ok(stmt.clone()),
        }
    }

    /// Recursively substitutes parameter references with actual arguments in an expression
    fn substitute_expr_parameters(&self, expr: &Expression, substitutions: &HashMap<String, Expression>) -> Result<Expression, String> {
        match expr {
            Expression::Identifier(name) => {
                // If this identifier matches a macro parameter, substitute it
                if let Some(replacement) = substitutions.get(name) {
                    Ok(replacement.clone())
                } else {
                    Ok(Expression::Identifier(name.clone()))
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let new_left = Box::new(self.substitute_expr_parameters(left, substitutions)?);
                let new_right = Box::new(self.substitute_expr_parameters(right, substitutions)?);
                Ok(Expression::BinaryOp(new_left, op.clone(), new_right))
            },
            Expression::Call(func_name, args) => {
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(self.substitute_expr_parameters(arg, substitutions)?);
                }
                Ok(Expression::Call(func_name.clone(), new_args))
            },
            // For now, we'll handle other expression types by returning them unchanged
            // In a full implementation, we'd recursively substitute in all expression types
            _ => Ok(expr.clone()),
        }
    }

    /// Checks if a macro with the given name exists
    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }
}

/// Preprocesses a program by expanding all macro invocations
pub fn preprocess_macros(program: &Program) -> Result<Program, String> {
    let mut macro_system = MacroSystem::new();
    
    // First, collect all macro definitions
    for stmt in &program.statements {
        if let Statement::MacroDefinition(macro_def) = stmt {
            macro_system.register_macro(macro_def)?;
        }
    }
    
    // Then, expand the program by replacing macro invocations with their expansions
    let mut expanded_statements = Vec::new();
    for stmt in &program.statements {
        // Skip macro definitions as they've been processed
        if matches!(stmt, Statement::MacroDefinition(_)) {
            continue;
        }
        
        // Expand any macro invocations in the statement
        let expanded_stmt = expand_macros_in_statement(stmt, &macro_system)?;
        expanded_statements.push(expanded_stmt);
    }
    
    Ok(Program {
        statements: expanded_statements,
    })
}

/// Recursively expands macro invocations in a statement
fn expand_macros_in_statement(stmt: &Statement, macro_system: &MacroSystem) -> Result<Statement, String> {
    match stmt {
        Statement::Expression(expr) => {
            let expanded_expr = expand_macros_in_expression(expr, macro_system)?;
            Ok(Statement::Expression(expanded_expr))
        },
        Statement::Block(statements) => {
            let mut expanded_block = Vec::new();
            for stmt in statements {
                expanded_block.push(expand_macros_in_statement(stmt, macro_system)?);
            }
            Ok(Statement::Block(expanded_block))
        },
        // For now, return other statement types unchanged
        _ => Ok(stmt.clone()),
    }
}

/// Recursively expands macro invocations in an expression
fn expand_macros_in_expression(expr: &Expression, macro_system: &MacroSystem) -> Result<Expression, String> {
    match expr {
        Expression::MacroInvocation(name, args) => {
            // Expand the macro and return the first expression from the expansion
            // In a more sophisticated implementation, we might need to handle multiple expressions
            let expanded_statements = macro_system.expand_macro(name, args)?;
            
            // Find the first expression in the expanded statements
            for stmt in expanded_statements {
                if let Statement::Expression(e) = stmt {
                    return Ok(e);
                }
            }
            
            // If no expression is found, return a unit value
            Ok(Expression::Nil)
        },
        Expression::BinaryOp(left, op, right) => {
            let new_left = Box::new(expand_macros_in_expression(left, macro_system)?);
            let new_right = Box::new(expand_macros_in_expression(right, macro_system)?);
            Ok(Expression::BinaryOp(new_left, op.clone(), new_right))
        },
        Expression::Call(func_name, args) => {
            let mut new_args = Vec::new();
            for arg in args {
                new_args.push(expand_macros_in_expression(arg, macro_system)?);
            }
            Ok(Expression::Call(func_name.clone(), new_args))
        },
        // For now, return other expression types unchanged
        _ => Ok(expr.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_registration() {
        let mut macro_system = MacroSystem::new();
        
        let macro_def = MacroDef {
            name: "test_macro".to_string(),
            parameters: vec!["x".to_string()],
            body: vec![Statement::Expression(Expression::Identifier("x".to_string()))],
            is_hygienic: true,
        };
        
        assert!(macro_system.register_macro(&macro_def).is_ok());
        assert!(macro_system.has_macro("test_macro"));
    }
}