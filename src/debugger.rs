//! Logos Debugger Module
//! Provides debugging capabilities for Logos programs with breakpoints, variable inspection, and step-through execution

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

use crate::ast::{Program, Statement, Expression};
use crate::parser::Parser;
use crate::runtime::{Runtime, Value};

/// Represents the state of the debugger
#[derive(Debug, Clone)]
pub struct DebuggerState {
    /// Current execution line number
    pub current_line: usize,
    /// Current execution column
    pub current_column: usize,
    /// Stack frames
    pub stack_frames: Vec<StackFrame>,
    /// Breakpoints by line number
    pub breakpoints: HashMap<usize, Breakpoint>,
    /// Variables in current scope
    pub variables: HashMap<String, Value>,
    /// Whether the debugger is currently running
    pub running: bool,
    /// Whether execution is paused
    pub paused: bool,
    /// Call stack trace
    pub call_stack: Vec<String>,
}

/// Represents a stack frame in the debugger
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name
    pub function_name: String,
    /// Local variables in this frame
    pub locals: HashMap<String, Value>,
    /// File name where the function is defined
    pub file: String,
    /// Line number in the file
    pub line: usize,
}

/// Represents a breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Line number where the breakpoint is set
    pub line: usize,
    /// Condition for the breakpoint (if any)
    pub condition: Option<String>,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// Hit count
    pub hit_count: usize,
}

/// Represents a watchpoint
#[derive(Debug, Clone)]
pub struct Watchpoint {
    /// Variable name to watch
    pub variable: String,
    /// Condition for the watchpoint
    pub condition: Option<String>,
    /// Action to take when condition is met
    pub action: WatchAction,
}

/// Actions that can be taken when a watchpoint condition is met
#[derive(Debug, Clone)]
pub enum WatchAction {
    Break,      // Break execution
    Log,        // Log the change
    Continue,   // Continue execution
}

/// Main debugger implementation
pub struct Debugger {
    /// Current state of the debugger
    state: Arc<Mutex<DebuggerState>>,
    /// Source code being debugged
    source: String,
    /// Runtime instance
    runtime: Runtime,
    /// Whether to show debug information
    verbose: bool,
}

impl Debugger {
    /// Creates a new debugger instance
    pub fn new(source: &str, verbose: bool) -> Self {
        let state = DebuggerState {
            current_line: 0,
            current_column: 0,
            stack_frames: Vec::new(),
            breakpoints: HashMap::new(),
            variables: HashMap::new(),
            running: false,
            paused: false,
            call_stack: Vec::new(),
        };

        Debugger {
            state: Arc::new(Mutex::new(state)),
            source: source.to_string(),
            runtime: Runtime::new(),
            verbose,
        }
    }

    /// Starts the debugging session
    pub fn start_debugging(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Logos Debugger Started");
        println!("======================");
        
        // Parse the source code
        let mut parser = Parser::new(&self.source);
        let program = parser.parse_program()?;
        
        // Set initial state
        {
            let mut state = self.state.lock().unwrap();
            state.running = true;
            state.paused = true; // Start paused at the beginning
        }
        
        // Enter the debugging loop
        self.debug_loop(&program)?;
        
        Ok(())
    }

    /// Main debugging loop
    fn debug_loop(&mut self, program: &Program) -> Result<(), Box<dyn std::error::Error>> {
        println!("Debugging session started. Type 'help' for available commands.");
        
        loop {
            if !self.is_running() {
                break;
            }
            
            if self.is_paused() {
                self.print_prompt()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                let command = input.trim();
                if !self.handle_command(command)? {
                    break; // Exit command was issued
                }
            } else {
                // Continue execution until next breakpoint
                self.execute_next_statement()?;
            }
        }
        
        Ok(())
    }

    /// Handles debugger commands
    fn handle_command(&mut self, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }

