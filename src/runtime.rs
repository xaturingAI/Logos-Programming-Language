// Logos Programming Language Runtime
// This module provides the runtime environment for executing Logos programs.
// It handles value representation, evaluation, and execution of the AST.

use crate::ast::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents different types of values in the Logos runtime
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
    Char(char),                           // Character value
    Array(Vec<Value>),
    Tuple(Vec<Value>),                    // For tuple values
    Struct(String, HashMap<String, Value>), // For struct values
    Function(String, Vec<Parameter>, Vec<Statement>, Environment), // Function with closure environment
    BuiltinFunction(fn(&[Value]) -> Result<Value, String>), // Built-in functions
    Future(Box<Value>),                   // For async/await futures
    Task(Box<Value>),                     // For spawned tasks
    Closure(Vec<Parameter>, Vec<Statement>, Environment), // For closures/anonymous functions
    // Add more value types as needed
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),  // Properly quote strings
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Char(c) => write!(f, "'{}'", c),      // Properly quote characters
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            },
            Value::Tuple(tuple) => {
                let elements: Vec<String> = tuple.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", elements.join(", "))
            },
            Value::Struct(name, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(field_name, field_val)| format!("{}: {}", field_name, field_val))
                    .collect();
                write!(f, "{} {{ {} }}", name, field_strs.join(", "))
            },
            Value::Function(name, _, _, _) => write!(f, "<function {}>", name),
            Value::BuiltinFunction(_) => write!(f, "<builtin function>"),
            Value::Future(_) => write!(f, "<future>"),
            Value::Task(_) => write!(f, "<task>"),
            Value::Closure(_, _, _) => write!(f, "<closure>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Future(a), Value::Future(b)) => a == b,  // Compare the wrapped values
            (Value::Task(a), Value::Task(b)) => a == b,      // Compare the wrapped values
            _ => false, // Different types or functions are not equal
        }
    }
}

/// Environment for variable bindings during execution
#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Creates a new environment with optional parent
    pub fn new(parent: Option<Environment>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: parent.map(Box::new),
        }
    }

    /// Gets a value from the environment
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.get(name)
                } else {
                    None
                }
            }
        }
    }

    /// Sets a value in the environment
    pub fn set(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// Checks if a variable exists in the environment
    pub fn contains(&self, name: &str) -> bool {
        if self.values.contains_key(name) {
            true
        } else if let Some(ref parent) = self.parent {
            parent.contains(name)
        } else {
            false
        }
    }
}

/// The Runtime struct manages program execution
pub struct Runtime {
    pub env: Environment,
    recursion_depth: usize,  // Track recursion depth to prevent stack overflow
    max_recursion_depth: usize,  // Maximum allowed recursion depth
}

impl Runtime {
    /// Creates a new runtime instance with built-in functions
    pub fn new() -> Self {
        let mut env = Environment::new(None);
        
        // Register built-in functions
        env.set("print".to_string(), Value::BuiltinFunction(runtime_print));
        env.set("len".to_string(), Value::BuiltinFunction(runtime_len));
        env.set("str".to_string(), Value::BuiltinFunction(runtime_str));
        env.set("int".to_string(), Value::BuiltinFunction(runtime_int));
        env.set("float".to_string(), Value::BuiltinFunction(runtime_float));
        
        Runtime {
            env,
            recursion_depth: 0,
            max_recursion_depth: 100,  // Reasonable default to prevent stack overflow
        }
    }

    /// Evaluates a program (sequence of statements)
    pub fn eval_program(&mut self, program: &Program) -> Result<Value, String> {
        let mut result = Value::Unit;

        for statement in &program.statements {
            result = self.eval_statement(statement)?;
        }

        Ok(result)
    }

    /// Executes a program (same as eval_program)
    pub fn execute_program(&mut self, program: &Program) -> Result<Value, String> {
        self.eval_program(program)
    }

