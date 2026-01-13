//! Debug Information Generation for the Logos Programming Language
//! This module adds debugging information to the compiled code to support source-level debugging

use crate::ast::*;
use std::collections::HashMap;

/// Debug information metadata
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Source file name
    pub file_name: String,
    /// Line number mapping (AST line -> compiled line)
    pub line_mapping: HashMap<usize, usize>,
    /// Variable location mapping
    pub variable_locations: HashMap<String, DebugLocation>,
    /// Function debug information
    pub function_info: HashMap<String, FunctionDebugInfo>,
}

/// Debug location information
#[derive(Debug, Clone)]
pub struct DebugLocation {
    /// Line number in source
    pub line: usize,
    /// Column number in source
    pub column: usize,
    /// Scope ID
    pub scope: usize,
}

/// Function debug information
#[derive(Debug, Clone)]
pub struct FunctionDebugInfo {
    /// Function name
    pub name: String,
    /// Starting line in source
    pub start_line: usize,
    /// Ending line in source
    pub end_line: usize,
    /// Parameter debug info
    pub parameters: Vec<ParameterDebugInfo>,
    /// Local variable debug info
    pub local_vars: Vec<VariableDebugInfo>,
}

/// Parameter debug information
#[derive(Debug, Clone)]
pub struct ParameterDebugInfo {
    /// Parameter name
    pub name: String,
    /// Line number where parameter is defined
    pub line: usize,
    /// Type information
    pub type_info: String, // String representation of the type
}

/// Variable debug information
#[derive(Debug, Clone)]
pub struct VariableDebugInfo {
    /// Variable name
    pub name: String,
    /// Line number where variable is defined
    pub line: usize,
    /// Type information
    pub type_info: String, // String representation of the type
    /// Storage location (register, stack, etc.)
    pub storage_location: StorageLocation,
}

/// Storage location for variables
#[derive(Debug, Clone)]
pub enum StorageLocation {
    Register(String),
    Stack(i32), // Offset from frame pointer
    Heap(String, i32), // Address as string and offset
}

/// Debug information generator
pub struct DebugInfoGenerator {
    /// Current debug information
    debug_info: DebugInfo,
    /// Current function being processed
    current_function: Option<String>,
    /// Current line number
    current_line: usize,
}

impl DebugInfoGenerator {
    /// Creates a new debug information generator
    pub fn new(file_name: &str) -> Self {
        Self {
            debug_info: DebugInfo {
                file_name: file_name.to_string(),
                line_mapping: HashMap::new(),
                variable_locations: HashMap::new(),
                function_info: HashMap::new(),
            },
            current_function: None,
            current_line: 0,
        }
    }

    /// Generates debug information for a program
    pub fn generate_debug_info_for_program(&mut self, program: &Program) {
        for statement in &program.statements {
            self.generate_debug_info_for_statement(statement);
        }
    }

    /// Generates debug information for a statement
    fn generate_debug_info_for_statement(&mut self, statement: &Statement) {
        // Set the current line based on the statement's position
        // For now, we'll just increment the line number
        self.current_line += 1;
        
        match statement {
            Statement::LetBinding { name, value, .. } => {
                // Generate debug info for variable binding
                self.record_variable_location(name, self.current_line, 0); // Column 0 for now
                self.generate_debug_info_for_expression(value);
            },
            Statement::Function(func_def) => {
                self.generate_debug_info_for_function(func_def);
            },
            Statement::Expression(expr) => {
                self.generate_debug_info_for_expression(expr);
            },
            Statement::Return(expr) => {
                if let Some(return_expr) = expr {
                    self.generate_debug_info_for_expression(return_expr);
                }
            },
            Statement::Block(statements) => {
                for stmt in statements {
                    self.generate_debug_info_for_statement(stmt);
                }
            },
            // Handle other statement types as needed
            _ => {
                // For unhandled statements, just continue
            }
        }
    }

    /// Generates debug information for a function
    fn generate_debug_info_for_function(&mut self, func_def: &FunctionDef) {
        // Set the current function
        self.current_function = Some(func_def.name.clone());
        
        // Create function debug info
        let param_debug_info: Vec<ParameterDebugInfo> = func_def.parameters.iter()
            .map(|param| ParameterDebugInfo {
                name: param.name.clone(),
                line: self.current_line, // This would be the actual line from AST
                type_info: format!("{:?}", param.type_annotation), // Simplified type representation
            })
            .collect();
        
        let func_debug_info = FunctionDebugInfo {
            name: func_def.name.clone(),
            start_line: self.current_line,
            end_line: self.current_line, // Would be calculated properly in a full implementation
            parameters: param_debug_info,
            local_vars: Vec::new(), // Will be populated as we process the function body
        };
        
        self.debug_info.function_info.insert(func_def.name.clone(), func_debug_info);
        
        // Process function body
        for stmt in &func_def.body {
            self.generate_debug_info_for_statement(stmt);
        }
        
        // Reset current function
        self.current_function = None;
    }