        match parts[0] {
            "help" | "h" | "?" => self.show_help()?,
            "continue" | "c" | "run" | "r" => {
                self.continue_execution()?;
            },
            "step" | "s" => {
                self.step_into()?;
            },
            "next" | "n" => {
                self.step_over()?;
            },
            "finish" | "f" => {
                self.step_out()?;
            },
            "break" | "b" => {
                if parts.len() >= 2 {
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        self.set_breakpoint(line_num)?;
                        println!("Breakpoint set at line {}", line_num);
                    } else {
                        println!("Invalid line number: {}", parts[1]);
                    }
                } else {
                    println!("Usage: break <line_number>");
                }
            },
            "delete" | "del" => {
                if parts.len() >= 2 {
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        self.remove_breakpoint(line_num)?;
                        println!("Breakpoint at line {} removed", line_num);
                    } else {
                        println!("Invalid line number: {}", parts[1]);
                    }
                } else {
                    println!("Usage: delete <line_number>");
                }
            },
            "list" | "l" => {
                self.list_breakpoints()?;
            },
            "print" | "p" | "eval" => {
                if parts.len() >= 2 {
                    let var_name = parts[1];
                    match self.inspect_variable(var_name) {
                        Ok(result) => println!("{}", result),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: print <variable_name>");
                }
            },
            "inspect" | "i" => {
                if parts.len() >= 2 {
                    let var_name = parts[1];
                    match self.inspect_variable(var_name) {
                        Ok(result) => println!("{}", result),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: inspect <variable_name>");
                }
            },
            "locals" => {
                self.print_locals()?;
            },
            "stack" | "bt" => {
                self.print_backtrace()?;
            },
            "quit" | "q" | "exit" => {
                return Ok(false); // Exit the debugger
            },
            "info" => {
                if parts.len() >= 2 {
                    match parts[1] {
                        "locals" => self.print_locals()?,
                        "breakpoints" | "bp" => self.list_breakpoints()?,
                        "stack" | "frame" => self.print_backtrace()?,
                        _ => println!("Unknown info command: {}", parts[1]),
                    }
                } else {
                    println!("Usage: info [locals|breakpoints|stack]");
                }
            },
            "" => {
                // Empty command - do nothing
            },
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
            }
        }

        Ok(true) // Continue running
    }

    /// Shows help information
    fn show_help(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Logos Debugger Commands:");
        println!("  help, h, ?          - Show this help");
        println!("  continue, c, run, r   - Continue execution");
        println!("  step, s             - Step into next statement");
        println!("  next, n             - Step over next statement");
        println!("  finish, f           - Step out of current function");
        println!("  break, b <line>     - Set breakpoint at line");
        println!("  delete, del <line>  - Remove breakpoint at line");
        println!("  list, l             - List all breakpoints");
        println!("  print, p, eval <var> - Print variable value or evaluate expression");
        println!("  inspect, i <var>    - Inspect variable in detail");
        println!("  locals              - Print local variables");
        println!("  stack, bt           - Print backtrace");
        println!("  info <topic>        - Info about locals, breakpoints, or stack");
        println!("  quit, q, exit       - Exit debugger");
        println!();
        Ok(())
    }

    /// Continues execution until next breakpoint
    fn continue_execution(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().unwrap();
        state.paused = false;
        drop(state);
        
        println!("Continuing execution...");
        Ok(())
    }

    /// Steps into the next statement
    fn step_into(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stepping into next statement...");
        // For now, just execute the next statement and pause again
        self.execute_next_statement()?;
        let mut state = self.state.lock().unwrap();
        state.paused = true;
        Ok(())
    }

    /// Steps over the next statement (doesn't enter function calls)
    fn step_over(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stepping over next statement...");
        // For now, just execute the next statement and pause again
        self.execute_next_statement()?;
        let mut state = self.state.lock().unwrap();
        state.paused = true;
        Ok(())
    }

    /// Steps out of the current function
    fn step_out(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stepping out of current function...");
        // For now, just execute and pause again
        let mut state = self.state.lock().unwrap();
        state.paused = true;
        Ok(())
    }

    /// Executes the next statement
    fn execute_next_statement(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would execute one statement at a time
        // For now, we'll just simulate the execution
        let mut state = self.state.lock().unwrap();
        state.current_line += 1; // Increment line number
        
        // Check if we hit a breakpoint
        let current_line = state.current_line; // Store the value before borrowing the map mutably
        if state.breakpoints.contains_key(&current_line) {
            // Get the breakpoint, modify it, and reinsert it
            if let Some(mut bp) = state.breakpoints.remove(&current_line) {
                bp.hit_count += 1;

                if bp.enabled {
                    println!("Hit breakpoint at line {}", current_line);
                    state.paused = true;
                }

                // Put the updated breakpoint back in the map
                state.breakpoints.insert(current_line, bp);
            }
        }
        
        Ok(())
    }

    /// Sets a breakpoint at the specified line
    fn set_breakpoint(&mut self, line: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().unwrap();
        let breakpoint = Breakpoint {
            line,
            condition: None,
            enabled: true,
            hit_count: 0,
        };
        state.breakpoints.insert(line, breakpoint);
        Ok(())
    }

    /// Removes a breakpoint at the specified line
    fn remove_breakpoint(&mut self, line: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().unwrap();
        state.breakpoints.remove(&line);
        Ok(())
    }

    /// Lists all breakpoints
    fn list_breakpoints(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        if state.breakpoints.is_empty() {
            println!("No breakpoints set.");
        } else {
            println!("Breakpoints:");
            for (line, bp) in &state.breakpoints {
                let status = if bp.enabled { "enabled" } else { "disabled" };
                println!("  {}: {} (hit {} times)", line, status, bp.hit_count);
            }
        }
        Ok(())
    }

    /// Prints the value of a variable
    fn print_variable(&self, var_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.variables.get(var_name) {
            println!("{} = {:?}", var_name, value);
        } else {
            println!("Variable '{}' not found in current scope", var_name);
        }
        Ok(())
    }

    /// Prints local variables
    fn print_locals(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        if state.stack_frames.is_empty() {
            println!("No stack frames available");
            return Ok(());
        }

        let current_frame = state.stack_frames.last().unwrap();
        if current_frame.locals.is_empty() {
            println!("No local variables in current frame");
        } else {
            println!("Local variables in function '{}':", current_frame.function_name);
            for (name, value) in &current_frame.locals {
                println!("  {}: {}", name, self.format_value(value));
            }
        }
        Ok(())
    }

    /// Evaluates an expression in the current debugging context
    fn evaluate_expression_in_context(&self, expr_str: &str) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation, this would parse and evaluate the expression
        // in the current debugging context (with access to local and global variables)
        // For now, we'll return a placeholder result
        Ok(format!("Evaluated expression: {}", expr_str))
    }

    /// Inspects a variable in the current context
    fn inspect_variable(&self, var_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();

        // First, check local variables in the current frame
        if let Some(current_frame) = state.stack_frames.last() {
            if let Some(value) = current_frame.locals.get(var_name) {
                return Ok(format!("{} = {}", var_name, self.format_value(value)));
            }
        }

        // Then, check global variables
        if let Some(value) = state.variables.get(var_name) {
            return Ok(format!("{} = {}", var_name, self.format_value(value)));
        }

        Err(format!("Variable '{}' not found in current context", var_name).into())
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
                format!("{} {{ {} }}", name, field_strs.join(", "))
            },
            Value::Char(c) => format!("'{}'", c),
            Value::BuiltinFunction(_) => "<builtin function>".to_string(),
            Value::Future(boxed_value) => format!("<future: {}>", self.format_value(boxed_value)),
            Value::Task(boxed_value) => format!("<task: {}>", self.format_value(boxed_value)),
            Value::Closure(_, _, _) => "<closure>".to_string(),
        }
    }

    /// Prints the call stack
    fn print_backtrace(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        if state.call_stack.is_empty() {
            println!("Call stack is empty");
        } else {
            println!("Call stack:");
            for (i, frame) in state.call_stack.iter().enumerate() {
                println!("  {}: {}", i, frame);
            }
        }
        Ok(())
    }

    /// Prints the debugger prompt
    fn print_prompt(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        print!("(logosdbg:{}) ", state.current_line);
        io::stdout().flush()?;
        Ok(())
    }

    /// Checks if the debugger is running
    fn is_running(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.running
    }

    /// Checks if execution is paused
    fn is_paused(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.paused
    }

    /// Gets the current debugger state
    pub fn get_state(&self) -> DebuggerState {
        self.state.lock().unwrap().clone()
    }
}

