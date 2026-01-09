use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
    Array(Vec<Value>),
    Tuple(Vec<Value>),  // For tuple values
    Function(String, Vec<Parameter>, Vec<Statement>), // Function name, params, body
    // Add more value types as needed
}

use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            },
            Value::Tuple(tuple_vals) => {
                let elements: Vec<String> = tuple_vals.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", elements.join(", "))
            },
            Value::Function(name, _, _) => write!(f, "<function {}>", name),
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
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            _ => false,
        }
    }
}

pub struct Runtime {
    variables: HashMap<String, Value>,
    functions: HashMap<String, FunctionDef>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    
    pub fn execute_program(&mut self, program: Program) -> Result<(), String> {
        // First, collect all function definitions
        for stmt in &program.statements {
            if let Statement::Function(func) = stmt {
                self.functions.insert(func.name.clone(), func.clone());
            }
        }
        
        // Then execute statements that aren't function definitions
        for stmt in program.statements {
            self.execute_statement(stmt)?;
        }
        
        Ok(())
    }
    
    fn execute_statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(())
            },
            Statement::LetBinding { mutable, name, type_annotation: _, value } => {
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name, val);
                Ok(())
            },
            Statement::ConstBinding { name, type_annotation: _, value } => {
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name, val);
                Ok(())
            },
            Statement::Function(func) => {
                // Function already stored during program initialization
                Ok(())
            },
            Statement::Return(expr) => {
                if let Some(return_expr) = expr {
                    let val = self.evaluate_expression(return_expr)?;
                    // In a real implementation, this would return the value
                    println!("Return value: {:?}", val);
                }
                Ok(())
            },
            Statement::Block(statements) => {
                // Execute each statement in the block
                for stmt in statements {
                    self.execute_statement(stmt)?;
                }
                Ok(())
            },
            _ => {
                // Other statement types not fully implemented in this example
                Ok(())
            }
        }
    }
    
    fn evaluate_expression(&mut self, expr: Expression) -> Result<Value, String> {
        match expr {
            Expression::Integer(value) => Ok(Value::Integer(value)),
            Expression::Float(value) => Ok(Value::Float(value)),
            Expression::String(value) => Ok(Value::String(value)),
            Expression::Boolean(value) => Ok(Value::Boolean(value)),
            Expression::Nil => Ok(Value::Unit), // Treat nil as unit for now
            Expression::Identifier(name) => {
                if let Some(val) = self.variables.get(&name) {
                    Ok(val.clone())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate_expression(*left)?;
                let right_val = self.evaluate_expression(*right)?;
                
                match op {
                    BinaryOp::Add => self.add_values(left_val, right_val),
                    BinaryOp::Sub => self.sub_values(left_val, right_val),
                    BinaryOp::Mul => self.mul_values(left_val, right_val),
                    BinaryOp::Div => self.div_values(left_val, right_val),
                    BinaryOp::Mod => self.mod_values(left_val, right_val),
                    BinaryOp::Eq => Ok(Value::Boolean(left_val == right_val)),
                    BinaryOp::Ne => Ok(Value::Boolean(left_val != right_val)),
                    BinaryOp::Lt => self.lt_values(left_val, right_val),
                    BinaryOp::Gt => self.gt_values(left_val, right_val),
                    BinaryOp::Le => self.le_values(left_val, right_val),
                    BinaryOp::Ge => self.ge_values(left_val, right_val),
                    BinaryOp::And => self.and_values(left_val, right_val),
                    BinaryOp::Or => self.or_values(left_val, right_val),
                    BinaryOp::PipeForward => {
                        // For pipeline: left |> right means right(left)
                        // This is simplified - in a real implementation, right would be a function
                        Ok(right_val)
                    },
                    BinaryOp::PipeBackward => {
                        // For pipeline: left <| right means left(right)
                        // This is simplified - in a real implementation, left would be a function
                        Ok(left_val)
                    },
                    BinaryOp::Spaceship => self.spaceship_values(left_val, right_val),
                    BinaryOp::Power => self.power_values(left_val, right_val),
                    BinaryOp::Range => self.range_values(left_val, right_val),
                }
            },
            Expression::UnaryOp(op, operand) => {
                let operand_val = self.evaluate_expression(*operand)?;
                
                match op {
                    UnaryOp::Neg => match operand_val {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err("Unary minus requires numeric operand".to_string()),
                    },
                    UnaryOp::Not => match operand_val {
                        Value::Boolean(b) => Ok(Value::Boolean(!b)),
                        _ => Err("Logical NOT requires boolean operand".to_string()),
                    },
                    _ => Err("Unary operation not implemented".to_string()),
                }
            },
            Expression::Call(name, args) => {
                if name == "print" {
                    // Built-in print function
                    if args.len() == 1 {
                        let arg_val = self.evaluate_expression(args[0].clone())?;
                        match arg_val {
                            Value::String(s) => println!("{}", s),
                            Value::Integer(n) => println!("{}", n),
                            Value::Float(f) => println!("{}", f),
                            Value::Boolean(b) => println!("{}", b),
                            Value::Unit => println!("()"),
                            _ => println!("{:?}", arg_val),
                        }
                        Ok(Value::Unit)
                    } else {
                        Err("print function expects 1 argument".to_string())
                    }
                } else if name == "len" {
                    // Built-in len function
                    if args.len() == 1 {
                        let arg_val = self.evaluate_expression(args[0].clone())?;
                        match arg_val {
                            Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                            _ => Err("len function expects string or array".to_string()),
                        }
                    } else {
                        Err("len function expects 1 argument".to_string())
                    }
                } else {
                    // User-defined function call
                    let func_opt = self.functions.get(&name).cloned();
                    if let Some(func) = func_opt {
                        // For simplicity, we'll just execute the function body
                        // In a real implementation, we'd handle parameter passing
                        for stmt in func.body {
                            self.execute_statement(stmt)?;
                        }
                        Ok(Value::Unit)
                    } else {
                        Err(format!("Undefined function: {}", name))
                    }
                }
            },
            Expression::MethodCall(obj, method, args) => {
                // For now, just evaluate the object and arguments
                self.evaluate_expression(*obj)?;
                for arg in args {
                    self.evaluate_expression(arg)?;
                }
                // In a real implementation, this would call the method
                Ok(Value::Unit)
            },
            Expression::FieldAccess(obj, field) => {
                // For now, just evaluate the object
                self.evaluate_expression(*obj)?;
                // In a real implementation, this would access the field
                Ok(Value::Unit)
            },
            Expression::If(condition, then_branch, else_branch) => {
                let cond_val = self.evaluate_expression(*condition)?;
                
                if let Value::Boolean(true) = cond_val {
                    for stmt in then_branch {
                        self.execute_statement(stmt)?;
                    }
                } else {
                    for stmt in else_branch {
                        self.execute_statement(stmt)?;
                    }
                }
                
                Ok(Value::Unit)
            },
            Expression::Match(expr, arms) => {
                // For now, just evaluate the expression
                self.evaluate_expression(*expr)?;

                // In a real implementation, this would match against patterns
                for (_, _, body) in arms {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }

                Ok(Value::Unit)
            },
            Expression::Lambda(params, body) => {
                // In a real implementation, this would create a closure
                // For now, just return a unit value
                Ok(Value::Unit)
            },
            Expression::BlockExpr(statements) => {
                // Execute each statement in the block and return the last value
                let mut result = Value::Unit;
                for stmt in statements {
                    match stmt {
                        Statement::Expression(expr) => {
                            result = self.evaluate_expression(expr)?;
                        },
                        _ => {
                            self.execute_statement(stmt)?;
                        }
                    }
                }
                Ok(result)
            },
            Expression::MultiLangCall(lang, code) => {
                // For multi-language calls, print a message
                // In a real implementation, this would call the foreign language
                println!("Calling {} code: {}", lang, code);
                Ok(Value::Unit)
            },
            Expression::MultiLangImport(lang, module, alias) => {
                // For multi-language imports, simulate importing functionality
                // In a real implementation, this would interface with the language's package manager
                println!("Importing {} module in {} language", module, lang);
                // Return a module-like value
                Ok(Value::String(format!("Imported_{}_{}", lang, module.replace(".", "_"))))
            },
            Expression::MultiLangIndex(lang, resource) => {
                // For indexing resources from other languages
                // In a real implementation, this would index the resource for cross-language access
                println!("Indexing {} resource in {} language", resource, lang);
                // Return an indexed resource value
                let clean_resource = resource.replace('.', "_").replace('/', "_").replace('\\', "_").replace('-', "_");
                Ok(Value::String(format!("Indexed_{}_{}", lang, clean_resource)))
            },
            Expression::InterpolatedString(parts) => {
                // Evaluate interpolated string parts and concatenate them
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(text) => result.push_str(&text),
                        StringPart::Interpolated(expr) => {
                            let value = self.evaluate_expression(*expr)?;
                            // Convert the value to string and append
                            result.push_str(&format!("{}", value)); // Using Display trait
                        }
                    }
                }
                Ok(Value::String(result))
            },
            Expression::LambdaSimple(params, body) => {
                // For simple lambda expressions |args| expr
                // In a real implementation, this would create a closure
                // For now, we'll return a function value
                Ok(Value::String("SimpleLambda".to_string()))
            },
            Expression::Pipeline(start_expr, funcs) => {
                // For pipeline expressions: start |> func1 |> func2
                // Evaluate the start expression
                let mut current_value = self.evaluate_expression(*start_expr)?;

                // Apply each function in the pipeline
                for func in funcs {
                    // In a real implementation, we'd call the function with the current value
                    // For now, we'll just evaluate the function (simplified)
                    current_value = self.evaluate_expression(func)?;
                }

                Ok(current_value)
            },
            Expression::BackPipeline(end_expr, funcs) => {
                // For backward pipeline expressions: funcN <| ... <| func1 <| start
                // This is equivalent to funcN(...(func1(start))...)
                // Evaluate the starting expression
                let mut current_value = self.evaluate_expression(*end_expr)?;

                // Apply each function in reverse order
                for func in funcs {
                    // In a real implementation, we'd call the function with the current value
                    // For now, we'll just evaluate the function (simplified)
                    current_value = self.evaluate_expression(func)?;
                }

                Ok(current_value)
            },
            Expression::DestructureAssignment(pattern, expr, stmt) => {
                // For destructure assignment: let pat = expr in stmt
                // Evaluate the expression to destructure
                let value = self.evaluate_expression(*expr)?;

                // In a real implementation, we'd bind pattern variables and evaluate the statement
                // For now, we'll just evaluate the statement in the current environment
                self.execute_statement(*stmt)?;

                Ok(Value::Unit)
            },
            Expression::Tuple(elements) => {
                // For tuples, evaluate all elements and return a tuple value
                let mut values = Vec::new();
                for expr in elements {
                    values.push(self.evaluate_expression(expr)?);
                }
                Ok(Value::Tuple(values))
            },
            Expression::Spawn(actor_name, args) => {
                // Evaluate the arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }

                // In a real implementation, this would create a new thread/process for the actor
                // For now, we'll simulate by storing the actor info
                // Create an actor handle value
                Ok(Value::String(format!("ActorHandle_{}", actor_name)))
            },
            Expression::Send(actor_expr, message_expr) => {
                // Evaluate the actor and message
                let actor_val = self.evaluate_expression(*actor_expr)?;
                let message_val = self.evaluate_expression(*message_expr)?;

                // In a real implementation, this would send the message to the actor's mailbox
                // For now, we'll just return Unit
                Ok(Value::Unit)
            },
            Expression::Receive => {
                // In a real implementation, this would receive a message from the actor's mailbox
                // For now, we'll return a dummy value
                Ok(Value::Unit)
            },
            Expression::ChannelCreate(channel_type) => {
                // Create a channel - in a real implementation, this would create an actual channel
                // For now, we'll return a placeholder value
                Ok(Value::String(format!("Channel<{}>", "unknown"))) // Placeholder
            },
            Expression::ChannelSend(channel_expr, value_expr) => {
                // Evaluate the channel and value
                let _channel_val = self.evaluate_expression(*channel_expr)?;
                let _value_val = self.evaluate_expression(*value_expr)?;

                // In a real implementation, this would send the value to the channel
                // For now, we'll just return Unit
                Ok(Value::Unit)
            },
            Expression::ChannelReceive(channel_expr) => {
                // Evaluate the channel
                let _channel_val = self.evaluate_expression(*channel_expr)?;

                // In a real implementation, this would receive a value from the channel
                // For now, we'll return a placeholder value
                Ok(Value::Unit) // Placeholder
            },
            Expression::ChannelClose(channel_expr) => {
                // Evaluate the channel
                let _channel_val = self.evaluate_expression(*channel_expr)?;

                // In a real implementation, this would close the channel
                // For now, we'll just return Unit
                Ok(Value::Unit)
            },
            Expression::Select(select_arms) => {
                // In a real implementation, this would perform a select operation on multiple channels
                // For now, we'll just return Unit
                Ok(Value::Unit)
            },
            Expression::AsyncBlock(_) => {
                // For async blocks, return a placeholder future value
                // In a real implementation, this would create a future/promise
                Ok(Value::String("Future".to_string()))
            },
            Expression::Await(expr) => {
                // For await expressions, evaluate the expression and return its result
                // In a real implementation, this would await the future
                self.evaluate_expression(*expr)
            },
            Expression::Future(expr) => {
                // For future expressions, return a placeholder future value
                // In a real implementation, this would wrap the expression in a future
                self.evaluate_expression(*expr) // For now, just evaluate directly
            },
            Expression::SpawnTask(expr) => {
                // For spawning tasks, evaluate the expression and return a task handle
                // In a real implementation, this would spawn a concurrent task
                let result = self.evaluate_expression(*expr)?;
                Ok(Value::String("TaskHandle".to_string())) // Return a task handle
            },
            Expression::Join(expr) => {
                // For joining tasks, evaluate the task expression and wait for completion
                // In a real implementation, this would join with a spawned task
                self.evaluate_expression(*expr)
            },
            Expression::Race(expressions) => {
                // For racing futures, evaluate all expressions and return the first to complete
                // In a real implementation, this would race the futures
                if let Some(first_expr) = expressions.first() {
                    self.evaluate_expression(first_expr.clone())
                } else {
                    Ok(Value::Unit) // Empty race returns Unit
                }
            },
            Expression::Timeout(expr, duration) => {
                // For timeout expressions, evaluate the expression with a timeout
                // In a real implementation, this would implement timeout logic
                self.evaluate_expression(*expr) // For now, just evaluate without timeout
            },
        }
    }
    
    fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            },
            _ => Err("Addition operation requires compatible types".to_string()),
        }
    }
    
    fn sub_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err("Subtraction operation requires numeric types".to_string()),
        }
    }
    
    fn mul_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err("Multiplication operation requires numeric types".to_string()),
        }
    }
    
    fn div_values(&self, left: Value, right: Value) -> Result<Value, String> {
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
            _ => Err("Division operation requires numeric types".to_string()),
        }
    }
    
    fn mod_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Integer(a % b))
                }
            },
            _ => Err("Modulo operation requires integer types".to_string()),
        }
    }
    
    fn lt_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
            _ => Err("Comparison operation requires comparable types".to_string()),
        }
    }
    
    fn gt_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
            _ => Err("Comparison operation requires comparable types".to_string()),
        }
    }
    
    fn le_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err("Comparison operation requires comparable types".to_string()),
        }
    }
    
    fn ge_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err("Comparison operation requires comparable types".to_string()),
        }
    }
    
    fn and_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
            _ => Err("Logical AND requires boolean types".to_string()),
        }
    }
    
    fn or_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
            _ => Err("Logical OR requires boolean types".to_string()),
        }
    }
    
    fn spaceship_values(&self, left: Value, right: Value) -> Result<Value, String> {
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
            _ => Err("Spaceship operator requires comparable types".to_string()),
        }
    }

    fn power_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b >= 0 {
                    Ok(Value::Integer(a.pow(b as u32)))
                } else {
                    // For negative exponents, return float
                    Ok(Value::Float((a as f64).powi(b as i32)))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                Ok(Value::Float(a.powf(b)))
            },
            (Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Float(a.powi(b as i32)))
            },
            (Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Float((a as f64).powf(b)))
            },
            _ => Err("Power operation requires numeric types".to_string()),
        }
    }

    fn range_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                // For now, just return a simple representation of the range
                // In a full implementation, this would create a range object
                Ok(Value::Array((a..=b).map(|x| Value::Integer(x)).collect()))
            },
            _ => Err("Range operation requires integer operands".to_string()),
        }
    }
}

pub fn execute_program(program: Program) -> Result<(), String> {
    let mut runtime = Runtime::new();
    runtime.execute_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_runtime_simple_program() {
        let input = r#"
        fn main() {
            let x = 42
            let y = x + 8
            print(y)
        }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let result = execute_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_arithmetic() {
        let input = r#"
        fn main() {
            let a = 10
            let b = 5
            let sum = a + b
            let diff = a - b
            let prod = a * b
            let quot = a / b
            print(sum)
            print(diff)
            print(prod)
            print(quot)
        }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let result = execute_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_conditionals() {
        let input = r#"
        fn main() {
            let x = 10
            if x > 5 {
                print("x is greater than 5")
            } else {
                print("x is not greater than 5")
            }
        }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let result = execute_program(program);
        assert!(result.is_ok());
    }
}