    /// Evaluates a single statement
    pub fn eval_statement(&mut self, statement: &Statement) -> Result<Value, String> {
        match statement {
            Statement::Expression(expr) => self.eval_expression(expr),
            Statement::LetBinding { mutable, name, type_annotation: _, value, ownership_modifier: _, lifetime_annotation: _ } => {
                let value = self.eval_expression(value)?;
                
                // In a real implementation, we'd check mutability and type annotations
                self.env.set(name.clone(), value);
                Ok(Value::Unit)
            },
            Statement::ConstBinding { name, type_annotation: _, value } => {
                let value = self.eval_expression(value)?;
                self.env.set(name.clone(), value);
                Ok(Value::Unit)
            },
            Statement::Function(func_def) => {
                let func_val = Value::Function(
                    func_def.name.clone(),
                    func_def.parameters.clone(),
                    func_def.body.clone(),
                    self.env.clone(), // Capture current environment for closures
                );
                self.env.set(func_def.name.clone(), func_val);
                Ok(Value::Unit)
            },
            Statement::Return(expr) => {
                match expr {
                    Some(e) => self.eval_expression(e),
                    None => Ok(Value::Unit),
                }
            },
            Statement::Break => Err("Break outside loop".to_string()),
            Statement::Continue => Err("Continue outside loop".to_string()),
            Statement::Block(statements) => {
                // Create a new environment scope for the block
                let mut block_runtime = Runtime::new();
                block_runtime.env = Environment::new(Some(self.env.clone()));

                let mut result = Value::Unit;
                for stmt in statements {
                    result = block_runtime.eval_statement(stmt)?;
                }

                Ok(result)
            },
            Statement::Trait(_) => {
                // Traits are compile-time constructs, so at runtime we just acknowledge them
                Ok(Value::Unit)
            },
            Statement::Implementation(_) => {
                // Implementations are compile-time constructs, so at runtime we just acknowledge them
                Ok(Value::Unit)
            },
            Statement::MacroDefinition(_) => {
                // Macro definitions are compile-time constructs, so at runtime we just acknowledge them
                Ok(Value::Unit)
            },
            // Handle other statement types as needed
            _ => Err("Unsupported statement type".to_string()),
        }
    }

