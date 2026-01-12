// Logos Programming Language Standard Library
// This module provides the core standard library functions for the Logos programming language
// including collections, I/O, networking, and other essential utilities.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

/// Core collection types and operations
pub mod collections {
    use std::collections::{HashMap, HashSet, VecDeque};

    /// Creates a new vector with the given elements
    pub fn vec_new<T>(elements: Vec<T>) -> Vec<T> {
        elements
    }

    /// Creates a new hash map with the given key-value pairs
    pub fn map_new<K, V>(pairs: Vec<(K, V)>) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        pairs.into_iter().collect()
    }

    /// Creates a new hash set with the given elements
    pub fn set_new<T>(elements: Vec<T>) -> HashSet<T>
    where
        T: std::hash::Hash + Eq,
    {
        elements.into_iter().collect()
    }

    /// Creates a new queue with the given elements
    pub fn queue_new<T>(elements: Vec<T>) -> VecDeque<T> {
        elements.into()
    }

    /// Map operations
    pub fn map_get<'a, K, V>(map: &'a HashMap<K, V>, key: &K) -> Option<&'a V>
    where
        K: std::hash::Hash + Eq,
    {
        map.get(key)
    }

    pub fn map_set<K, V>(map: &mut HashMap<K, V>, key: K, value: V) -> Option<V>
    where
        K: std::hash::Hash + Eq,
    {
        map.insert(key, value)
    }

    /// Vector operations
    pub fn vec_push<T>(vec: &mut Vec<T>, element: T) {
        vec.push(element);
    }

    pub fn vec_pop<T>(vec: &mut Vec<T>) -> Option<T> {
        vec.pop()
    }

    pub fn vec_len<T>(vec: &Vec<T>) -> usize {
        vec.len()
    }

    pub fn vec_get<T>(vec: &Vec<T>, index: usize) -> Option<&T> {
        vec.get(index)
    }

    pub fn vec_set<T>(vec: &mut Vec<T>, index: usize, value: T) -> Result<(), String> {
        if index < vec.len() {
            vec[index] = value;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    /// Set operations
    pub fn set_add<T>(set: &mut HashSet<T>, element: T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.insert(element)
    }

    pub fn set_contains<T>(set: &HashSet<T>, element: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.contains(element)
    }

    pub fn set_remove<T>(set: &mut HashSet<T>, element: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.remove(element)
    }

    /// Queue operations
    pub fn queue_enqueue<T>(queue: &mut VecDeque<T>, element: T) {
        queue.push_back(element);
    }

    pub fn queue_dequeue<T>(queue: &mut VecDeque<T>) -> Option<T> {
        queue.pop_front()
    }

    pub fn queue_peek<T>(queue: &VecDeque<T>) -> Option<&T> {
        queue.front()
    }
}

/// String manipulation utilities
pub mod string_utils {
    /// Joins a vector of strings with a separator
    pub fn join(strings: Vec<String>, separator: &str) -> String {
        strings.join(separator)
    }
    
    /// Splits a string by a separator
    pub fn split(string: &str, separator: &str) -> Vec<String> {
        string.split(separator).map(|s| s.to_string()).collect()
    }
    
    /// Checks if a string contains a substring
    pub fn contains(haystack: &str, needle: &str) -> bool {
        haystack.contains(needle)
    }
    
    /// Converts a string to uppercase
    pub fn to_uppercase(string: &str) -> String {
        string.to_uppercase()
    }
    
    /// Converts a string to lowercase
    pub fn to_lowercase(string: &str) -> String {
        string.to_lowercase()
    }
    
    /// Trims whitespace from both ends of a string
    pub fn trim(string: &str) -> String {
        string.trim().to_string()
    }
    
    /// Replaces all occurrences of a pattern with a replacement
    pub fn replace(string: &str, pattern: &str, replacement: &str) -> String {
        string.replace(pattern, replacement)
    }
    
    /// Checks if a string starts with a prefix
    pub fn starts_with(string: &str, prefix: &str) -> bool {
        string.starts_with(prefix)
    }
    
    /// Checks if a string ends with a suffix
    pub fn ends_with(string: &str, suffix: &str) -> bool {
        string.ends_with(suffix)
    }
    
    /// Gets the length of a string in characters
    pub fn len(string: &str) -> usize {
        string.chars().count()
    }
    
    /// Gets a substring from start to end indices
    pub fn substring(string: &str, start: usize, end: usize) -> Result<String, String> {
        if start > end || end > string.len() {
            return Err("Invalid indices".to_string());
        }
        
        Ok(string[start..end].to_string())
    }
}

/// File I/O operations
pub mod file_io {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Write};
    use std::path::Path;
    
    /// Reads the entire contents of a file as a string
    pub fn read_file(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))
    }
    
    /// Writes a string to a file
    pub fn write_file(path: &str, content: &str) -> Result<(), String> {
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write to file '{}': {}", path, e))
    }
    
    /// Appends a string to a file
    pub fn append_file(path: &str, content: &str) -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| format!("Failed to open file '{}': {}", path, e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to append to file '{}': {}", path, e))
    }
    
    /// Checks if a file exists
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).exists()
    }
    
    /// Checks if a path is a directory
    pub fn is_directory(path: &str) -> bool {
        Path::new(path).is_dir()
    }
    
    /// Checks if a path is a file
    pub fn is_file(path: &str) -> bool {
        Path::new(path).is_file()
    }
    
    /// Reads lines from a file
    pub fn read_lines(path: &str) -> Result<Vec<String>, String> {
        let file = File::open(path)
            .map_err(|e| format!("Failed to open file '{}': {}", path, e))?;
        let reader = BufReader::new(file);
        
        let mut lines = Vec::new();
        for line in reader.lines() {
            lines.push(line.map_err(|e| format!("Failed to read line: {}", e))?);
        }
        
        Ok(lines)
    }
    
    /// Creates a directory and all its parent directories
    pub fn create_dir(path: &str) -> Result<(), String> {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{}': {}", path, e))
    }
    
    /// Removes a file
    pub fn remove_file(path: &str) -> Result<(), String> {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file '{}': {}", path, e))
    }
    
    /// Copies a file from source to destination
    pub fn copy_file(src: &str, dst: &str) -> Result<(), String> {
        std::fs::copy(src, dst)
            .map_err(|e| format!("Failed to copy file from '{}' to '{}': {}", src, dst, e))
            .map(|_| ())
    }
}

