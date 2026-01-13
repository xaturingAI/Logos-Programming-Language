// CSP-style channels for concurrency in Logos programming language
// Implements Communicating Sequential Processes concepts with channels

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct ConcurrencyError {
    details: String,
}

impl ConcurrencyError {
    pub fn new(msg: &str) -> ConcurrencyError {
        ConcurrencyError{details: msg.to_string()}
    }
}

impl fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ConcurrencyError {
    fn description(&self) -> &str {
        &self.details
    }
}

// Channel type for CSP-style communication
pub struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T: Send + 'static> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Channel {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn send(&self, value: T) -> Result<(), ConcurrencyError> {
        self.sender.send(value)
            .map_err(|e| ConcurrencyError::new(&format!("Send error: {}", e)))
    }

    pub fn recv(&self) -> Result<T, ConcurrencyError> {
        let receiver = self.receiver.lock()
            .map_err(|e| ConcurrencyError::new(&format!("Lock error: {}", e)))?;
        receiver.recv()
            .map_err(|e| ConcurrencyError::new(&format!("Receive error: {}", e)))
    }

    pub fn try_recv(&self) -> Result<T, ConcurrencyError> {
        let receiver = self.receiver.lock()
            .map_err(|e| ConcurrencyError::new(&format!("Lock error: {}", e)))?;
        receiver.try_recv()
            .map_err(|e| ConcurrencyError::new(&format!("Try receive error: {}", e)))
    }
}

// Buffered channel for CSP-style communication
pub struct BufferedChannel<T> {
    sender: mpsc::SyncSender<T>,
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T: Send + 'static> BufferedChannel<T> {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(buffer_size);
        BufferedChannel {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn send(&self, value: T) -> Result<(), ConcurrencyError> {
        self.sender.send(value)
            .map_err(|e| ConcurrencyError::new(&format!("Buffered send error: {}", e)))
    }

    pub fn recv(&self) -> Result<T, ConcurrencyError> {
        let receiver = self.receiver.lock()
            .map_err(|e| ConcurrencyError::new(&format!("Lock error: {}", e)))?;
        receiver.recv()
            .map_err(|e| ConcurrencyError::new(&format!("Buffered receive error: {}", e)))
    }

    pub fn try_recv(&self) -> Result<T, ConcurrencyError> {
        let receiver = self.receiver.lock()
            .map_err(|e| ConcurrencyError::new(&format!("Lock error: {}", e)))?;
        receiver.try_recv()
            .map_err(|e| ConcurrencyError::new(&format!("Buffered try receive error: {}", e)))
    }
}

// Select operation for choosing between multiple channels
pub struct Select<T> {
    operations: Vec<SelectOperation<T>>,
}

enum SelectOperation<T> {
    Send { sender: mpsc::Sender<T>, value: T },
    Recv { receiver: Arc<Mutex<mpsc::Receiver<T>>> },
}

impl<T: Send + Clone + 'static> Select<T> {
    pub fn new() -> Self {
        Select {
            operations: Vec::new(),
        }
    }

    pub fn add_send(&mut self, channel: &Channel<T>, value: T) {
        self.operations.push(SelectOperation::Send {
            sender: channel.sender.clone(),
            value,
        });
    }

    pub fn add_recv(&mut self, channel: &Channel<T>) {
        self.operations.push(SelectOperation::Recv {
            receiver: channel.receiver.clone(),
        });
    }

    // Note: Actual select implementation would require more complex logic
    // This is a simplified version for demonstration
    pub fn select(&mut self) -> Result<SelectResult<T>, ConcurrencyError> {
        // In a real CSP implementation, this would wait on all operations
        // and return the first one that becomes available
        // For now, we'll just try the first operation

        if self.operations.is_empty() {
            return Err(ConcurrencyError::new("No operations in select"));
        }

        // This is a simplified implementation - in a real CSP system,
        // we'd use OS-level primitives to wait on multiple channels simultaneously
        for (i, op) in self.operations.iter().enumerate() {
            match op {
                SelectOperation::Recv { receiver } => {
                    let receiver_guard = receiver.lock()
                        .map_err(|e| ConcurrencyError::new(&format!("Lock error: {}", e)))?;
                    match receiver_guard.try_recv() {
                        Ok(value) => return Ok(SelectResult::Received(i, value)),
                        Err(_) => continue, // Channel not ready, try next
                    }
                },
                SelectOperation::Send { sender, value } => {
                    match sender.send((*value).clone()) {
                        Ok(_) => return Ok(SelectResult::Sent(i)),
                        Err(_) => continue, // Channel not ready, try next
                    }
                }
            }
        }

        // If no operation is immediately available, we could block or return an error
        // For this implementation, we'll return an error indicating timeout
        Err(ConcurrencyError::new("No channel operation ready"))
    }
}

pub enum SelectResult<T> {
    Sent(usize),           // Index of the sent operation
    Received(usize, T),    // Index of the received operation and the value
}

// CSP-style goroutine/spawn function
pub fn spawn<F, T>(f: F) -> thread::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    thread::spawn(f)
}

// Sleep function
pub fn sleep(duration: Duration) {
    thread::sleep(duration);
}

// Channel creation functions
pub fn make_channel<T: Send + 'static>() -> Channel<T> {
    Channel::new()
}

pub fn make_buffered_channel<T: Send + 'static>(buffer_size: usize) -> BufferedChannel<T> {
    BufferedChannel::new(buffer_size)
}

// Alternative spawn that works with channels
pub fn go<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(f);
}

// CSP select helper
pub fn select<T: Send + Clone + 'static>() -> Select<T> {
    Select::new()
}

// Initialization function
pub fn init() -> Result<(), Box<dyn Error>> {
    // Initialize concurrency system
    println!("CSP-style concurrency system initialized with channels");
    Ok(())
}

// Channel operations for use in the language
pub fn channel_send<T: Send + 'static>(channel: &Channel<T>, value: T) -> Result<(), ConcurrencyError> {
    channel.send(value)
}

pub fn channel_receive<T: Send + 'static>(channel: &Channel<T>) -> Result<T, ConcurrencyError> {
    channel.recv()
}

pub fn buffered_send<T: Send + 'static>(channel: &BufferedChannel<T>, value: T) -> Result<(), ConcurrencyError> {
    channel.send(value)
}

pub fn buffered_receive<T: Send + 'static>(channel: &BufferedChannel<T>) -> Result<T, ConcurrencyError> {
    channel.recv()
}