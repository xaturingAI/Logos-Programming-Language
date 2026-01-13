//! Bytecode Interpreter for the Logos Programming Language
//! This module provides a virtual machine that executes Logos bytecode instructions

use std::collections::HashMap;
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

/// A value in the virtual machine
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
    Function(String, Vec<String>, Vec<Instruction>), // Function name, parameters, bytecode
    Tuple(Vec<Value>),
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
}

/// A call frame on the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function name
    pub function_name: String,
    /// Local variables
    pub locals: HashMap<String, Value>,
    /// Return address
    pub return_address: usize,
    /// Base pointer for local variables
    pub base_pointer: usize,
}

/// Virtual machine to execute bytecode
pub struct VM {
    /// Stack for values
    stack: Vec<Value>,
    /// Call stack for function calls
    call_stack: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Program counter
    pc: usize,
    /// The bytecode program to execute
    program: Vec<Instruction>,
    /// Constant pool
    constants: Vec<Constant>,
    /// Function definitions
    functions: HashMap<String, (Vec<String>, Vec<Instruction>)>, // (parameters, bytecode)
}

impl VM {
    /// Creates a new virtual machine instance
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            pc: 0,
            program: Vec::new(),
            constants: Vec::new(),
            functions: HashMap::new(),
        }
    }

    /// Loads a program into the VM
    pub fn load_program(&mut self, program: Vec<Instruction>) {
        self.program = program;
        self.pc = 0;
    }

    /// Executes the loaded program
    pub fn execute(&mut self) -> Result<(), String> {
        while self.pc < self.program.len() {
            let instruction = &self.program[self.pc];
            self.execute_instruction(instruction)?;
            self.pc += 1;
        }
        Ok(())
    }

    /// Executes a single instruction
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), String> {
        match instruction {
            Instruction::LoadConstant(constant) => {
                let value = self.constant_to_value(constant);
                self.stack.push(value);
                Ok(())
            },
            Instruction::LoadVar(name) => {
                if let Some(value) = self.globals.get(name) {
                    self.stack.push(value.clone());
                    Ok(())
                } else {
                    Err(format!("Variable '{}' not found", name))
                }
            },
            Instruction::StoreVar(name) => {
                if let Some(value) = self.stack.pop() {
                    self.globals.insert(name.clone(), value);
                    Ok(())
                } else {
                    Err("Stack underflow: trying to store variable without value".to_string())
                }
            },
            Instruction::Add => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for addition".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a + b as f64),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(a as f64 + b),
                    _ => return Err("Type error: can only add numbers".to_string()),
                };
                
                self.stack.push(result);
                Ok(())
            },
            Instruction::Sub => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for subtraction".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a - b as f64),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(a as f64 - b),
                    _ => return Err("Type error: can only subtract numbers".to_string()),
                };
                
                self.stack.push(result);
                Ok(())
            },
            Instruction::Mul => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for multiplication".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a * b as f64),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(a as f64 * b),
                    _ => return Err("Type error: can only multiply numbers".to_string()),
                };
                
                self.stack.push(result);
                Ok(())
            },
            Instruction::Div => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for division".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Value::Integer(a / b)
                    },
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        Value::Float(a / b)
                    },
                    (Value::Float(a), Value::Integer(b)) => {
                        if b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Value::Float(a / b as f64)
                    },
                    (Value::Integer(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        Value::Float(a as f64 / b)
                    },
                    _ => return Err("Type error: can only divide numbers".to_string()),
                };
                
                self.stack.push(result);
                Ok(())
            },
            Instruction::Jump(offset) => {
                self.pc = *offset;
                // Subtract 1 because the main loop will increment pc after this function returns
                self.pc = self.pc.saturating_sub(1);
                Ok(())
            },
            Instruction::JumpIfTrue(offset) => {
                if self.stack.is_empty() {
                    return Err("Stack underflow: need value for conditional jump".to_string());
                }
                
                let value = self.stack.pop().unwrap();
                if let Value::Boolean(true) = value {
                    self.pc = *offset;
                    // Subtract 1 because the main loop will increment pc after this function returns
                    self.pc = self.pc.saturating_sub(1);
                }
                Ok(())
            },
            Instruction::JumpIfFalse(offset) => {
                if self.stack.is_empty() {
                    return Err("Stack underflow: need value for conditional jump".to_string());
                }
                
                let value = self.stack.pop().unwrap();
                if let Value::Boolean(false) = value {
                    self.pc = *offset;
                    // Subtract 1 because the main loop will increment pc after this function returns
                    self.pc = self.pc.saturating_sub(1);
                }
                Ok(())
            },
            Instruction::Call(name, arg_count) => {
                // Look up the function
                if let Some((params, body)) = self.functions.get(name).cloned() {
                    // Create a new call frame
                    let mut locals = HashMap::new();
                    
                    // Pop arguments from the stack and assign to parameters
                    for (i, param) in params.iter().rev().enumerate() {
                        if i < *arg_count && !self.stack.is_empty() {
                            let arg = self.stack.pop().unwrap();
                            locals.insert(param.clone(), arg);
                        }
                    }
                    
                    let frame = CallFrame {
                        function_name: name.clone(),
                        locals,
                        return_address: self.pc + 1, // Return to next instruction
                        base_pointer: self.stack.len(), // Base for local variables
                    };
                    
                    self.call_stack.push(frame);
                    
                    // For now, just return - in a real implementation, we'd execute the function body
                    Ok(())
                } else {
                    Err(format!("Function '{}' not found", name))
                }
            },
            Instruction::Return => {
                if let Some(frame) = self.call_stack.pop() {
                    self.pc = frame.return_address;
                    // Subtract 1 because the main loop will increment pc after this function returns
                    self.pc = self.pc.saturating_sub(1);
                }
                Ok(())
            },
            Instruction::Pop => {
                if !self.stack.is_empty() {
                    self.stack.pop();
                    Ok(())
                } else {
                    Err("Stack underflow: trying to pop from empty stack".to_string())
                }
            },
            Instruction::Dup => {
                if let Some(top) = self.stack.last().cloned() {
                    self.stack.push(top);
                    Ok(())
                } else {
                    Err("Stack underflow: trying to duplicate top of empty stack".to_string())
                }
            },
            Instruction::Print => {
                if let Some(value) = self.stack.pop() {
                    println!("{}", self.value_to_string(&value));
                    Ok(())
                } else {
                    Err("Stack underflow: trying to print without value".to_string())
                }
            },
            Instruction::DefineFunction(name, params, body) => {
                // Store function definition
                self.functions.insert(name.clone(), (params.clone(), body.clone()));
                Ok(())
            },
            Instruction::Eq => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for equality".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = Value::Boolean(self.values_equal(&left, &right));
                self.stack.push(result);
                Ok(())
            },
            Instruction::Ne => {
                if self.stack.len() < 2 {
                    return Err("Stack underflow: need 2 values for inequality".to_string());
                }
                
                let right = self.stack.pop().unwrap();
                let left = self.stack.pop().unwrap();
                
                let result = Value::Boolean(!self.values_equal(&left, &right));
                self.stack.push(result);
                Ok(())
            },
            // Handle other operations
            _ => Err("Instruction not yet implemented".to_string()),
        }
    }

    /// Converts a constant to a value
    fn constant_to_value(&self, constant: &Constant) -> Value {
        match constant {
            Constant::Integer(i) => Value::Integer(*i),
            Constant::Float(f) => Value::Float(*f),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Boolean(b) => Value::Boolean(*b),
            Constant::Unit => Value::Unit,
        }
    }

    /// Converts a value to a string representation
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Unit => "()".to_string(),
            Value::Function(name, _, _) => format!("<function {}>", name),
            Value::Tuple(items) => {
                let item_strs: Vec<String> = items.iter().map(|v| self.value_to_string(v)).collect();
                format!("({})", item_strs.join(", "))
            },
            Value::Array(items) => {
                let item_strs: Vec<String> = items.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", item_strs.join(", "))
            },
            Value::Struct(fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            },
        }
    }

    /// Checks if two values are equal
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            _ => false, // Different types are not equal
        }
    }
}

/// Bytecode generator that converts AST to bytecode
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
                    // Add more operations as needed
                    _ => {
                        // For now, we'll just push a dummy instruction
                        // In a full implementation, we'd handle all operations
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_execution() {
        let mut vm = VM::new();
        
        // Create a simple program: push 42, print it
        let program = vec![
            Instruction::LoadConstant(Constant::Integer(42)),
            Instruction::Print,
        ];
        
        vm.load_program(program);
        assert!(vm.execute().is_ok());
    }

    #[test]
    fn test_bytecode_generation() {
        use crate::ast::*;
        
        // Create a simple AST: let x = 42
        let program = Program {
            statements: vec![
                Statement::LetBinding {
                    mutable: false,
                    name: "x".to_string(),
                    type_annotation: None,
                    value: Expression::Integer(42),
                    ownership_modifier: None,
                    lifetime_annotation: None,
                }
            ]
        };
        
        let mut generator = BytecodeGenerator::new();
        let bytecode = generator.generate_program(&program);
        
        assert!(!bytecode.is_empty());
    }
}