    /// Evaluates an expression
    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Integer(val) => Ok(Value::Integer(*val)),
            Expression::Float(val) => Ok(Value::Float(*val)),
            Expression::String(val) => Ok(Value::String(val.clone())),
            Expression::Boolean(val) => Ok(Value::Boolean(*val)),
            Expression::Nil => Ok(Value::Unit),
            Expression::Identifier(name) => {
                match self.env.get(name) {
                    Some(value) => Ok(value),
                    None => Err(format!("Undefined variable: {}", name)),
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                
                match op {
                    BinaryOp::Add => binary_op_add(left_val, right_val),
                    BinaryOp::Sub => binary_op_sub(left_val, right_val),
                    BinaryOp::Mul => binary_op_mul(left_val, right_val),
                    BinaryOp::Div => binary_op_div(left_val, right_val),
                    BinaryOp::Mod => binary_op_mod(left_val, right_val),
                    BinaryOp::Eq => Ok(Value::Boolean(left_val == right_val)),
                    BinaryOp::Ne => Ok(Value::Boolean(left_val != right_val)),
                    BinaryOp::Lt => binary_op_lt(left_val, right_val),
                    BinaryOp::Gt => binary_op_gt(left_val, right_val),
                    BinaryOp::Le => binary_op_le(left_val, right_val),
                    BinaryOp::Ge => binary_op_ge(left_val, right_val),
                    BinaryOp::And => binary_op_and(left_val, right_val),
                    BinaryOp::Or => binary_op_or(left_val, right_val),
                    BinaryOp::PipeForward => {
                        // Implement pipe forward operator (value |> function)
                        // This passes the left value as the first argument to the right function
                        match (&left_val, &right_val) {
                            (value, Value::Function(func_name, params, body, closure_env)) => {
                                // Create a new environment for the function call
                                let mut func_env = Environment::new(Some(closure_env.clone()));
                                
                                // Use the left value as the first argument
                                if !params.is_empty() {
                                    func_env.set(params[0].name.clone(), value.clone());
                                }
                                
                                // Evaluate the function body in the new environment
                                let mut func_runtime = Runtime::new();
                                func_runtime.env = func_env;
                                
                                let mut result = Value::Unit;
                                for stmt in body {
                                    result = func_runtime.eval_statement(stmt)?;
                                }
                                Ok(result)
                            },
                            (value, Value::BuiltinFunction(func)) => {
                                // Apply the builtin function to the value
                                func(&[value.clone()])
                            },
                            (_, _) => Err("Pipe forward expects a function on the right side".to_string()),
                        }
                    },
                    BinaryOp::PipeBackward => {
                        // Implement pipe backward operator (function <| value)
                        // This passes the right value as the first argument to the left function
                        match (&left_val, &right_val) {
                            (Value::Function(func_name, params, body, closure_env), value) => {
                                // Create a new environment for the function call
                                let mut func_env = Environment::new(Some(closure_env.clone()));
                                
                                // Use the right value as the first argument
                                if !params.is_empty() {
                                    func_env.set(params[0].name.clone(), value.clone());
                                }
                                
                                // Evaluate the function body in the new environment
                                let mut func_runtime = Runtime::new();
                                func_runtime.env = func_env;
                                
                                let mut result = Value::Unit;
                                for stmt in body {
                                    result = func_runtime.eval_statement(stmt)?;
                                }
                                Ok(result)
                            },
                            (Value::BuiltinFunction(func), value) => {
                                // Apply the builtin function to the value
                                func(&[value.clone()])
                            },
                            (_, _) => Err("Pipe backward expects a function on the left side".to_string()),
                        }
                    },
                    BinaryOp::Power => binary_op_power(left_val, right_val),
                    BinaryOp::Range => {
                        // Implement range operator (start..end)
                        match (&left_val, &right_val) {
                            (Value::Integer(start), Value::Integer(end)) => {
                                // Create a range value
                                let mut elements = Vec::new();
                                let start_val = *start;
                                let end_val = *end;
                                
                                if start_val <= end_val {
                                    for i in start_val..=end_val {
                                        elements.push(Value::Integer(i));
                                    }
                                } else {
                                    // Create a reverse range if start > end
                                    for i in (end_val..=start_val).rev() {
                                        elements.push(Value::Integer(i));
                                    }
                                }
                                
                                Ok(Value::Array(elements))
                            },
                            (_, _) => Err("Range operator expects integer operands".to_string()),
                        }
                    },
                    BinaryOp::Spaceship => binary_op_spaceship(left_val, right_val),
                }
            },
            Expression::UnaryOp(op, expr) => {
                let val = self.eval_expression(expr)?;
                match op {
                    UnaryOp::Neg => unary_op_neg(val),
                    UnaryOp::Not => unary_op_not(val),
                    _ => Err("Unsupported unary operation".to_string()),
                }
            },
            Expression::Call(name, args) => {
                let func_val = self.env.get(name).ok_or_else(|| format!("Function not found: {}", name))?;
                
                match func_val {
                    Value::BuiltinFunction(func) => {
                        let evaluated_args: Result<Vec<Value>, String> = 
                            args.iter().map(|arg| self.eval_expression(arg)).collect();
                        let args = evaluated_args?;
                        func(&args)
                    },
                    Value::Function(_, params, body, closure_env) => {
                        if params.len() != args.len() {
                            return Err(format!("Argument count mismatch for function {}", name));
                        }
                        
                        // Create new environment for function execution
                        let mut func_env = Environment::new(Some(closure_env));
                        
                        // Bind parameters to arguments
                        for (param, arg) in params.iter().zip(args.iter()) {
                            let arg_val = self.eval_expression(arg)?;
                            func_env.set(param.name.clone(), arg_val);
                        }
                        
                        // Create a new environment for function execution
                        let mut func_runtime = Runtime::new();
                        func_runtime.env = func_env;

                        let mut result = Value::Unit;
                        for stmt in body {
                            result = func_runtime.eval_statement(&stmt)?;
                            // If we encounter a return, we should return that value
                            if matches!(stmt, Statement::Return(_)) {
                                break;
                            }
                        }

                        Ok(result)
                    },
                    _ => Err(format!("Not a function: {}", name)),
                }
            },
            Expression::If(condition, then_stmts, else_stmts) => {
                let cond_val = self.eval_expression(condition)?;

                if is_truthy(&cond_val) {
                    // Evaluate then branch with a new runtime to avoid borrowing issues
                    let mut then_runtime = Runtime::new();
                    then_runtime.env = Environment::new(Some(self.env.clone()));

                    let mut result = Value::Unit;
                    for stmt in then_stmts {
                        result = then_runtime.eval_statement(stmt)?;
                    }

                    Ok(result)
                } else {
                    // Evaluate else branch with a new runtime to avoid borrowing issues
                    let mut else_runtime = Runtime::new();
                    else_runtime.env = Environment::new(Some(self.env.clone()));

                    let mut result = Value::Unit;
                    for stmt in else_stmts {
                        result = else_runtime.eval_statement(stmt)?;
                    }

                    Ok(result)
                }
            },
            Expression::Char(c) => Ok(Value::Char(*c)),
            Expression::Array(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval_expression(item)?);
                }
                Ok(Value::Array(values))
            },
            Expression::Tuple(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval_expression(item)?);
                }
                Ok(Value::Tuple(values))
            },
            Expression::Struct(name, fields) => {
                let mut field_map = HashMap::new();
                for (field_name, field_expr) in fields {
                    field_map.insert(field_name.clone(), self.eval_expression(field_expr)?);
                }
                Ok(Value::Struct(name.clone(), field_map))
            },
            Expression::Lambda(params, body) => {
                // Create a closure with the current environment
                Ok(Value::Closure(params.clone(), body.clone(), self.env.clone()))
            },
            Expression::AsyncBlock(statements) => {
                // Create a future that represents the async block
                // In a real implementation, this would create a proper future and schedule it
                let mut result = Value::Unit;
                for stmt in statements {
                    result = self.eval_statement(stmt)?;
                }
                // Wrap the result in a Future value
                Ok(Value::Future(Box::new(result)))
            },
            Expression::Await(expr) => {
                // In a real implementation, this would await a future
                // For now, we'll extract the value from a Future if present
                let future_val = self.eval_expression(expr)?;
                match future_val {
                    Value::Future(inner_val) => Ok(*inner_val),
                    _ => Ok(future_val), // If not a future, return as-is
                }
            },
            Expression::Future(expr) => {
                // Wrap the expression in a future
                let val = self.eval_expression(expr)?;
                Ok(Value::Future(Box::new(val)))
            },
            Expression::SpawnTask(expr) => {
                // In a real implementation, this would spawn a task in the async runtime
                // For now, we'll just evaluate the expression and wrap in a Task value
                let val = self.eval_expression(expr)?;
                Ok(Value::Task(Box::new(val)))
            },
            Expression::Join(task) => {
                // In a real implementation, this would join a spawned task
                // For now, we'll just evaluate the task expression
                self.eval_expression(task)
            },
            Expression::Race(tasks) => {
                // For now, we'll just evaluate the first task
                // In a real implementation, this would race multiple tasks
                if tasks.is_empty() {
                    Ok(Value::Unit)
                } else {
                    self.eval_expression(&tasks[0])
                }
            },
            Expression::Timeout(expr, _duration) => {
                // For now, we'll just evaluate the expression directly
                // In a real implementation, this would implement timeout functionality
                self.eval_expression(expr)
            },
            Expression::Match(expr, arms) => {
                let match_value = self.eval_expression(expr)?;

                for (pattern, guard, body) in arms {
                    // Create a new environment for this match arm
                    let mut arm_env = Environment::new(Some(self.env.clone()));
                    let mut runtime_for_arm = Runtime::new();
                    runtime_for_arm.env = arm_env;

                    // Check if the pattern matches the value
                    if self.pattern_matches(&match_value, pattern, &mut runtime_for_arm)? {
                        // If there's a guard, evaluate it
                        if let Some(guard_expr) = guard {
                            let guard_result = runtime_for_arm.eval_expression(guard_expr)?;
                            if is_truthy(&guard_result) {
                                // Guard passed, execute the body
                                let mut result = Value::Unit;
                                for stmt in body {
                                    result = runtime_for_arm.eval_statement(stmt)?;
                                }
                                return Ok(result);
                            }
                        } else {
                            // No guard, execute the body
                            let mut result = Value::Unit;
                            for stmt in body {
                                result = runtime_for_arm.eval_statement(stmt)?;
                            }
                            return Ok(result);
                        }
                    }
                }

                // No match found
                Err("No matching pattern found".to_string())
            },
            Expression::MacroInvocation(name, args) => {
                // For now, we'll treat macro invocations specially
                // In a full implementation, this would expand the macro at compile time
                // For demonstration purposes, we'll just return a special macro invocation value
                let evaluated_args: Result<Vec<Value>, String> =
                    args.iter().map(|arg| self.eval_expression(arg)).collect();
                let args_values = evaluated_args?;

                // In a real implementation, macro expansion would happen here at compile time
                // For now, we'll return a string representation of the macro invocation
                Ok(Value::String(format!("MACRO_INVOCATION: {} with args {:?}", name, args_values)))
            },
            Expression::MultiLangCall(lang, code) => {
                // Handle multi-language calls by delegating to the appropriate interpreter
                // In a real implementation, this would call the appropriate language runtime
                match lang.as_str() {
                    "python" => {
                        Ok(self.execute_python_code(&code)?)
                    },
                    "rust" => {
                        Ok(self.execute_rust_code(&code)?)
                    },
                    "go" => {
                        Ok(self.execute_go_code(&code)?)
                    },
                    "js" | "javascript" => {
                        Ok(self.execute_javascript_code(&code)?)
                    },
                    "java" => {
                        Ok(self.execute_java_code(&code)?)
                    },
                    "c" => {
                        Ok(self.execute_c_code(&code)?)
                    },
                    _ => {
                        // Unknown language, return an error
                        Err(format!("Unknown language for multi-language call: {}", lang))
                    }
                }
            },
            Expression::MultiLangImport(lang, resource, resource_type) => {
                // Handle multi-language imports
                // In a real implementation, this would import code from another language
                // For now, we'll simulate it
                Ok(Value::String(format!("Imported {} from {} (type: {:?})", resource, lang, resource_type)))
            },
            Expression::MultiLangIndex(indexer, resource) => {
                // Handle multi-language indexing
                // In a real implementation, this would index resources from another language
                // For now, we'll simulate it
                Ok(Value::String(format!("Indexed {} using {} indexer", resource, indexer)))
            },
            Expression::Block(statements) => {
                // Create a new environment for the block
                let mut block_env = Environment::new(Some(self.env.clone()));
                let mut block_runtime = Runtime::new();
                block_runtime.env = block_env;

                let mut result = Value::Unit;
                for stmt in statements {
                    result = block_runtime.eval_statement(stmt)?;
                }

                Ok(result)
            },
            // Handle other expression types as needed
            _ => Err("Unsupported expression type".to_string()),
        }
    }

    /// Executes Python code from within Logos
    fn execute_python_code(&mut self, code: &str) -> Result<Value, String> {
        #[cfg(feature = "python")]
        {
            use pyo3::prelude::*;

            Python::with_gil(|py| {
                match PyModule::from_code(py, code, "logos_embedded", "logos_embedded") {
                    Ok(module) => {
                        match module.call_method0("__main__") {
                            Ok(result) => {
                                // Convert Python result to Logos Value
                                // For now, we'll just return a string representation
                                Ok(Value::String(format!("{:?}", result)))
                            },
                            Err(e) => Err(format!("Python execution error: {}", e))
                        }
                    },
                    Err(e) => Err(format!("Python compilation error: {}", e))
                }
            })
        }
        #[cfg(not(feature = "python"))]
        {
            // If Python feature is not enabled, return an error
            Err("Python support not enabled (compile with --features python)".to_string())
        }
    }

    /// Executes Rust code from within Logos
    fn execute_rust_code(&mut self, code: &str) -> Result<Value, String> {
        // In a real implementation, this would compile and execute Rust code
        // For now, we'll return a placeholder
        Ok(Value::String(format!("Rust code executed: {}", code)))
    }

    /// Executes Go code from within Logos
    fn execute_go_code(&mut self, code: &str) -> Result<Value, String> {
        // In a real implementation, this would call Go code
        // For now, we'll return a placeholder
        Ok(Value::String(format!("Go code executed: {}", code)))
    }

    /// Executes JavaScript code from within Logos
    fn execute_javascript_code(&mut self, code: &str) -> Result<Value, String> {
        #[cfg(feature = "javascript")]
        {
            // In a real implementation, this would execute JavaScript code using V8 or another engine
            // For now, we'll return a placeholder
            Ok(Value::String(format!("JavaScript code executed: {}", code)))
        }
        #[cfg(not(feature = "javascript"))]
        {
            // If JavaScript feature is not enabled, return an error
            Err("JavaScript support not enabled (compile with --features javascript)".to_string())
        }
    }

    /// Executes Java code from within Logos
    fn execute_java_code(&mut self, code: &str) -> Result<Value, String> {
        #[cfg(feature = "java")]
        {
            // In a real implementation, this would execute Java code using JNI
            // For now, we'll return a placeholder
            Ok(Value::String(format!("Java code executed: {}", code)))
        }
        #[cfg(not(feature = "java"))]
        {
            // If Java feature is not enabled, return an error
            Err("Java support not enabled (compile with --features java)".to_string())
        }
    }

    /// Executes C code from within Logos
    fn execute_c_code(&mut self, code: &str) -> Result<Value, String> {
        // In a real implementation, this would compile and execute C code
        // For now, we'll return a placeholder
        Ok(Value::String(format!("C code executed: {}", code)))
    }

    /// Checks if a value matches a pattern, binding variables to the environment if it does
    fn pattern_matches(&mut self, value: &Value, pattern: &Pattern, runtime_for_arm: &mut Runtime) -> Result<bool, String> {
        // Check recursion depth to prevent stack overflow
        if self.recursion_depth >= self.max_recursion_depth {
            return Err("Recursion depth exceeded in pattern matching".to_string());
        }

        // Increase recursion depth
        self.recursion_depth += 1;

        // Decrement recursion depth when function exits
        let result = match pattern {
            Pattern::Identifier(name) => {
                // Bind the value to the identifier in the arm's environment
                runtime_for_arm.env.set(name.clone(), value.clone());
                Ok(true)
            },
            Pattern::Literal(literal_expr) => {
                let literal_value = self.eval_expression(literal_expr)?;
                Ok(value == &literal_value)
            },
            Pattern::Wildcard => Ok(true), // Wildcard always matches
            Pattern::Tuple(pattern_items) => {
                if let Value::Tuple(value_items) = value {
                    if pattern_items.len() != value_items.len() {
                        return Ok(false);
                    }

                    // Check each element in the tuple
                    for (pattern_item, value_item) in pattern_items.iter().zip(value_items.iter()) {
                        if !self.pattern_matches(value_item, pattern_item, runtime_for_arm)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false) // Value is not a tuple
                }
            },
            Pattern::Array(pattern_items) => {
                if let Value::Array(value_items) = value {
                    if pattern_items.len() != value_items.len() {
                        return Ok(false);
                    }

                    // Check each element in the array
                    for (pattern_item, value_item) in pattern_items.iter().zip(value_items.iter()) {
                        if !self.pattern_matches(value_item, pattern_item, runtime_for_arm)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false) // Value is not an array
                }
            },
            Pattern::Struct(name, fields) => {
                // For now, we'll just return false since we don't have struct support in values
                // In a full implementation, we would check if the value is a struct with the given name
                // and match the specified fields
                Err(format!("Struct pattern matching not implemented for: {}", name))
            },
            Pattern::Or(left, right) => {
                // Create a copy of the environment for the left pattern
                let original_env = runtime_for_arm.env.clone();

                // Try left pattern
                if self.pattern_matches(value, left, runtime_for_arm)? {
                    return Ok(true);
                }

                // If left didn't match, reset environment and try right pattern
                runtime_for_arm.env = original_env;
                self.pattern_matches(value, right, runtime_for_arm)
            },
            Pattern::Enum(enum_name, variant_name, sub_patterns) => {
                // For now, we'll return an error since we don't have enum support in values
                // In a full implementation, we would check if the value is an enum with the given name and variant
                Err(format!("Enum pattern matching not implemented for: {}::{}", enum_name, variant_name))
            },
            Pattern::Range(start, end) => {
                // Match against integer values in a range
                if let Value::Integer(int_val) = value {
                    Ok(*start <= *int_val && *int_val <= *end)
                } else {
                    Ok(false)
                }
            },
            Pattern::Irrefutable(inner_pattern) => {
                // An irrefutable pattern should always match
                self.pattern_matches(value, inner_pattern, runtime_for_arm)
            },
            Pattern::Guard(pattern, guard_expr) => {
                // First check if the pattern matches
                if self.pattern_matches(value, pattern, runtime_for_arm)? {
                    // Then evaluate the guard expression in the current environment
                    let guard_result = self.eval_expression(guard_expr)?;
                    // The guard must evaluate to true
                    if let Value::Boolean(true) = guard_result {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            },
        };

        // Decrease recursion depth
        self.recursion_depth -= 1;

        result
    }
}

// Helper functions for binary operations
fn binary_op_add(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
        _ => Err("Cannot add values of different types".to_string()),
    }
}