/// Mathematical operations
pub mod math {
    /// Computes the absolute value of an integer
    pub fn abs_int(value: i64) -> i64 {
        if value < 0 { -value } else { value }
    }

    /// Computes the absolute value of a float
    pub fn abs_float(value: f64) -> f64 {
        if value < 0.0 { -value } else { value }
    }

    /// Computes the minimum of two integers
    pub fn min_int(a: i64, b: i64) -> i64 {
        if a < b { a } else { b }
    }

    /// Computes the minimum of two floats
    pub fn min_float(a: f64, b: f64) -> f64 {
        if a < b { a } else { b }
    }

    /// Computes the maximum of two integers
    pub fn max_int(a: i64, b: i64) -> i64 {
        if a > b { a } else { b }
    }

    /// Computes the maximum of two floats
    pub fn max_float(a: f64, b: f64) -> f64 {
        if a > b { a } else { b }
    }

    /// Computes the power of a number (base^exp)
    pub fn pow(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }

    /// Computes the square root of a number
    pub fn sqrt(value: f64) -> f64 {
        value.sqrt()
    }

    /// Computes the sine of an angle (in radians)
    pub fn sin(angle: f64) -> f64 {
        angle.sin()
    }

    /// Computes the cosine of an angle (in radians)
    pub fn cos(angle: f64) -> f64 {
        angle.cos()
    }

