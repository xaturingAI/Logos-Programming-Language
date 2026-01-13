//! Source-level Debugger for the Logos Programming Language
//! Provides debugging capabilities with breakpoints, stepping, and variable inspection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self};

use crate::ast::*;
use crate::parser::Parser;
use crate::runtime::{Runtime, Value};

/// Represents the current state of the debugger
#[derive(Debug, Clone)]
pub struct DebuggerState {
    /// Current execution line number
    pub current_line: usize,
    /// Current execution column
    pub current_column: usize,
    /// Whether the debugger is running
    pub running: bool,
    /// Whether execution is paused
    pub paused: bool,
    /// Active breakpoints
    pub breakpoints: HashMap<usize, Breakpoint>,
    /// Call stack
    pub call_stack: Vec<CallFrame>,
    /// Local variables in current scope
    pub local_vars: HashMap<String, Value>,
    /// Global variables
    pub global_vars: HashMap<String, Value>,
    /// Current stack frame index
    pub current_frame: usize,
}

/// Represents a breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Line number where the breakpoint is set
    pub line: usize,
    /// Optional condition for the breakpoint
    pub condition: Option<String>,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// Hit count
    pub hit_count: usize,
}

/// Represents a call frame in the stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function name
    pub function_name: String,
    /// File name
    pub file: String,
    /// Line number in the function
    pub line: usize,
    /// Local variables in this frame
    pub local_vars: HashMap<String, Value>,
}

/// Source-level debugger for Logos programs
pub struct SourceDebugger {
    /// Current debugger state
    state: Arc<Mutex<DebuggerState>>,
    /// Source code being debugged
    source: String,
    /// Source lines for reference
    source_lines: Vec<String>,
    /// Runtime instance
    runtime: Runtime,
    /// Whether to show verbose output
    verbose: bool,
}

impl SourceDebugger {
    /// Creates a new source debugger instance
    pub fn new(source: &str, verbose: bool) -> Self {
        let lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
        
        let initial_state = DebuggerState {
            current_line: 0,
            current_column: 0,
            running: false,
            paused: false,
            breakpoints: HashMap::new(),
            call_stack: Vec::new(),
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            current_frame: 0,
        };
        
        Self {
            state: Arc::new(Mutex::new(initial_state)),
            source: source.to_string(),
            source_lines: lines,
            runtime: Runtime::new(),
            verbose,
        }
    }

    /// Starts a debugging session
    pub fn start_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Logos Source Debugger");
        println!("=====================");
        
        // Parse the source code
        let mut parser = Parser::new(&self.source);
        let program = parser.parse_program()
            .map_err(|e| format!("Parse error: {}", e))?;
        
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
            if !self.is_running()? {
                break;
            }
            
