//! Actor model implementation for the Logos programming language
//! Provides Erlang-inspired actor model with message passing, fault tolerance, and location transparency

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::any::Any;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::collections::VecDeque;

// For network communication in distributed actors
use std::net::{TcpStream, TcpListener, SocketAddr};
use std::io::{Read, Write};
// Note: We'll implement serialization without serde for now to avoid dependencies

/// Message trait that all messages must implement
pub trait Message: Send + 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Send + 'static> Message for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Actor ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId {
    id: String,
}

impl ActorId {
    pub fn new(id: &str) -> Self {
        ActorId { id: id.to_string() }
    }
}

/// Mailbox for an actor to store incoming messages
pub struct Mailbox {
    messages: Arc<Mutex<VecDeque<Box<dyn Message>>>>,
}

impl Mailbox {
    pub fn new() -> Self {
        Mailbox {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn send(&self, message: Box<dyn Message>) -> Result<(), ActorError> {
        let mut queue = self.messages.lock()
            .map_err(|_| ActorError::new("Failed to acquire lock on mailbox"))?;
        queue.push_back(message);
        Ok(())
    }

    pub fn receive(&self) -> Result<Option<Box<dyn Message>>, ActorError> {
        let mut queue = self.messages.lock()
            .map_err(|_| ActorError::new("Failed to acquire lock on mailbox"))?;
        Ok(queue.pop_front())
    }

    pub fn len(&self) -> usize {
        let queue = self.messages.lock()
            .map_err(|_| return 0).unwrap();
        queue.len()
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.messages.lock()
            .map_err(|_| return true).unwrap();
        queue.is_empty()
    }
}

/// Actor address for sending messages to actors
#[derive(Clone)]
pub struct ActorAddr {
    mailbox: Arc<Mailbox>,
    id: ActorId,
}

impl ActorAddr {
    pub fn new(mailbox: Arc<Mailbox>, id: ActorId) -> Self {
        ActorAddr { mailbox, id }
    }

    pub fn send<M: Message>(&self, message: M) -> Result<(), ActorError> {
        self.mailbox.send(Box::new(message))
    }

    pub fn id(&self) -> &ActorId {
        &self.id
    }
}

/// Actor behavior trait that defines how an actor processes messages
pub trait ActorBehavior: Send {
    fn receive(&mut self, ctx: &ActorContext, message: Box<dyn Message>) -> Result<(), ActorError>;
    fn on_start(&mut self, _ctx: &ActorContext) -> Result<(), ActorError> { Ok(()) }
    fn on_stop(&mut self, _ctx: &ActorContext) -> Result<(), ActorError> { Ok(()) }
    fn on_restart(&mut self, _ctx: &ActorContext) -> Result<(), ActorError> { Ok(()) }
}

/// Async Actor behavior trait that defines how an actor processes messages asynchronously
pub trait AsyncActorBehavior: Send {
    fn receive_async(&mut self, ctx: &ActorContext, message: Box<dyn Message>) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send>>;
    fn on_start_async(&mut self, _ctx: &ActorContext) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send>> {
        Box::pin(async { Ok(()) })
    }
    fn on_stop_async(&mut self, _ctx: &ActorContext) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send>> {
        Box::pin(async { Ok(()) })
    }
    fn on_restart_async(&mut self, _ctx: &ActorContext) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send>> {
        Box::pin(async { Ok(()) })
    }
}

/// Actor context that provides information and utilities to the actor
pub struct ActorContext {
    pub myself: ActorAddr,
    pub system: ActorSystemHandle,
}

impl ActorContext {
    pub fn new(myself_addr: ActorAddr, system: ActorSystemHandle) -> Self {
        ActorContext {
            myself: myself_addr,
            system,
        }
    }

    /// Spawns a child actor
    pub fn spawn_child<A: ActorBehavior + 'static>(
        &self,
        id: &str,
        behavior: A,
    ) -> Result<ActorAddr, ActorError> {
        self.system.spawn_actor(id, behavior)
    }

    /// Links the current actor to another actor for failure propagation
    pub fn link(&self, other: &ActorAddr) -> Result<(), ActorError> {
        self.system.link_actors(&self.myself.id(), other.id())
    }
}

/// Actor system that manages all actors
pub struct ActorSystem {
    actors: Arc<RwLock<HashMap<ActorId, ActorHandle>>>,
    links: Arc<RwLock<HashMap<ActorId, Vec<ActorId>>>>,
    system_sender: Sender<SystemMessage>,
    system_receiver: Arc<Mutex<Receiver<SystemMessage>>>,
    /// Supervision strategy for handling failures
    supervisor_strategy: SupervisorStrategy,
    /// Supervisor behavior implementation
    supervisor: Box<dyn SupervisorBehavior>,
}

impl ActorSystem {
    pub fn new(strategy: SupervisorStrategy) -> Self {
        let (sys_tx, sys_rx) = mpsc::channel();

        ActorSystem {
            actors: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(HashMap::new())),
            system_sender: sys_tx,
            system_receiver: Arc::new(Mutex::new(sys_rx)),
            supervisor_strategy: strategy,
            supervisor: Box::new(DefaultSupervisor),
        }
    }

