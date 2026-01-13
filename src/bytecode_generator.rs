//! Bytecode Generator for the Logos Programming Language
//! This module converts the AST to bytecode instructions for the virtual machine

use crate::ast::*;

/// Represents a single bytecode instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Constants
    LoadConstant(Constant),

    // Variables
    LoadVar(String),
    StoreVar(String),
    
    // Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    
    // Control Flow
    Jump(usize),           // Unconditional jump to instruction index
    JumpIfTrue(usize),     // Jump if top of stack is true
    JumpIfFalse(usize),    // Jump if top of stack is false
    Call(String, usize),   // Call function with name and argument count
    Return,                // Return from function
    
    // Function definitions
    DefineFunction(String, Vec<String>, Vec<Instruction>), // Function name, parameters, body
    
    // Stack operations
    Pop,
    Dup,
    
    // Print
    Print,
}

/// Constants that can be stored in the constant pool
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
}

/// Bytecode generator that converts AST to bytecode instructions
pub struct BytecodeGenerator {
    /// Current instruction list
    instructions: Vec<Instruction>,
    /// Constant pool
    constants: Vec<Constant>,
    /// Temporary variable counter
    temp_counter: usize,
}

impl BytecodeGenerator {
    /// Creates a new bytecode generator
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            temp_counter: 0,
        }
    }

    /// Generates bytecode for a program
    pub fn generate_program(&mut self, program: &Program) -> Vec<Instruction> {
        for statement in &program.statements {
            self.generate_statement(statement);
        }
        
        self.instructions.clone()
    }

    /// Generates bytecode for a statement
    fn generate_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expr) => {
                self.generate_expression(expr);
                // Pop the result since expressions as statements don't return anything
                self.instructions.push(Instruction::Pop);
            },
            Statement::LetBinding { mutable: _, name, type_annotation: _, value, ownership_modifier: _, lifetime_annotation: _ } => {
                // Generate code for the value
                self.generate_expression(value);
                // Store it in the variable
                self.instructions.push(Instruction::StoreVar(name.clone()));
            },
            Statement::Function(func_def) => {
                // Generate bytecode for the function body
                let mut func_generator = BytecodeGenerator::new();
                for stmt in &func_def.body {
                    func_generator.generate_statement(stmt);
                }
                
                let param_names: Vec<String> = func_def.parameters.iter()
                    .map(|param| param.name.clone())
                    .collect();
                
                self.instructions.push(Instruction::DefineFunction(
                    func_def.name.clone(),
                    param_names,
                    func_generator.instructions,
                ));
            },
            Statement::Return(expr) => {
                if let Some(return_expr) = expr {
                    self.generate_expression(return_expr);
                }
                self.instructions.push(Instruction::Return);
            },
            Statement::Block(statements) => {
                for stmt in statements {
                    self.generate_statement(stmt);
                }
            },
            // Handle other statement types as needed
            _ => {
                // For now, we'll just add a comment for unhandled statements
                // In a full implementation, we'd handle all statement types
            }
        }
    }

    /// Generates bytecode for an expression
    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Integer(value) => {
                let const_idx = self.add_constant(Constant::Integer(*value));
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            },
            Expression::Float(value) => {
                let const_idx = self.add_constant(Constant::Float(*value));
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            },
            Expression::String(value) => {
                let const_idx = self.add_constant(Constant::String(value.clone()));
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            },
            Expression::Boolean(value) => {
                let const_idx = self.add_constant(Constant::Boolean(*value));
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            },
            Expression::Nil => {
                let const_idx = self.add_constant(Constant::Unit);
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            },
            Expression::Identifier(name) => {
                self.instructions.push(Instruction::LoadVar(name.clone()));
            },
            Expression::BinaryOp(left, op, right) => {
                // Generate code for left operand
                self.generate_expression(left);
                // Generate code for right operand
                self.generate_expression(right);
                
                // Generate operation instruction
                match op {
                    BinaryOp::Add => self.instructions.push(Instruction::Add),
                    BinaryOp::Sub => self.instructions.push(Instruction::Sub),
                    BinaryOp::Mul => self.instructions.push(Instruction::Mul),
                    BinaryOp::Div => self.instructions.push(Instruction::Div),
                    BinaryOp::Eq => self.instructions.push(Instruction::Eq),
                    BinaryOp::Ne => self.instructions.push(Instruction::Ne),
                    BinaryOp::Lt => self.instructions.push(Instruction::Lt),
                    BinaryOp::Gt => self.instructions.push(Instruction::Gt),
                    BinaryOp::Le => self.instructions.push(Instruction::Le),
                    BinaryOp::Ge => self.instructions.push(Instruction::Ge),
                    BinaryOp::And => self.instructions.push(Instruction::And),
                    BinaryOp::Or => self.instructions.push(Instruction::Or),
                    // Add more operations as needed
                    _ => {
                        // For now, we'll just push a unit value for unhandled operations
                        let const_idx = self.add_constant(Constant::Unit);
                        self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
                    }
                }
            },
            Expression::Call(name, args) => {
                // Generate code for each argument
                for arg in args {
                    self.generate_expression(arg);
                }
                
                // Call the function with the number of arguments
                self.instructions.push(Instruction::Call(name.clone(), args.len()));
            },
            // Handle other expression types as needed
            _ => {
                // For now, we'll just push a unit value for unhandled expressions
                let const_idx = self.add_constant(Constant::Unit);
                self.instructions.push(Instruction::LoadConstant(self.constants[const_idx].clone()));
            }
        }
    }

    /// Adds a constant to the constant pool and returns its index
    fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
}