            if self.is_paused()? {
                self.print_prompt()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                let command = input.trim();
                if !self.handle_command(command, program)? {
                    break; // Exit command was issued
                }
            } else {
                // Continue execution until next breakpoint
                // For now, we'll just pause immediately - in a real implementation,
                // this would execute the program until hitting a breakpoint
                self.pause()?;
            }
        }
        
        Ok(())
    }

    /// Handles debugger commands
    fn handle_command(&mut self, command: &str, program: &Program) -> Result<bool, Box<dyn std::error::Error>> {
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
            "finish" | "fin" => {
                self.step_out()?;
            },
            "break" | "b" => {
                if parts.len() >= 2 {
                    if let Ok(line_num) = parts[1].parse::<usize>() {
                        self.set_breakpoint(line_num)?;
                        println!("Breakpoint {} set at line {}", self.state.lock().unwrap().breakpoints.len(), line_num);
                    } else {
                        println!("Invalid line number: {}", parts[1]);
                    }
                } else {
                    println!("Usage: break <line_number>");
                }
            },
            "delete" | "del" | "clear" => {
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
                    self.print_variable(var_name)?;
                } else {
                    println!("Usage: print <variable_name>");
                }
            },
            "locals" => {
                self.print_local_vars()?;
            },
            "globals" => {
                self.print_global_vars()?;
            },
            "backtrace" | "bt" | "where" => {
                self.print_backtrace()?;
            },
            "frame" => {
                if parts.len() >= 2 {
                    if let Ok(frame_idx) = parts[1].parse::<usize>() {
                        self.switch_frame(frame_idx)?;
                    } else {
                        println!("Invalid frame number: {}", parts[1]);
                    }
                } else {
                    println!("Current frame: {}", self.state.lock().unwrap().current_frame);
                }
            },
            "up" => {
                self.move_up_frame()?;
            },
            "down" => {
                self.move_down_frame()?;
            },
            "quit" | "q" | "exit" => {
                return Ok(false); // Exit the debugger
            },
            "info" => {
                if parts.len() >= 2 {
                    match parts[1] {
                        "locals" => self.print_local_vars()?,
                        "globals" => self.print_global_vars()?,
                        "breakpoints" | "bp" => self.list_breakpoints()?,
                        "stack" | "frames" => self.print_backtrace()?,
                        _ => println!("Unknown info topic: {}", parts[1]),
                    }
                } else {
                    println!("Usage: info [locals|globals|breakpoints|stack]");
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
        println!("  help, h, ?              - Show this help");
        println!("  continue, c, run, r       - Continue execution");
        println!("  step, s                   - Step into next statement");
        println!("  next, n                   - Step over next statement");
        println!("  finish, fin               - Step out of current function");
        println!("  break, b <line>           - Set breakpoint at line");
        println!("  delete, del <line>        - Remove breakpoint at line");
        println!("  list, l                   - List all breakpoints");
        println!("  print, p <var>            - Print variable value");
        println!("  locals                    - Print local variables");
        println!("  globals                   - Print global variables");
        println!("  backtrace, bt, where      - Print call stack");
        println!("  frame [<n>]               - Show or switch to frame n");
        println!("  up                        - Move up one frame in stack");
        println!("  down                      - Move down one frame in stack");
        println!("  info [locals|globals|breakpoints|stack] - Show specific info");
        println!("  quit, q, exit             - Exit debugger");
        println!();
        Ok(())
    }

    /// Checks if the debugger is running
    fn is_running(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.running)
    }

    /// Checks if execution is paused
    fn is_paused(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.paused)
    }

    /// Pauses execution
    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.paused = true;
        if self.verbose {
            println!("Execution paused at line {}", state.current_line);
        }
        Ok(())
    }

    /// Continues execution until next breakpoint
    fn continue_execution(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.paused = false;
        if self.verbose {
            println!("Continuing execution...");
        }
        Ok(())
    }

    /// Steps into the next statement
    fn step_into(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would execute one statement and pause
        // For now, we'll just move to the next line and pause
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.current_line += 1;
        state.paused = true;
        if self.verbose {
            println!("Stepped into line {}", state.current_line);
        }
        Ok(())
    }

    /// Steps over the next statement (doesn't enter function calls)
    fn step_over(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would execute the next statement without entering function calls
        // For now, we'll just move to the next line and pause
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.current_line += 1;
        state.paused = true;
        if self.verbose {
            println!("Stepped over to line {}", state.current_line);
        }
        Ok(())
    }

    /// Steps out of the current function
    fn step_out(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would continue until returning from the current function
        // For now, we'll just pause
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.paused = true;
        if self.verbose {
            println!("Stepped out of current function");
        }
        Ok(())
    }

    /// Sets a breakpoint at the specified line
    fn set_breakpoint(&mut self, line: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
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
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.breakpoints.remove(&line);
        Ok(())
    }

    /// Lists all breakpoints
    fn list_breakpoints(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
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
        let state = self.state.lock().map_err(|e| e.to_string())?;
        
        // Check local variables first
        if let Some(value) = state.local_vars.get(var_name) {
            println!("{} = {}", var_name, self.format_value(value));
            return Ok(());
        }

        // Check global variables
        if let Some(value) = state.global_vars.get(var_name) {
            println!("{} = {}", var_name, self.format_value(value));
            return Ok(());
        }
        
        println!("Variable '{}' not found in current scope", var_name);
        Ok(())
    }

    /// Prints local variables
    fn print_local_vars(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        if state.local_vars.is_empty() {
            println!("No local variables in current scope");
        } else {
            println!("Local variables:");
            for (name, value) in &state.local_vars {
                println!("  {}: {}", name, self.format_value(value));
            }
        }
        Ok(())
    }

    /// Prints global variables
    fn print_global_vars(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        if state.global_vars.is_empty() {
            println!("No global variables");
        } else {
            println!("Global variables:");
            for (name, value) in &state.global_vars {
                println!("  {}: {}", name, self.format_value(value));
            }
        }
        Ok(())
    }

    /// Prints the call stack
    fn print_backtrace(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        if state.call_stack.is_empty() {
            println!("Call stack is empty");
        } else {
            println!("Call stack:");
            for (i, frame) in state.call_stack.iter().enumerate() {
                let marker = if i == state.current_frame { " -> " } else { "    " };
                println!("{} {}: {} at {}:{}", marker, i, frame.function_name, frame.file, frame.line);
            }
        }
        Ok(())
    }

    /// Switches to a different stack frame
    fn switch_frame(&mut self, frame_idx: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        if frame_idx < state.call_stack.len() {
            state.current_frame = frame_idx;
            let frame = &state.call_stack[frame_idx];
            println!("Switched to frame {}: {} at {}:{}", frame_idx, frame.function_name, frame.file, frame.line);
        } else {
            println!("Invalid frame number: {}", frame_idx);
        }
        Ok(())
    }

    /// Moves up one frame in the stack
    fn move_up_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        if state.current_frame < state.call_stack.len().saturating_sub(1) {
            state.current_frame += 1;
            let frame = &state.call_stack[state.current_frame];
            println!("Moved up to frame {}: {} at {}:{}", state.current_frame, frame.function_name, frame.file, frame.line);
        } else {
            println!("Already at the topmost frame");
        }
        Ok(())
    }

    /// Moves down one frame in the stack
    fn move_down_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        if state.current_frame > 0 {
            state.current_frame -= 1;
            let frame = &state.call_stack[state.current_frame];
            println!("Moved down to frame {}: {} at {}:{}", state.current_frame, frame.function_name, frame.file, frame.line);
        } else {
            println!("Already at the bottommost frame");
        }
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
            Value::Struct(fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value(v)))
                    .collect();
                format!("{{{}}}", field_strs.join(", "))
            },
        }
    }

    /// Prints the debugger prompt
    fn print_prompt(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        print!("(logosdbg:{}) ", state.current_line);
        io::stdout().flush()?;
        Ok(())
    }
}

/// Debugging session manager
pub struct DebugSession {
    /// Debugger instance
    debugger: Arc<Mutex<SourceDebugger>>,
    /// Whether the session is active
    active: bool,
}

impl DebugSession {
    /// Creates a new debugging session
    pub fn new(source: &str, verbose: bool) -> Self {
        let debugger = Arc::new(Mutex::new(SourceDebugger::new(source, verbose)));
        DebugSession {
            debugger,
            active: false,
        }
    }

    /// Starts the debugging session
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.active = true;
        let mut dbg = self.debugger.lock().map_err(|e| e.to_string())?;
        dbg.start_session()
    }

    /// Stops the debugging session
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.active = false;
        let mut state = self.debugger.lock().map_err(|e| e.to_string())?.state.lock().map_err(|e| e.to_string())?;
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
        let debugger = SourceDebugger::new("print(\"Hello, World!\")", false);
        assert_eq!(debugger.source_lines.len(), 1);
    }

    #[test]
    fn test_breakpoint_operations() {
        let mut debugger = SourceDebugger::new("print(\"test\")", false);
        
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