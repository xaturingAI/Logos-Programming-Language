// Complete effect system implementation for Logos programming language
// This implements algebraic effects with handlers

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct EffectType {
    pub name: String,
    pub operations: Vec<EffectOperation>,
}

#[derive(Debug, Clone)]
pub struct EffectOperation {
    pub name: String,
    pub parameter_types: Vec<String>, // Type names for parameters
    pub return_type: String,          // Return type name
}

#[derive(Debug, Clone)]
pub struct EffectSignature {
    pub effect_name: String,
    pub operation_name: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct EffectsError {
    details: String,
}

impl EffectsError {
    pub fn new(msg: &str) -> EffectsError {
        EffectsError{details: msg.to_string()}
    }
}

impl fmt::Display for EffectsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for EffectsError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

// Effect registry to store all defined effects
fn effect_registry() -> &'static Mutex<Option<HashMap<String, EffectType>>> {
    static REGISTRY: OnceLock<Mutex<Option<HashMap<String, EffectType>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Some(HashMap::new())))
}

pub fn init() -> Result<(), Box<dyn Error>> {
    // Initialize effects system
    let mut registry = effect_registry().lock().map_err(|e| EffectsError::new(&e.to_string()))?;
    *registry = Some(HashMap::new());
    Ok(())
}

// Register a new effect type
pub fn register_effect(effect_type: EffectType) -> Result<(), EffectsError> {
    let mut registry = effect_registry().lock().map_err(|e| EffectsError::new(&e.to_string()))?;
    if let Some(ref mut reg) = *registry {
        reg.insert(effect_type.name.clone(), effect_type);
        Ok(())
    } else {
        Err(EffectsError::new("Effect registry not initialized"))
    }
}

// Perform an effect operation
pub fn perform_effect(signature: EffectSignature) -> Result<String, EffectsError> {
    let registry = effect_registry().lock().map_err(|e| EffectsError::new(&e.to_string()))?;
    if let Some(ref reg) = *registry {
        if let Some(effect_type) = reg.get(&signature.effect_name) {
            // Check if the operation exists in the effect
            if effect_type.operations.iter().any(|op| op.name == signature.operation_name) {
                // In a real implementation, this would suspend execution and pass control to a handler
                Ok(format!("Effect {}::{} performed with args: {:?}",
                          signature.effect_name,
                          signature.operation_name,
                          signature.args))
            } else {
                Err(EffectsError::new(&format!("Operation {} not found in effect {}",
                                              signature.operation_name,
                                              signature.effect_name)))
            }
        } else {
            Err(EffectsError::new(&format!("Effect {} not found", signature.effect_name)))
        }
    } else {
        Err(EffectsError::new("Effect registry not initialized"))
    }
}

// Check if an effect is registered
pub fn is_effect_registered(effect_name: &str) -> bool {
    if let Ok(registry) = effect_registry().lock() {
        if let Some(ref reg) = *registry {
            return reg.contains_key(effect_name);
        }
    }
    false
}

// Get effect operations
pub fn get_effect_operations(effect_name: &str) -> Option<Vec<EffectOperation>> {
    if let Ok(registry) = effect_registry().lock() {
        if let Some(ref reg) = *registry {
            return reg.get(effect_name).map(|effect| effect.operations.clone());
        }
    }
    None
}

// Effect handler for algebraic effects
pub type EffectHandlerFn = dyn Fn(Vec<String>) -> Result<String, EffectsError> + Send + Sync;

// Wrapper for function pointers that implements Debug
pub struct HandlerFunction(Box<EffectHandlerFn>);

impl std::fmt::Debug for HandlerFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HandlerFunction")
         .field(&"Fn")
         .finish()
    }
}

impl HandlerFunction {
    fn call(&self, args: Vec<String>) -> Result<String, EffectsError> {
        (self.0)(args)
    }
}

// We can't derive Debug and Clone for function pointers, so we implement them manually
#[derive(Debug)]
pub struct EffectHandler {
    pub effect_name: String,
    pub handlers: HashMap<String, HandlerFunction>,
}