    /// Computes the tangent of an angle (in radians)
    pub fn tan(angle: f64) -> f64 {
        angle.tan()
    }

    /// Computes the natural logarithm of a number
    pub fn ln(value: f64) -> f64 {
        value.ln()
    }

    /// Computes the base-10 logarithm of a number
    pub fn log10(value: f64) -> f64 {
        value.log10()
    }

    /// Rounds a number to the nearest integer
    pub fn round(value: f64) -> f64 {
        value.round()
    }

    /// Floors a number (rounds down to nearest integer)
    pub fn floor(value: f64) -> f64 {
        value.floor()
    }

    /// Ceils a number (rounds up to nearest integer)
    pub fn ceil(value: f64) -> f64 {
        value.ceil()
    }
}

/// Networking utilities
#[cfg(feature = "networking")]
pub mod net {
    use std::net::{TcpStream, TcpListener, UdpSocket};
    use std::io::{Read, Write};
    
    /// Creates a TCP connection to the specified address
    pub fn tcp_connect(address: &str) -> Result<TcpStream, String> {
        TcpStream::connect(address)
            .map_err(|e| format!("Failed to connect to '{}': {}", address, e))
    }
    
    /// Sends data over a TCP connection
    pub fn tcp_send(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
        stream.write_all(data)
            .map_err(|e| format!("Failed to send data: {}", e))
    }
    
    /// Receives data from a TCP connection
    pub fn tcp_receive(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<usize, String> {
        stream.read(buffer)
            .map_err(|e| format!("Failed to receive data: {}", e))
    }
    
    /// Creates a TCP listener on the specified address
    pub fn tcp_listen(address: &str) -> Result<TcpListener, String> {
        TcpListener::bind(address)
            .map_err(|e| format!("Failed to bind to '{}': {}", address, e))
    }
    
    /// Creates a UDP socket
    pub fn udp_bind(address: &str) -> Result<UdpSocket, String> {
        UdpSocket::bind(address)
            .map_err(|e| format!("Failed to bind UDP socket to '{}': {}", address, e))
    }
}

/// Concurrency utilities
pub mod concurrency {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    /// Spawns a new thread to execute the given function
    pub fn spawn<F, T>(f: F) -> thread::JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(f)
    }
    
    /// Creates a new mutex with the given value
    pub fn mutex_new<T>(value: T) -> Arc<Mutex<T>> {
        Arc::new(Mutex::new(value))
    }
    
    /// Locks a mutex and returns the value (blocking)
    pub fn mutex_lock<T>(mutex: &Arc<Mutex<T>>) -> Result<std::sync::MutexGuard<T>, String> {
        mutex.lock()
            .map_err(|e| format!("Failed to acquire mutex lock: {}", e))
    }
    
    /// Sleeps for the specified number of milliseconds
    pub fn sleep_ms(milliseconds: u64) {
        thread::sleep(std::time::Duration::from_millis(milliseconds));
    }
    
    /// Gets the number of available CPU cores
    pub fn num_cpus() -> usize {
        num_cpus::get()
    }
}

/// Utility functions for debugging and development
pub mod debug {
    /// Prints a value with debug formatting
    pub fn debug_print<T: std::fmt::Debug>(value: &T) {
        println!("{:?}", value);
    }
    
    /// Prints a value with display formatting
    pub fn print<T: std::fmt::Display>(value: T) {
        println!("{}", value);
    }
    
    /// Prints a formatted string
    pub fn print_formatted(format_string: &str, args: Vec<String>) -> String {
        // In a real implementation, this would format the string with the arguments
        // For now, we'll just return the format string
        format_string.to_string()
    }
    
    /// Asserts that a condition is true, panicking if it's not
    pub fn assert(condition: bool, message: &str) {
        if !condition {
            panic!("Assertion failed: {}", message);
        }
    }
    