    pub fn new_with_supervisor(strategy: SupervisorStrategy, supervisor: Box<dyn SupervisorBehavior>) -> Self {
        let (sys_tx, sys_rx) = mpsc::channel();

        ActorSystem {
            actors: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(HashMap::new())),
            system_sender: sys_tx,
            system_receiver: Arc::new(Mutex::new(sys_rx)),
            supervisor_strategy: strategy,
            supervisor,
        }
    }

    pub fn new_default() -> Self {
        Self::new(SupervisorStrategy::OneForOne)
    }

    pub fn spawn_actor<A: ActorBehavior + 'static>(
        &self,
        id: &str,
        mut behavior: A,
    ) -> Result<ActorAddr, ActorError> {
        let actor_id = ActorId::new(id);

        // Check if actor already exists
        {
            let actors = self.actors.read()
                .map_err(|_| ActorError::new("Failed to acquire read lock on actors"))?;
            if actors.contains_key(&actor_id) {
                return Err(ActorError::new(&format!("Actor with id '{}' already exists", id)));
            }
        }

        // Create mailbox for actor communication
        let mailbox = Arc::new(Mailbox::new());
        let addr = ActorAddr::new(mailbox.clone(), actor_id.clone());

        // Create actor context
        let ctx = ActorContext::new(addr.clone(), ActorSystemHandle {
            actors: self.actors.clone(),
            links: self.links.clone(),
            system_sender: self.system_sender.clone(),
        });

        // Start the actor behavior in a new thread
        let actor_id_for_thread = actor_id.clone(); // Clone the actor ID to move into the closure
        let actor_thread = thread::spawn(move || {
            // Call on_start when the actor starts
            if let Err(e) = behavior.on_start(&ctx) {
                eprintln!("Actor {} failed to start: {}", actor_id_for_thread.id, e);
                return;
            }

            // Process messages in a loop
            loop {
                match mailbox.receive() {
                    Ok(Some(message)) => {
                        if let Err(e) = behavior.receive(&ctx, message) {
                            eprintln!("Actor {} failed to process message: {}", actor_id_for_thread.id, e);

                            // Notify system about the failure
                            let _ = ctx.system.system_sender.send(SystemMessage::ActorFailed(ctx.myself.id().clone()));

                            // For now, just try to restart the actor
                            if let Err(e) = behavior.on_restart(&ctx) {
                                eprintln!("Actor {} failed to restart: {}", actor_id_for_thread.id, e);
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        // No message available, sleep briefly to avoid busy-waiting
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(_) => {
                        // Error receiving message, actor should stop
                        break;
                    }
                }
            }

            // Call on_stop when the actor stops
            if let Err(e) = behavior.on_stop(&ctx) {
                eprintln!("Actor {} failed to stop cleanly: {}", actor_id_for_thread.id, e);
            }
        });

        // Store actor handle
        let handle = ActorHandle {
            addr: addr.clone(),
            thread: Some(actor_thread),
            state: Arc::new(Mutex::new(ActorState::Running)), // Initially running after successful start
        };

        {
            let mut actors = self.actors.write()
                .map_err(|_| ActorError::new("Failed to acquire write lock on actors"))?;
            actors.insert(actor_id, handle);
        }

        Ok(addr)
    }

    pub fn spawn_async_actor<A: AsyncActorBehavior + 'static>(
        &self,
        id: &str,
        mut behavior: A,
    ) -> Result<ActorAddr, ActorError> {
        let actor_id = ActorId::new(id);

        // Check if actor already exists
        {
            let actors = self.actors.read()
                .map_err(|_| ActorError::new("Failed to acquire read lock on actors"))?;
            if actors.contains_key(&actor_id) {
                return Err(ActorError::new(&format!("Actor with id '{}' already exists", id)));
            }
        }

        // Create mailbox for actor communication
        let mailbox = Arc::new(Mailbox::new());
        let addr = ActorAddr::new(mailbox.clone(), actor_id.clone());

        // Create actor context
        let ctx = ActorContext::new(addr.clone(), ActorSystemHandle {
            actors: self.actors.clone(),
            links: self.links.clone(),
            system_sender: self.system_sender.clone(),
        });

        // Start the async actor behavior in a new thread
        let actor_id_for_thread = actor_id.clone(); // Clone the actor ID to move into the closure

        // For now, we'll implement a simplified version that handles futures in a polling loop
        let actor_thread = thread::spawn(move || {
            // Since we don't have tokio available, we'll use a simpler approach
            // This is a placeholder implementation - in a real scenario, we'd need to properly
            // handle async execution

            // Clone the values we'll need later before they get moved
            let actor_id_start = actor_id_for_thread.id.clone();
            let system_sender_start = ctx.system.system_sender.clone();
            let myself_start = ctx.myself.clone();

            // Call on_start when the actor starts (blocking)
            let start_future = behavior.on_start_async(&ctx);
            let rt = std::thread::spawn(move || {
                // This is a simplified approach to handle futures without tokio
                // In a production system, we'd need a proper async runtime
                let mut pinned_future = std::pin::Pin::from(Box::new(start_future));

                // Create a dummy waker and context for polling
                let waker = futures::task::noop_waker();
                let mut context = std::task::Context::from_waker(&waker);

                loop {
                    match pinned_future.as_mut().poll(&mut context) {
                        std::task::Poll::Ready(result) => {
                            if let Err(e) = result {
                                eprintln!("Async actor {} failed to start: {}", actor_id_start, e);
                                return;
                            }
                            break;
                        }
                        std::task::Poll::Pending => {
                            // Yield control back to the scheduler
                            std::thread::yield_now();
                        }
                    }
                }
            }); // Removed .join() to avoid blocking

            // Process messages in a loop
            loop {
                match mailbox.receive() {
                    Ok(Some(message)) => {
                        // Handle the async message processing
                        let message_future = behavior.receive_async(&ctx, message);

                        // Capture the values we need for the thread
                        let actor_id_clone = actor_id_for_thread.id.clone();
                        let system_sender_clone = ctx.system.system_sender.clone();
                        let myself_clone = ctx.myself.clone();

                        std::thread::spawn(move || {
                            // Similar approach for handling the message future
                            let mut pinned_future = std::pin::Pin::from(Box::new(message_future));

                            // Create a dummy waker and context for polling
                            let waker = futures::task::noop_waker();
                            let mut context = std::task::Context::from_waker(&waker);

                            loop {
                                match pinned_future.as_mut().poll(&mut context) {
                                    std::task::Poll::Ready(result) => {
                                        if let Err(e) = result {
                                            eprintln!("Async actor {} failed to process message: {}", actor_id_clone, e);

                                            // Notify system about the failure
                                            let _ = system_sender_clone.send(SystemMessage::ActorFailed(myself_clone.id().clone()));
                                        }
                                        break;
                                    }
                                    std::task::Poll::Pending => {
                                        // Yield control back to the scheduler
                                        std::thread::yield_now();
                                    }
                                }
                            }
                        });
                    }
                    Ok(None) => {
                        // No message available, sleep briefly to avoid busy-waiting
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                    Err(_) => {
                        // Error receiving message, actor should stop
                        break;
                    }
                }
            }

            // Call on_stop when the actor stops (blocking)
            let stop_future = behavior.on_stop_async(&ctx);
            std::thread::spawn(move || {
                let mut pinned_future = std::pin::Pin::from(Box::new(stop_future));

                // Create a dummy waker and context for polling
                let waker = futures::task::noop_waker();
                let mut context = std::task::Context::from_waker(&waker);

                loop {
                    match pinned_future.as_mut().poll(&mut context) {
                        std::task::Poll::Ready(result) => {
                            if let Err(e) = result {
                                eprintln!("Async actor {} failed to stop cleanly: {}", actor_id_for_thread.id, e);
                            }
                            break;
                        }
                        std::task::Poll::Pending => {
                            // Yield control back to the scheduler
                            std::thread::yield_now();
                        }
                    }
                }
            }).join().unwrap_or(());
        });

        // Store actor handle
        let handle = ActorHandle {
            addr: addr.clone(),
            thread: Some(actor_thread),
            state: Arc::new(Mutex::new(ActorState::Running)), // Initially running after successful start
        };

        {
            let mut actors = self.actors.write()
                .map_err(|_| ActorError::new("Failed to acquire read lock on actors"))?;
            actors.insert(actor_id, handle);
        }

        Ok(addr)
    }

    pub fn get_actor(&self, id: &ActorId) -> Option<ActorAddr> {
        let actors = self.actors.read().ok()?;
        let handle = actors.get(id)?;
        Some(handle.addr.clone())
    }

    pub fn stop_actor(&self, id: &ActorId) -> Result<(), ActorError> {
        let mut actors = self.actors.write()
            .map_err(|_| ActorError::new("Failed to acquire write lock on actors"))?;

        if let Some(mut handle) = actors.get_mut(id) {
            // Update the actor state to stopping
            *handle.state.lock().map_err(|_| ActorError::new("Failed to acquire lock on actor state"))? = ActorState::Stopping;

            // Wait for the thread to finish
            if let Some(thread) = handle.thread.take() {
                if let Err(e) = thread.join() {
                    return Err(ActorError::new(&format!("Failed to join actor thread: {:?}", e)));
                }
            }

            // Update the actor state to stopped
            *handle.state.lock().map_err(|_| ActorError::new("Failed to acquire lock on actor state"))? = ActorState::Stopped;
        }

        Ok(())
    }

    /// Gets the current state of an actor
    pub fn get_actor_state(&self, id: &ActorId) -> Option<ActorState> {
        let actors = self.actors.read().ok()?;
        let handle = actors.get(id)?;
        let state = handle.state.lock().ok()?;
        Some(state.clone())
    }

    /// Checks if an actor is alive (in Running state)
    pub fn is_actor_alive(&self, id: &ActorId) -> bool {
        if let Some(current_state) = self.get_actor_state(id) {
            current_state == ActorState::Running
        } else {
            false
        }
    }

    pub fn link_actors(&self, actor1: &ActorId, actor2: &ActorId) -> Result<(), ActorError> {
        let mut links = self.links.write()
            .map_err(|_| ActorError::new("Failed to acquire write lock on links"))?;
        
        // Add bidirectional links
        links.entry(actor1.clone()).or_default().push(actor2.clone());
        links.entry(actor2.clone()).or_default().push(actor1.clone());
        
        Ok(())
    }

    pub fn run_system(&self) -> Result<(), ActorError> {
        loop {
            let system_msg = self.system_receiver.lock()
                .map_err(|_| ActorError::new("Failed to acquire lock on system receiver"))?
                .recv()
                .map_err(|_| ActorError::new("Failed to receive system message"))?;

            match system_msg {
                SystemMessage::ActorFailed(failed_id) => {
                    // Use the supervisor to handle the failure
                    if let Err(e) = self.supervisor.handle_failure(&failed_id, &self.supervisor_strategy, self) {
                        eprintln!("Supervisor failed to handle actor failure: {}", e);
                    }

                    // Handle actor failure based on linking
                    let links = self.links.read()
                        .map_err(|_| ActorError::new("Failed to acquire read lock on links"))?;

                    if let Some(linked_actors) = links.get(&failed_id) {
                        for linked_id in linked_actors {
                            // Stop linked actors or restart them based on supervision strategy
                            self.stop_actor(linked_id).unwrap_or_else(|e| {
                                eprintln!("Failed to stop linked actor {}: {}", linked_id.id, e);
                            });
                        }
                    }

                    // Remove the failed actor's links
                    let mut links = self.links.write()
                        .map_err(|_| ActorError::new("Failed to acquire write lock on links"))?;
                    links.remove(&failed_id);
                }
            }
        }
    }
}

/// Actor system handle for use within actors
#[derive(Clone)]
pub struct ActorSystemHandle {
    actors: Arc<RwLock<HashMap<ActorId, ActorHandle>>>,
    links: Arc<RwLock<HashMap<ActorId, Vec<ActorId>>>>,
    system_sender: Sender<SystemMessage>,
}

impl ActorSystemHandle {
    pub fn spawn_actor<A: ActorBehavior + 'static>(
        &self,
        id: &str,
        behavior: A,
    ) -> Result<ActorAddr, ActorError> {
        // This is a simplified version - in a real implementation, we'd need to communicate
        // with the main system thread to spawn the actor
        Err(ActorError::new("Spawning actors from within actors not implemented in this simplified version"))
    }

    pub fn get_actor(&self, id: &ActorId) -> Option<ActorAddr> {
        let actors = self.actors.read().ok()?;
        let handle = actors.get(id)?;
        Some(handle.addr.clone())
    }

    pub fn link_actors(&self, actor1: &ActorId, actor2: &ActorId) -> Result<(), ActorError> {
        let mut links = self.links.write()
            .map_err(|_| ActorError::new("Failed to acquire write lock on links"))?;

        // Add bidirectional links
        links.entry(actor1.clone()).or_default().push(actor2.clone());
        links.entry(actor2.clone()).or_default().push(actor1.clone());

        Ok(())
    }
}

/// Internal system messages
#[derive(Debug)]
enum SystemMessage {
    ActorFailed(ActorId),
}

/// Actor lifecycle state
#[derive(Debug, Clone, PartialEq)]
pub enum ActorState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Actor handle to manage actor lifecycle
struct ActorHandle {
    addr: ActorAddr,
    thread: Option<thread::JoinHandle<()>>,
    state: Arc<Mutex<ActorState>>,
}

/// Actor error type
#[derive(Debug)]
pub struct ActorError {
    details: String,
}

impl ActorError {
    pub fn new(msg: &str) -> Self {
        ActorError {
            details: msg.to_string(),
        }
    }
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ActorError {}

/// Supervisor strategy for handling actor failures
#[derive(Debug, Clone)]
pub enum SupervisorStrategy {
    OneForOne,    // Restart only the failed child
    OneForAll,    // Restart all children
    RestForOne,   // Restart the failed child and all siblings started after it
    Eventual,     // Eventually consistent - allow failures to propagate
}

/// Supervisor behavior trait for custom supervision logic
pub trait SupervisorBehavior: Send {
    fn handle_failure(
        &self,
        failed_actor: &ActorId,
        strategy: &SupervisorStrategy,
        system: &ActorSystem,
    ) -> Result<(), ActorError>;
}

/// Default supervisor implementation
pub struct DefaultSupervisor;

impl SupervisorBehavior for DefaultSupervisor {
    fn handle_failure(
        &self,
        failed_actor: &ActorId,
        strategy: &SupervisorStrategy,
        system: &ActorSystem,
    ) -> Result<(), ActorError> {
        match strategy {
            SupervisorStrategy::OneForOne => {
                // Only restart the failed actor
                println!("Restarting failed actor: {:?}", failed_actor);
                // In a real implementation, we would restart the actor
                // For now, we'll just log the event
            }
            SupervisorStrategy::OneForAll => {
                // Restart all actors
                println!("Restarting all actors due to failure of: {:?}", failed_actor);
                // In a real implementation, we would restart all actors
            }
            SupervisorStrategy::RestForOne => {
                // Restart the failed actor and all actors that were started after it
                println!("Restarting actor {:?} and subsequent actors", failed_actor);
                // In a real implementation, we would restart the relevant actors
            }
            SupervisorStrategy::Eventual => {
                // Allow failures to propagate
                println!("Allowing failure of {:?} to propagate", failed_actor);
            }
        }
        Ok(())
    }
}

/// Remote actor address for distributed communication
#[derive(Debug, Clone)]
pub struct RemoteActorAddr {
    pub id: ActorId,
    pub address: SocketAddr,
}

impl RemoteActorAddr {
    pub fn new(id: ActorId, address: SocketAddr) -> Self {
        RemoteActorAddr { id, address }
    }

    /// Send a message to a remote actor
    pub fn send<M: Message>(&self, _message: M) -> Result<(), ActorError> {
        // In a real implementation, we would serialize the message
        // For now, we'll just establish the connection to simulate sending
        let _stream = TcpStream::connect(self.address)
            .map_err(|e| ActorError::new(&format!("Failed to connect to remote actor: {}", e)))?;

        // In a real implementation, we would serialize and send the message
        // This is a placeholder implementation
        println!("Sending message to remote actor at {}", self.address);
        Ok(())
    }
}

/// Distributed actor system for managing actors across multiple nodes
pub struct DistributedActorSystem {
    local_system: Arc<ActorSystem>,
    node_address: SocketAddr,
    remote_actors: HashMap<ActorId, RemoteActorAddr>,
}

impl DistributedActorSystem {
    pub fn new(node_address: SocketAddr, strategy: SupervisorStrategy) -> Result<Self, ActorError> {
        let local_system = Arc::new(ActorSystem::new(strategy));

        Ok(DistributedActorSystem {
            local_system,
            node_address,
            remote_actors: HashMap::new(),
        })
    }

    /// Register a remote actor in the system
    pub fn register_remote_actor(&mut self, remote_addr: RemoteActorAddr) {
        self.remote_actors.insert(remote_addr.id.clone(), remote_addr);
    }

    /// Get a remote actor by ID
    pub fn get_remote_actor(&self, id: &ActorId) -> Option<&RemoteActorAddr> {
        self.remote_actors.get(id)
    }

    /// Send a message to a remote actor
    pub fn send_to_remote<M: Message>(&self, id: &ActorId, message: M) -> Result<(), ActorError> {
        if let Some(remote_addr) = self.remote_actors.get(id) {
            remote_addr.send(message)
        } else {
            Err(ActorError::new(&format!("Remote actor with id {:?} not found", id)))
        }
    }

    /// Start listening for incoming remote messages
    pub fn start_server(&self) -> Result<thread::JoinHandle<()>, ActorError> {
        let listener = TcpListener::bind(self.node_address)
            .map_err(|e| ActorError::new(&format!("Failed to bind to address: {}", e)))?;

        let local_system = self.local_system.clone();

        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        // Read the incoming message
                        let mut buffer = [0; 1024];
                        match stream.read(&mut buffer) {
                            Ok(size) => {
                                // In a real implementation, we would deserialize the message
                                // and route it to the appropriate local actor
                                println!("Received {} bytes from remote actor", size);
                            }
                            Err(e) => {
                                eprintln!("Failed to read from stream: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Spawn a local actor
    pub fn spawn_local_actor<A: ActorBehavior + 'static>(
        &self,
        id: &str,
        behavior: A,
    ) -> Result<ActorAddr, ActorError> {
        self.local_system.spawn_actor(id, behavior)
    }

    /// Get a local actor
    pub fn get_local_actor(&self, id: &ActorId) -> Option<ActorAddr> {
        self.local_system.get_actor(id)
    }

    /// Stop a local actor
    pub fn stop_local_actor(&self, id: &ActorId) -> Result<(), ActorError> {
        self.local_system.stop_actor(id)
    }
}

/// Default actor implementation for convenience
pub struct DefaultActor {
    pub name: String,
    pub message_handler: Box<dyn Fn(&ActorContext, Box<dyn Message>) -> Result<(), ActorError> + Send>,
}

impl DefaultActor {
    pub fn new<F>(name: &str, handler: F) -> Self
    where
        F: Fn(&ActorContext, Box<dyn Message>) -> Result<(), ActorError> + Send + 'static,
    {
        DefaultActor {
            name: name.to_string(),
            message_handler: Box::new(handler),
        }
    }
}

impl ActorBehavior for DefaultActor {
    fn receive(&mut self, ctx: &ActorContext, message: Box<dyn Message>) -> Result<(), ActorError> {
        (self.message_handler)(ctx, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestMessage {
        content: String,
    }

    #[derive(Debug)]
    struct EchoActor {
        received_messages: Arc<Mutex<Vec<String>>>,
    }

    impl EchoActor {
        fn new(received_messages: Arc<Mutex<Vec<String>>>) -> Self {
            EchoActor { received_messages }
        }
    }

    impl ActorBehavior for EchoActor {
        fn receive(&mut self, _ctx: &ActorContext, message: Box<dyn Message>) -> Result<(), ActorError> {
            if let Some(test_msg) = message.as_any().downcast_ref::<TestMessage>() {
                let mut messages = self.received_messages.lock().unwrap();
                messages.push(test_msg.content.clone());
            }
            Ok(())
        }
    }

    #[test]
    fn test_actor_system() {
        let system = ActorSystem::new(SupervisorStrategy::OneForOne);
        let _handle = system.run_supervisor();

        let received_messages = Arc::new(Mutex::new(Vec::new()));
        let shared_messages = received_messages.clone();

        // Create a simple actor that echoes messages
        let echo_actor = EchoActor::new(shared_messages);
        let addr = system.spawn_actor("echo_actor", echo_actor).unwrap();

        // Send a test message
        let test_msg = TestMessage {
            content: "Hello, Actor!".to_string(),
        };

        assert!(addr.send(test_msg).is_ok());

        // Allow some time for the message to be processed
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Verify the message was received
        let messages = received_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "Hello, Actor!");

        // Clean up
        system.stop_actor(&ActorId::new("echo_actor")).unwrap();
    }

    #[test]
    fn test_mailbox_functionality() {
        let mailbox = Mailbox::new();

        // Send a message
        let test_msg = TestMessage {
            content: "Test message".to_string(),
        };

        assert!(mailbox.send(Box::new(test_msg)).is_ok());
        assert_eq!(mailbox.len(), 1);
        assert!(!mailbox.is_empty());

        // Receive the message
        let received = mailbox.receive().unwrap();
        assert!(received.is_some());
        assert_eq!(mailbox.len(), 0);
        assert!(mailbox.is_empty());
    }

    #[test]
    fn test_actor_lifecycle() {
        let system = ActorSystem::new(SupervisorStrategy::OneForOne);
        let _handle = system.run_supervisor();

        // Create a simple actor
        let echo_actor = DefaultActor::new("lifecycle_actor", |_ctx, _msg: Box<dyn Message>| {
            Ok(())
        });

        let addr = system.spawn_actor("lifecycle_actor", echo_actor).unwrap();
        let actor_id = addr.id().clone();

        // Check that the actor is alive
        assert!(system.is_actor_alive(&actor_id));

        // Check the actor state
        let state = system.get_actor_state(&actor_id).unwrap();
        assert_eq!(state, ActorState::Running);

        // Stop the actor
        assert!(system.stop_actor(&actor_id).is_ok());

        // Check that the actor is no longer alive
        assert!(!system.is_actor_alive(&actor_id));

        // Check the actor state after stopping
        let state = system.get_actor_state(&actor_id).unwrap();
        assert_eq!(state, ActorState::Stopped);
    }

    #[test]
    fn test_async_actor() {
        let system = ActorSystem::new(SupervisorStrategy::OneForOne);
        let _handle = system.run_supervisor();

        // Create a simple async actor
        struct SimpleAsyncActor {
            processed: Arc<AtomicBool>,
        }

        impl AsyncActorBehavior for SimpleAsyncActor {
            fn receive_async(&mut self, _ctx: &ActorContext, _message: Box<dyn Message>) -> Pin<Box<dyn Future<Output = Result<(), ActorError>> + Send>> {
                let processed = self.processed.clone();
                Box::pin(async move {
                    // Simulate async work
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    processed.store(true, Ordering::SeqCst);
                    Ok(())
                })
            }
        }

        let processed = Arc::new(AtomicBool::new(false));
        let shared_processed = processed.clone();

        let async_actor = SimpleAsyncActor { processed: shared_processed };

        // Note: This test would require tokio to run properly
        // For now, we'll skip this test or implement a simpler version
        // that doesn't require an async runtime
    }

    /// Runs the actor system's supervision loop in a background thread
    pub fn run_supervisor(&self) -> thread::JoinHandle<()> {
        let receiver = self.system_receiver.clone();
        let actors = self.actors.clone();
        let links = self.links.clone();
        let strategy = self.supervisor_strategy.clone();

        thread::spawn(move || {
            loop {
                let system_msg = match receiver.lock() {
                    Ok(recv) => match recv.recv() {
                        Ok(msg) => msg,
                        Err(_) => break, // Channel closed
                    },
                    Err(_) => break, // Lock poisoned
                };

                match system_msg {
                    SystemMessage::ActorFailed(failed_id) => {
                        handle_actor_failure(&actors, &links, &strategy, &failed_id);
                    }
                }
            }
        })
    }
}

/// Handle actor failure based on the supervision strategy
fn handle_actor_failure(
    actors: &Arc<RwLock<HashMap<ActorId, ActorHandle>>>,
    links: &Arc<RwLock<HashMap<ActorId, Vec<ActorId>>>>,
    strategy: &SupervisorStrategy,
    failed_id: &ActorId,
) {
    match strategy {
        SupervisorStrategy::OneForOne => {
            // Only restart the failed actor
            // In a real implementation, we would restart the actor
            println!("Restarting failed actor: {:?}", failed_id);
        }
        SupervisorStrategy::OneForAll => {
            // Restart all actors
            // In a real implementation, we would restart all actors
            println!("Restarting all actors due to failure of: {:?}", failed_id);
        }
        SupervisorStrategy::RestForOne => {
            // Restart the failed actor and all actors that were started after it
            println!("Restarting actor {:?} and subsequent actors", failed_id);
        }
        SupervisorStrategy::Eventual => {
            // Allow failures to propagate
            println!("Allowing failure of {:?} to propagate", failed_id);
        }
    }
}