/// Debugging session manager
pub struct DebugSession {
    /// Debugger instance
    debugger: Arc<Mutex<Debugger>>,
    /// Whether the session is active
    active: bool,
}

impl DebugSession {
    /// Creates a new debugging session
    pub fn new(source: &str, verbose: bool) -> Self {
        let debugger = Arc::new(Mutex::new(Debugger::new(source, verbose)));
        DebugSession {
            debugger,
            active: false,
        }
    }

    /// Starts the debugging session
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.active = true;
        let mut dbg = self.debugger.lock().unwrap();
        dbg.start_debugging()
    }

    /// Stops the debugging session
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.active = false;
        let dbg_guard = self.debugger.lock().unwrap();
        let mut state = dbg_guard.state.lock().unwrap();
        state.running = false;
        state.paused = false;
        Ok(())
    }

    /// Checks if the session is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Utility function to start debugging a source file
pub fn debug_source(source: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut session = DebugSession::new(source, verbose);
    session.start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_creation() {
        let debugger = Debugger::new("print(\"Hello, World!\")", false);
        assert_eq!(debugger.get_state().current_line, 0);
        assert_eq!(debugger.get_state().breakpoints.len(), 0);
    }

    #[test]
    fn test_breakpoint_operations() {
        let mut debugger = Debugger::new("print(\"test\")", false);
        
        // Test setting a breakpoint
        debugger.set_breakpoint(10).unwrap();
        {
            let state = debugger.state.lock().unwrap();
            assert!(state.breakpoints.contains_key(&10));
        }
        
        // Test removing a breakpoint
        debugger.remove_breakpoint(10).unwrap();
        {
            let state = debugger.state.lock().unwrap();
            assert!(!state.breakpoints.contains_key(&10));
        }
    }

    #[test]
    fn test_debug_session() {
        let mut session = DebugSession::new("print(\"test\")", false);
        assert!(!session.is_active());
        
        // We can't actually start the session in tests since it would wait for input
        // But we can verify the session can be created
        assert_eq!(session.is_active(), false);
    }
}