fn binary_op_sub(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
        _ => Err("Cannot subtract values of different types".to_string()),
    }
}

fn binary_op_mul(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
        _ => Err("Cannot multiply values of different types".to_string()),
    }
}

fn binary_op_div(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Integer(a / b))
            }
        },
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(a / b))
            }
        },
        (Value::Float(a), Value::Integer(b)) => {
            if b == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(a / b as f64))
            }
        },
        (Value::Integer(a), Value::Float(b)) => {
            if b == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Value::Float(a as f64 / b))
            }
        },
        _ => Err("Cannot divide values of different types".to_string()),
    }
}

fn binary_op_mod(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b == 0 {
                Err("Modulo by zero".to_string())
            } else {
                Ok(Value::Integer(a % b))
            }
        },
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                Err("Modulo by zero".to_string())
            } else {
                Ok(Value::Float(a % b))
            }
        },
        _ => Err("Cannot perform modulo on values of different types".to_string()),
    }
}

fn binary_op_lt(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
        _ => Err("Cannot compare values of different types".to_string()),
    }
}

fn binary_op_gt(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
        _ => Err("Cannot compare values of different types".to_string()),
    }
}

fn binary_op_le(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
        _ => Err("Cannot compare values of different types".to_string()),
    }
}

fn binary_op_ge(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
        _ => Err("Cannot compare values of different types".to_string()),
    }
}