    /// Generates debug information for an expression
    fn generate_debug_info_for_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                // Record usage of the variable
                self.record_variable_usage(name, self.current_line, 0); // Column 0 for now
            },
            Expression::BinaryOp(left, _, right) => {
                self.generate_debug_info_for_expression(left);
                self.generate_debug_info_for_expression(right);
            },
            Expression::Call(func_name, args) => {
                // Record function call location
                self.record_function_call_location(func_name, self.current_line, 0);
                
                // Process arguments
                for arg in args {
                    self.generate_debug_info_for_expression(arg);
                }
            },
            Expression::Integer(_) | Expression::Float(_) | Expression::String(_) | 
            Expression::Boolean(_) | Expression::Nil => {
                // For literals, just record the location
                self.record_literal_location(self.current_line, 0);
            },
            // Handle other expression types as needed
            _ => {
                // For unhandled expressions, just continue
            }
        }
    }

    /// Records the location of a variable definition
    fn record_variable_location(&mut self, name: &str, line: usize, column: usize) {
        self.debug_info.variable_locations.insert(
            name.to_string(),
            DebugLocation {
                line,
                column,
                scope: self.get_current_scope(),
            }
        );
    }

    /// Records the usage of a variable
    fn record_variable_usage(&mut self, name: &str, line: usize, column: usize) {
        // If the variable doesn't exist in current scope, it might be a global or parameter
        if !self.debug_info.variable_locations.contains_key(name) {
            // For now, just add it with the usage location
            // In a full implementation, we'd look up the definition
            self.record_variable_location(name, line, column);
        }
        
        // Also record the usage in the line mapping
        self.debug_info.line_mapping.insert(line, line);
    }

    /// Records the location of a function call
    fn record_function_call_location(&mut self, func_name: &str, line: usize, column: usize) {
        // Record the call in the line mapping
        self.debug_info.line_mapping.insert(line, line);
        
        // If we have function info for this function, we could link the call to the definition
        if let Some(func_info) = self.debug_info.function_info.get(func_name) {
            // In a full implementation, we'd create a call site debug info
            // For now, we just acknowledge that the function exists
        }
    }

    /// Records the location of a literal
    fn record_literal_location(&mut self, line: usize, column: usize) {
        // Record the literal in the line mapping
        self.debug_info.line_mapping.insert(line, line);
    }

    /// Gets the current scope ID
    fn get_current_scope(&self) -> usize {
        // For now, return 0 as the default scope
        // In a full implementation, we'd track nested scopes
        0
    }

    /// Gets the generated debug information
    pub fn get_debug_info(&self) -> &DebugInfo {
        &self.debug_info
    }

    /// Emits debug information (for now, just prints it)
    pub fn emit_debug_info(&self) {
        println!("Debug Info for file: {}", self.debug_info.file_name);
        println!("Line mapping: {:?}", self.debug_info.line_mapping);
        println!("Variable locations: {:?}", self.debug_info.variable_locations);
        println!("Function info: {:?}", self.debug_info.function_info.keys().collect::<Vec<_>>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_debug_info_generator() {
        let mut generator = DebugInfoGenerator::new("test.logos");

        // Create a simple program: let x = 42
        let program = Program {
            statements: vec![
                Statement::LetBinding {
                    mutable: false,
                    name: "x".to_string(),
                    type_annotation: Some(Type::Int),
                    value: Expression::Integer(42),
                    ownership_modifier: None,
                    lifetime_annotation: None,
                }
            ]
        };

        generator.generate_debug_info_for_program(&program);
        generator.emit_debug_info();

        // Check that the variable location was recorded
        assert!(generator.debug_info.variable_locations.contains_key("x"));
    }
}

/// Generates debugging information for a program
pub fn generate_debug_info(program: &Program, file_name: &str) -> DebugInfo {
    let mut generator = DebugInfoGenerator::new(file_name);
    generator.generate_debug_info_for_program(program);
    generator.debug_info
}