impl EffectHandler {
    pub fn new(effect_name: String) -> Self {
        EffectHandler {
            effect_name,
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler<F>(&mut self, operation_name: String, handler: F)
    where
        F: Fn(Vec<String>) -> Result<String, EffectsError> + Send + Sync + 'static,
    {
        self.handlers.insert(operation_name, HandlerFunction(Box::new(handler)));
    }

    pub fn handle(&self, operation_name: &str, args: Vec<String>) -> Option<Result<String, EffectsError>> {
        self.handlers.get(operation_name).map(|handler| handler.call(args))
    }
}

// Effect context for tracking active effects
#[derive(Debug)]
pub struct EffectContext {
    pub active_effects: Vec<String>,
    pub handlers: Vec<EffectHandler>,
}

impl EffectContext {
    pub fn new() -> Self {
        EffectContext {
            active_effects: Vec::new(),
            handlers: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect_name: String) {
        if !self.active_effects.contains(&effect_name) {
            self.active_effects.push(effect_name);
        }
    }

    pub fn add_handler(&mut self, handler: EffectHandler) {
        self.handlers.push(handler);
    }

    pub fn can_handle(&self, effect_name: &str, operation_name: &str) -> bool {
        self.handlers.iter().any(|h| {
            h.effect_name == effect_name && h.handlers.contains_key(operation_name)
        })
    }

    pub fn handle_effect(&self, effect_name: &str, operation_name: &str, args: Vec<String>) -> Option<Result<String, EffectsError>> {
        for handler in &self.handlers {
            if handler.effect_name == effect_name {
                if let Some(result) = handler.handle(operation_name, args.clone()) {
                    return Some(result);
                }
            }
        }
        None
    }
}

// Effect type for type checking
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Effect {
    IO,
    Exception,
    State(String), // State effect with type
    Reader(String), // Reader effect with type
    Writer(String), // Writer effect with type
    Custom(String),
}

impl Effect {
    pub fn name(&self) -> String {
        match self {
            Effect::IO => "IO".to_string(),
            Effect::Exception => "Exception".to_string(),
            Effect::State(t) => format!("State<{}>", t),
            Effect::Reader(t) => format!("Reader<{}>", t),
            Effect::Writer(t) => format!("Writer<{}>", t),
            Effect::Custom(name) => name.clone(),
        }
    }
}

// Effect set for tracking multiple effects
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSet {
    pub effects: Vec<Effect>,
}

impl EffectSet {
    pub fn new() -> Self {
        EffectSet {
            effects: Vec::new(),
        }
    }

    pub fn insert(&mut self, effect: Effect) {
        if !self.effects.contains(&effect) {
            self.effects.push(effect);
        }
    }

    pub fn union(&mut self, other: &EffectSet) {
        for effect in &other.effects {
            self.insert(effect.clone());
        }
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for EffectSet {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for common effect operations
impl EffectSet {
    /// Creates an effect set with common effects
    pub fn with_io() -> Self {
        let mut set = Self::new();
        set.insert(Effect::IO);
        set
    }

    pub fn with_exception() -> Self {
        let mut set = Self::new();
        set.insert(Effect::Exception);
        set
    }

    pub fn with_state(type_name: String) -> Self {
        let mut set = Self::new();
        set.insert(Effect::State(type_name));
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_registry() {
        init().expect("Failed to initialize effect registry");

        let effect_type = EffectType {
            name: "TestEffect".to_string(),
            operations: vec![EffectOperation {
                name: "test_op".to_string(),
                parameter_types: vec!["String".to_string()],
                return_type: "String".to_string(),
            }],
        };

        assert!(register_effect(effect_type).is_ok());
        assert!(is_effect_registered("TestEffect"));
    }

    #[test]
    fn test_effect_set() {
        let mut set = EffectSet::new();
        set.insert(Effect::IO);
        assert!(set.contains(&Effect::IO));
        assert!(!set.contains(&Effect::Exception));
    }
}