fn binary_op_and(left: Value, right: Value) -> Result<Value, String> {
    Ok(Value::Boolean(is_truthy(&left) && is_truthy(&right)))
}

fn binary_op_or(left: Value, right: Value) -> Result<Value, String> {
    Ok(Value::Boolean(is_truthy(&left) || is_truthy(&right)))
}

fn binary_op_power(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float((a as f64).powf(b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(b as f64))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
        _ => Err("Cannot raise to power values of different types".to_string()),
    }
}

fn binary_op_spaceship(left: Value, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            if a < b { Ok(Value::Integer(-1)) }
            else if a > b { Ok(Value::Integer(1)) }
            else { Ok(Value::Integer(0)) }
        },
        (Value::Float(a), Value::Float(b)) => {
            if a < b { Ok(Value::Integer(-1)) }
            else if a > b { Ok(Value::Integer(1)) }
            else { Ok(Value::Integer(0)) }
        },
        (Value::String(a), Value::String(b)) => {
            if a < b { Ok(Value::Integer(-1)) }
            else if a > b { Ok(Value::Integer(1)) }
            else { Ok(Value::Integer(0)) }
        },
        _ => Err("Cannot compare values of different types for spaceship operator".to_string()),
    }
}

// Helper functions for unary operations
fn unary_op_neg(val: Value) -> Result<Value, String> {
    match val {
        Value::Integer(i) => Ok(Value::Integer(-i)),
        Value::Float(f) => Ok(Value::Float(-f)),
        _ => Err("Cannot negate non-numeric value".to_string()),
    }
}

