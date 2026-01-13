//! REPL (Read-Eval-Print Loop) for the Logos Programming Language
//! Provides an interactive environment for writing, debugging, and executing Logos code

use std::io::{self, Write};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::*;
use crate::parser::Parser;
use crate::type_checker::TypeChecker;
use crate::runtime::{Runtime, Value};

/// REPL (Read-Eval-Print Loop) for Logos
pub struct Repl {
    /// Runtime environment for executing code
    runtime: Runtime,
    /// Type checker for validating code
    type_checker: TypeChecker,
    /// Variables defined in the REPL session
    variables: Arc<Mutex<HashMap<String, Value>>>,
    /// History of commands entered
    history: Vec<String>,
    /// Whether to show debug information
    verbose: bool,
}

impl Repl {
    /// Creates a new REPL instance
    pub fn new(verbose: bool) -> Self {
        Self {
            runtime: Runtime::new(),
            type_checker: TypeChecker::new(),
            variables: Arc::new(Mutex::new(HashMap::new())),
            history: Vec::new(),
            verbose,
        }
    }

    /// Starts the REPL session
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Welcome to the Logos REPL!");
        println!("Type 'help' for available commands, 'quit' to exit.");
        println!();

        loop {
            print!("logos> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let command = input.trim();
            if command.is_empty() {
                continue;
            }

            // Add command to history
            self.history.push(command.to_string());

            // Process the command
            if !self.process_command(command)? {
                break; // Exit command was issued
            }
        }

        println!("Goodbye!");
        Ok(())
    }

    /// Processes a REPL command
    fn process_command(&mut self, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let command = command.trim();
        
        if command.starts_with(':') {
            // Handle REPL commands
            self.handle_repl_command(command)?;
            return Ok(true);
        }

        // Treat as Logos expression/statement
        self.evaluate_logos_code(command)?;
        Ok(true) // Continue running
    }

    /// Handles REPL-specific commands (starting with :)
    fn handle_repl_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            ":help" | ":h" | ":?" => self.show_help()?,
            ":quit" | ":q" | ":exit" => std::process::exit(0),
            ":clear" | ":cls" => {
                // Clear the screen (works on most terminals)
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            },
            ":history" | ":hist" => {
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, cmd);
                }
            },
            ":vars" | ":variables" => {
                let vars = self.variables.lock().unwrap();
                if vars.is_empty() {
                    println!("No variables defined");
                } else {
                    println!("Defined variables:");
                    for (name, value) in &*vars {
                        println!("  {}: {}", name, self.format_value(value));
                    }
                }
            },
            ":type" => {
                if parts.len() >= 2 {
                    let code = parts[1..].join(" ");
                    self.check_type(&code)?;
                } else {
                    println!("Usage: :type <expression>");
                }
            },
            ":reset" => {
                // Reset the REPL state
                self.runtime = Runtime::new();
                self.type_checker = TypeChecker::new();
                let mut vars = self.variables.lock().unwrap();
                vars.clear();
                println!("REPL state reset");
            },
            ":load" => {
                if parts.len() >= 2 {
                    let filename = parts[1];
                    self.load_file(filename)?;
                } else {
                    println!("Usage: :load <filename>");
                }
            },
            ":eval" => {
                if parts.len() >= 2 {
                    let code = parts[1..].join(" ");
                    self.evaluate_logos_code(&code)?;
                } else {
                    println!("Usage: :eval <code>");
                }
            },
            _ => {
                println!("Unknown command: {}. Type :help for available commands.", parts[0]);
            }
        }

        Ok(())
    }

    /// Evaluates Logos code in the REPL
    fn evaluate_logos_code(&mut self, code: &str) -> Result<(), String> {
        if self.verbose {
            println!("[DEBUG] Evaluating code: {}", code);
        }

        // Parse the code
        let mut parser = Parser::new(code);
        let program = match parser.parse_program() {
            Ok(prog) => prog,
            Err(e) => {
                return Err(format!("Parse error: {}", e));
            }
        };

        // Type check the code
        let mut type_checker = self.type_checker.clone(); // Assuming TypeChecker implements Clone
        if let Err(e) = type_checker.check_program(&program) {
            return Err(format!("Type error: {}", e));
        }

        // Execute the code
        match self.runtime.execute_program(&program) {
            Ok(_) => {
                // For expressions that return values, we might want to print them
                // This would require modifying the runtime to return the last expression's value
            },
            Err(e) => {
                return Err(format!("Runtime error: {}", e));
            },
        }

        Ok(())
    }

    /// Checks the type of an expression
    fn check_type(&self, expr_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        // For now, we'll just try to parse and type check the expression
        // In a full implementation, we'd need to handle expressions differently than statements
        let code = format!("let _temp = {};", expr_str); // Wrap in a statement to make it parseable
        
        let mut parser = Parser::new(&code);
        match parser.parse_program() {
            Ok(program) => {
                let mut type_checker = self.type_checker.clone();
                match type_checker.check_program(&program) {
                    Ok(()) => {
                        // In a full implementation, we'd determine the type of the expression
                        println!("Expression is well-typed");
                    },
                    Err(e) => println!("Type error: {}", e),
                }
            },
            Err(e) => println!("Parse error: {}", e),
        }
        
        Ok(())
    }

    /// Loads code from a file
    fn load_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| format!("Error reading file '{}': {}", filename, e))?;
        
        println!("Loading file: {}", filename);
        self.evaluate_logos_code(&content)?;
        Ok(())
    }

    /// Shows help information
    fn show_help(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Logos REPL Commands:");
        println!("  :help, :h, :?      - Show this help");
        println!("  :quit, :q, :exit   - Exit the REPL");
        println!("  :clear, :cls       - Clear the screen");
        println!("  :history, :hist    - Show command history");
        println!("  :vars, :variables  - Show defined variables");
        println!("  :type <expr>       - Check the type of an expression");
        println!("  :reset             - Reset REPL state");
        println!("  :load <file>       - Load code from a file");
        println!("  :eval <code>       - Evaluate Logos code");
        println!();
        println!("You can also enter Logos expressions/statements directly for evaluation.");
        Ok(())
    }

    /// Formats a value for display
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Boolean(b) => b.to_string(),
            Value::Unit => "()".to_string(),
            Value::Function(name, _, _, _) => format!("<function {}>", name),
            Value::Tuple(items) => {
                let item_strs: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                format!("({})", item_strs.join(", "))
            },
            Value::Array(items) => {
                let item_strs: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", item_strs.join(", "))
            },
            Value::Struct(name, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{} {{{}}}", name, field_strs.join(", "))
            },
            _ => format!("{:?}", value), // Handle all other variants with debug formatting
        }
    }
}

/// Starts the Logos REPL
pub fn start_repl(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new(verbose);
    repl.start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new(false);
        assert_eq!(repl.history.len(), 0);
    }

    #[test]
    fn test_format_value() {
        let repl = Repl::new(false);
        
        assert_eq!(repl.format_value(&Value::Integer(42)), "42");
        assert_eq!(repl.format_value(&Value::Boolean(true)), "true");
        assert_eq!(repl.format_value(&Value::String("hello".to_string())), "\"hello\"");
        assert_eq!(repl.format_value(&Value::Unit), "()");
    }
}