    /// Asserts that two values are equal, panicking if they're not
    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: &T, right: &T, message: &str) {
        if left != right {
            panic!("Assertion failed: {} - left: {:?}, right: {:?}", message, left, right);
        }
    }
    
    /// Times a function execution and prints the duration
    pub fn time_it<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        println!("{} took {:?}", name, duration);
        result
    }
}

/// Error handling utilities
pub mod error_handling {
    /// Result type alias for convenience
    pub type Result<T> = std::result::Result<T, String>;
    
    /// Creates a new error with the given message
    pub fn new_error(message: &str) -> String {
        message.to_string()
    }
    
    /// Creates a formatted error message
    pub fn format_error(format_string: &str, args: Vec<String>) -> String {
        format_string.to_string() // Simplified for now
    }
    
    /// Checks if a result is an error and returns a boolean
    pub fn is_error<T>(result: &Result<T>) -> bool {
        result.is_err()
    }
    
    /// Checks if a result is OK and returns a boolean
    pub fn is_ok<T>(result: &Result<T>) -> bool {
        result.is_ok()
    }
}

/// Memory management utilities
pub mod memory {
    use std::alloc::{alloc, dealloc, Layout};
    
    /// Allocates memory of the specified size
    pub fn allocate(size: usize) -> *mut u8 {
        unsafe {
            let layout = Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap();
            alloc(layout)
        }
    }
    
    /// Deallocates memory at the specified pointer
    pub unsafe fn deallocate(ptr: *mut u8, size: usize) {
        let layout = Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap();
        dealloc(ptr, layout);
    }
    
    /// Gets the size of a value in memory
    pub fn size_of<T>() -> usize {
        std::mem::size_of::<T>()
    }
    
    /// Gets the alignment of a type in memory
    pub fn align_of<T>() -> usize {
        std::mem::align_of::<T>()
    }
}

/// Time utilities
pub mod time {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    /// Gets the current Unix timestamp in seconds
    pub fn unix_timestamp() -> Result<u64, String> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))
            .map(|d| d.as_secs())
    }
    
    /// Gets the current Unix timestamp in milliseconds
    pub fn unix_timestamp_ms() -> Result<u64, String> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))
            .map(|d| d.as_millis() as u64)
    }
    
    /// Formats a timestamp as a human-readable string
    pub fn format_timestamp(timestamp: u64) -> String {
        // In a real implementation, this would format the timestamp properly
        // For now, we'll just return it as a string
        timestamp.to_string()
    }
    
    /// Sleeps for the specified number of milliseconds
    pub fn sleep(milliseconds: u64) {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
    }
}

/// Process and system utilities
pub mod system {
    use std::process::{Command, Stdio};
    
    /// Executes a shell command and returns the output
    pub fn execute_command(cmd: &str) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| format!("Failed to execute command '{}': {}", cmd, e))?;
        
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("Failed to parse command output: {}", e))
        } else {
            let stderr = String::from_utf8(output.stderr).unwrap_or_default();
            Err(format!("Command '{}' failed: {}", cmd, stderr))
        }
    }
    
    /// Gets an environment variable
    pub fn get_env_var(name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    
    /// Sets an environment variable
    pub fn set_env_var(name: &str, value: &str) {
        std::env::set_var(name, value);
    }
    
    /// Gets the current working directory
    pub fn current_dir() -> Result<String, String> {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))
            .map(|p| p.to_string_lossy().to_string())
    }
    
    /// Gets the platform information
    pub fn platform() -> String {
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    }
    
    /// Exits the process with the specified exit code
    pub fn exit(code: i32) {
        std::process::exit(code);
    }
}

// Re-export commonly used functions for easy access
pub use collections::*;
pub use string_utils::*;
pub use file_io::*;
pub use math::*;
pub use concurrency::*;
pub use debug::*;
pub use error_handling::*;
pub use memory::*;
pub use time::*;
pub use system::*;

#[cfg(feature = "networking")]
pub use net::*;