fn unary_op_not(val: Value) -> Result<Value, String> {
    Ok(Value::Boolean(!is_truthy(&val)))
}

// Helper function to determine truthiness
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Boolean(b) => *b,
        Value::Integer(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Unit => false,
        Value::Array(arr) => !arr.is_empty(),
        Value::Tuple(tup) => !tup.is_empty(),
        _ => true, // Functions and other values are truthy
    }
}

// Built-in function implementations
fn runtime_print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Unit)
}

fn runtime_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("len() expects exactly one argument".to_string());
    }
    
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Tuple(tup) => Ok(Value::Integer(tup.len() as i64)),
        _ => Err("len() expects a string, array, or tuple".to_string()),
    }
}

fn runtime_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("str() expects exactly one argument".to_string());
    }
    
    Ok(Value::String(args[0].to_string()))
}

fn runtime_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("int() expects exactly one argument".to_string());
    }
    
    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(*i)),
        Value::Float(f) => Ok(Value::Integer(*f as i64)),
        Value::String(s) => {
            match s.parse::<i64>() {
                Ok(i) => Ok(Value::Integer(i)),
                Err(_) => Err(format!("Cannot convert '{}' to integer", s)),
            }
        },
        Value::Boolean(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        _ => Err("Cannot convert to integer".to_string()),
    }
}

fn runtime_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("float() expects exactly one argument".to_string());
    }
    
    match &args[0] {
        Value::Integer(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => {
            match s.parse::<f64>() {
                Ok(f) => Ok(Value::Float(f)),
                Err(_) => Err(format!("Cannot convert '{}' to float", s)),
            }
        },
        Value::Boolean(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        _ => Err("Cannot convert to float".to_string()),
    }
}

    /// Checks if a value matches a pattern, binding variables to the environment if it does
/// Executes a Logos program
pub fn execute_program(program: &Program) -> Result<Value, String> {
    let mut runtime = Runtime::new();
    runtime.eval_program(program)
}