//! Enhanced error handling and debugging module for the Logos programming language
//! Provides comprehensive error reporting, stack traces, and debugging capabilities

use std::fmt;
use std::error::Error as StdError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Enhanced error types for the Logos programming language
#[derive(Error, Debug, Diagnostic)]
pub enum LogosError {
    /// Syntax errors with detailed location information
    #[error("Syntax error: {message}")]
    #[diagnostic(code(logos::syntax_error))]
    SyntaxError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Type errors with detailed information
    #[error("Type error: {message}")]
    #[diagnostic(code(logos::type_error))]
    TypeError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Runtime errors with stack trace
    #[error("Runtime error: {message}")]
    #[diagnostic(code(logos::runtime_error))]
    RuntimeError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
        backtrace: Option<String>,
    },

    /// Memory safety errors
    #[error("Memory safety error: {message}")]
    #[diagnostic(code(logos::memory_error))]
    MemorySafetyError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Foreign function interface errors
    #[error("FFI error: {message}")]
    #[diagnostic(code(logos::ffi_error))]
    FFIError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Multi-language integration errors
    #[error("Multi-language integration error: {message}")]
    #[diagnostic(code(logos::multilang_error))]
    MultiLangError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// Concurrency errors
    #[error("Concurrency error: {message}")]
    #[diagnostic(code(logos::concurrency_error))]
    ConcurrencyError {
        message: String,
        #[label("here")]
        span: Option<SourceSpan>,
    },

    /// File system errors
    #[error("File system error: {message}")]
    #[diagnostic(code(logos::fs_error))]
    FileSystemError {
        message: String,
        source: Option<Arc<dyn StdError + Send + Sync>>,
    },

    /// Network errors
    #[error("Network error: {message}")]
    #[diagnostic(code(logos::network_error))]
    NetworkError {
        message: String,
        source: Option<Arc<dyn StdError + Send + Sync>>,
    },
}

impl LogosError {
    /// Create a new syntax error
    pub fn syntax_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::SyntaxError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new type error
    pub fn type_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::TypeError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new runtime error with optional backtrace
    pub fn runtime_error(message: &str, line: usize, column: usize, length: usize, backtrace: Option<String>) -> Self {
        LogosError::RuntimeError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
            backtrace,
        }
    }

    /// Create a new memory safety error
    pub fn memory_safety_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::MemorySafetyError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new FFI error
    pub fn ffi_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::FFIError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new multi-language integration error
    pub fn multilang_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::MultiLangError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new concurrency error
    pub fn concurrency_error(message: &str, line: usize, column: usize, length: usize) -> Self {
        LogosError::ConcurrencyError {
            message: message.to_string(),
            span: Some(SourceSpan::new((line * 100 + column).into(), length.into())),
        }
    }

    /// Create a new file system error
    pub fn file_system_error(message: &str, source: Option<Arc<dyn StdError + Send + Sync>>) -> Self {
        LogosError::FileSystemError {
            message: message.to_string(),
            source,
        }
    }

    /// Create a new network error
    pub fn network_error(message: &str, source: Option<Arc<dyn StdError + Send + Sync>>) -> Self {
        LogosError::NetworkError {
            message: message.to_string(),
            source,
        }
    }
}

/// Stack frame for error reporting
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Enhanced error reporter with detailed diagnostics
pub struct ErrorReporter {
    /// Stack trace for runtime errors
    stack_trace: Vec<StackFrame>,
    /// Error statistics
    error_stats: Arc<Mutex<HashMap<String, usize>>>,
    /// Whether to show detailed error information
    verbose_errors: bool,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(verbose_errors: bool) -> Self {
        Self {
            stack_trace: Vec::new(),
            error_stats: Arc::new(Mutex::new(HashMap::new())),
            verbose_errors,
        }
    }

    /// Push a frame to the stack trace
    pub fn push_frame(&mut self, function_name: &str, file: &str, line: usize, column: usize) {
        self.stack_trace.push(StackFrame {
            function_name: function_name.to_string(),
            file: file.to_string(),
            line,
            column,
        });
    }

    /// Pop a frame from the stack trace
    pub fn pop_frame(&mut self) {
        self.stack_trace.pop();
    }

    /// Report an error with detailed information
    pub fn report_error(&self, error: &LogosError) {
        eprintln!("❌ Logos Error:");
        eprintln!("   {}", error);
        
        if self.verbose_errors {
            match error {
                LogosError::RuntimeError { backtrace: Some(bt), .. } => {
                    eprintln!("   Backtrace:\n{}", bt);
                }
                LogosError::RuntimeError { backtrace: None, .. } => {
                    eprintln!("   Stack trace:");
                    for (i, frame) in self.stack_trace.iter().rev().enumerate() {
                        eprintln!("     {}: {} ({}:{}:{})", i, frame.function_name, frame.file, frame.line, frame.column);
                    }
                }
                _ => {}
            }
        }
        
        // Update error statistics
        if let Ok(mut stats) = self.error_stats.try_lock() {
            let error_type = format!("{:?}", error);
            *stats.entry(error_type).or_insert(0) += 1;
        }
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> HashMap<String, usize> {
        self.error_stats.lock().unwrap().clone()
    }

    /// Clear error statistics
    pub fn clear_error_stats(&self) {
        self.error_stats.lock().unwrap().clear();
    }
}

/// Debugging context for enhanced debugging capabilities
pub struct DebugContext {
    /// Whether debugging is enabled
    enabled: bool,
    /// Debug level (0-3)
    level: u8,
    /// Variables and their values at debug points
    variables: HashMap<String, String>,
    /// Breakpoints
    breakpoints: HashMap<String, Vec<usize>>, // file -> [line_numbers]
    /// Step mode
    step_mode: bool,
    /// Call stack for debugging
    call_stack: Vec<StackFrame>,
}

impl DebugContext {
    /// Create a new debugging context
    pub fn new(enabled: bool, level: u8) -> Self {
        Self {
            enabled,
            level,
            variables: HashMap::new(),
            breakpoints: HashMap::new(),
            step_mode: false,
            call_stack: Vec::new(),
        }
    }

    /// Enable or disable debugging
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set the debug level
    pub fn set_level(&mut self, level: u8) {
        self.level = level;
    }

    /// Add a breakpoint at the specified file and line
    pub fn add_breakpoint(&mut self, file: &str, line: usize) {
        self.breakpoints
            .entry(file.to_string())
            .or_insert_with(Vec::new)
            .push(line);
    }

    /// Remove a breakpoint at the specified file and line
    pub fn remove_breakpoint(&mut self, file: &str, line: usize) {
        if let Some(lines) = self.breakpoints.get_mut(file) {
            lines.retain(|&l| l != line);
        }
    }

    /// Check if there's a breakpoint at the specified file and line
    pub fn has_breakpoint(&self, file: &str, line: usize) -> bool {
        self.breakpoints
            .get(file)
            .map_or(false, |lines| lines.contains(&line))
    }

    /// Enter step mode
    pub fn enter_step_mode(&mut self) {
        self.step_mode = true;
    }

    /// Exit step mode
    pub fn exit_step_mode(&mut self) {
        self.step_mode = false;
    }

    /// Check if we should break at the current location
    pub fn should_break(&self, file: &str, line: usize) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.has_breakpoint(file, line) || self.step_mode
    }

    /// Add a variable to the debugging context
    pub fn add_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    /// Get the value of a variable
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Print all variables in the current scope
    pub fn print_variables(&self) {
        if !self.enabled || self.level < 2 {
            return;
        }
        
        println!("📖 Variables in current scope:");
        for (name, value) in &self.variables {
            println!("  {}: {}", name, value);
        }
    }

    /// Push a frame to the call stack
    pub fn push_call_frame(&mut self, function_name: &str, file: &str, line: usize, column: usize) {
        self.call_stack.push(StackFrame {
            function_name: function_name.to_string(),
            file: file.to_string(),
            line,
            column,
        });
    }

    /// Pop a frame from the call stack
    pub fn pop_call_frame(&mut self) {
        self.call_stack.pop();
    }

    /// Print the current call stack
    pub fn print_call_stack(&self) {
        if !self.enabled || self.level < 2 {
            return;
        }
        
        println!("📋 Call stack:");
        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            println!("  {}: {} ({}:{}:{})", i, frame.function_name, frame.file, frame.line, frame.column);
        }
    }

    /// Print debugging information at the specified location
    pub fn debug_print(&self, file: &str, line: usize, column: usize, message: &str) {
        if !self.enabled || self.level < 1 {
            return;
        }
        
        println!("🐛 [DEBUG] {}:{}:{} - {}", file, line, column, message);
    }
}

/// Global error reporter instance
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_ERROR_REPORTER: Arc<Mutex<ErrorReporter>> = 
        Arc::new(Mutex::new(ErrorReporter::new(true)));
    
    pub static ref GLOBAL_DEBUG_CONTEXT: Arc<Mutex<DebugContext>> = 
        Arc::new(Mutex::new(DebugContext::new(false, 2)));
}

/// Report an error using the global error reporter
pub fn report_error(error: &LogosError) {
    let reporter = GLOBAL_ERROR_REPORTER.lock().unwrap();
    reporter.report_error(error);
}

/// Add a variable to the global debugging context
pub fn add_debug_variable(name: &str, value: &str) {
    let mut debug_ctx = GLOBAL_DEBUG_CONTEXT.lock().unwrap();
    debug_ctx.add_variable(name, value);
}

/// Print debugging information using the global debugging context
pub fn debug_print(file: &str, line: usize, column: usize, message: &str) {
    let debug_ctx = GLOBAL_DEBUG_CONTEXT.lock().unwrap();
    debug_ctx.debug_print(file, line, column, message);
}

/// Check if we should break at the current location
pub fn should_break(file: &str, line: usize) -> bool {
    let debug_ctx = GLOBAL_DEBUG_CONTEXT.lock().unwrap();
    debug_ctx.should_break(file, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = LogosError::syntax_error("Unexpected token", 10, 5, 3);
        assert!(matches!(error, LogosError::SyntaxError { .. }));
    }

    #[test]
    fn test_error_reporter() {
        let reporter = ErrorReporter::new(true);
        let error = LogosError::type_error("Type mismatch", 5, 10, 4);
        reporter.report_error(&error);
    }

    #[test]
    fn test_debug_context() {
        let mut debug_ctx = DebugContext::new(true, 2);
        debug_ctx.add_variable("x", "42");
        assert_eq!(debug_ctx.get_variable("x"), Some(&"42".to_string()));
    }
}