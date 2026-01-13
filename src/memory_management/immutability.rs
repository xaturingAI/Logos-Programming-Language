//! Immutability System for Logos
//! Implements immutable-by-default semantics with explicit mutability

use std::collections::HashMap;

/// Represents the mutability status of a variable
#[derive(Debug, Clone, PartialEq)]
pub enum MutabilityStatus {
    Immutable,  // Variable cannot be changed after initialization
    Mutable,    // Variable can be changed after initialization
}

/// Represents a variable with its mutability status
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,  // In a real implementation, this would be a more complex value type
    pub mutability: MutabilityStatus,
    pub scope_level: usize,
}

/// The mutability system that enforces immutability by default
pub struct MutabilitySystem {
    /// Map of variables to their properties
    variables: HashMap<String, Variable>,
    
    /// Stack of scopes to track variable lifetimes
    scopes: Vec<Vec<String>>,
    
    /// Current scope level
    current_scope: usize,
}

impl MutabilitySystem {
    /// Creates a new mutability system
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scopes: vec![Vec::new()],
            current_scope: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
        self.scopes.push(Vec::new());
    }

    /// Exit the current scope and clean up variables
    pub fn exit_scope(&mut self) {
        if self.current_scope == 0 {
            return; // Cannot exit global scope
        }

        // Get the variables in the current scope
        let current_scope_vars = self.scopes.pop().unwrap();
        
        // Remove variables from the current scope
        for var_name in current_scope_vars {
            self.variables.remove(&var_name);
        }
        
        self.current_scope -= 1;
    }

    /// Declare a new immutable variable
    pub fn declare_immutable_variable(&mut self, name: String, value: String) -> Result<(), MutabilityError> {
        self.declare_variable(name, value, MutabilityStatus::Immutable)
    }

    /// Declare a new mutable variable
    pub fn declare_mutable_variable(&mut self, name: String, value: String) -> Result<(), MutabilityError> {
        self.declare_variable(name, value, MutabilityStatus::Mutable)
    }

    /// Internal method to declare a variable with a specific mutability
    fn declare_variable(&mut self, name: String, value: String, mutability: MutabilityStatus) -> Result<(), MutabilityError> {
        // Check if variable already exists in current scope
        if self.variables.contains_key(&name) {
            return Err(MutabilityError::VariableAlreadyExists(name));
        }

        // Create the variable
        let variable = Variable {
            name: name.clone(),
            value,
            mutability,
            scope_level: self.current_scope,
        };

        // Add to variables map
        self.variables.insert(name.clone(), variable);

        // Add to current scope
        self.scopes.last_mut().unwrap().push(name);

        Ok(())
    }

    /// Update the value of a variable (only allowed for mutable variables)
    pub fn update_variable(&mut self, name: &str, new_value: String) -> Result<(), MutabilityError> {
        // Check if the variable exists
        let var = self.variables.get_mut(name)
            .ok_or_else(|| MutabilityError::VariableNotFound(name.to_string()))?;

        // Check if the variable is mutable
        if var.mutability != MutabilityStatus::Mutable {
            return Err(MutabilityError::CannotMutateImmutable(name.to_string()));
        }

        // Update the value
        var.value = new_value;

        Ok(())
    }

    /// Get the value of a variable
    pub fn get_variable_value(&self, name: &str) -> Option<&String> {
        self.variables.get(name).map(|var| &var.value)
    }

    /// Get the mutability status of a variable
    pub fn get_mutability_status(&self, name: &str) -> Option<&MutabilityStatus> {
        self.variables.get(name).map(|var| &var.mutability)
    }

    /// Check if a variable is mutable
    pub fn is_mutable(&self, name: &str) -> bool {
        if let Some(var) = self.variables.get(name) {
            var.mutability == MutabilityStatus::Mutable
        } else {
            false
        }
    }

    /// Check if a variable is immutable
    pub fn is_immutable(&self, name: &str) -> bool {
        if let Some(var) = self.variables.get(name) {
            var.mutability == MutabilityStatus::Immutable
        } else {
            false  // If variable doesn't exist, it's neither immutable nor mutable
        }
    }
}

/// Error types for mutability operations
#[derive(Debug, Clone)]
pub enum MutabilityError {
    VariableNotFound(String),
    VariableAlreadyExists(String),
    CannotMutateImmutable(String),
}

impl std::fmt::Display for MutabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutabilityError::VariableNotFound(var) => write!(f, "Variable not found: {}", var),
            MutabilityError::VariableAlreadyExists(var) => write!(f, "Variable already exists: {}", var),
            MutabilityError::CannotMutateImmutable(var) => write!(f, "Cannot mutate immutable variable: {}", var),
        }
    }
}

impl std::error::Error for MutabilityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_by_default() {
        let mut ms = MutabilitySystem::new();
        
        // Declare an immutable variable
        assert!(ms.declare_immutable_variable("x".to_string(), "5".to_string()).is_ok());
        
        // Should not be able to update it
        assert!(ms.update_variable("x", "10".to_string()).is_err());
        
        // Value should remain unchanged
        assert_eq!(ms.get_variable_value("x"), Some(&"5".to_string()));
        
        // Variable should be immutable
        assert!(ms.is_immutable("x"));
        assert!(!ms.is_mutable("x"));
    }

    #[test]
    fn test_mutable_variables() {
        let mut ms = MutabilitySystem::new();
        
        // Declare a mutable variable
        assert!(ms.declare_mutable_variable("y".to_string(), "10".to_string()).is_ok());
        
        // Should be able to update it
        assert!(ms.update_variable("y", "20".to_string()).is_ok());
        
        // Value should have changed
        assert_eq!(ms.get_variable_value("y"), Some(&"20".to_string()));
        
        // Variable should be mutable
        assert!(ms.is_mutable("y"));
        assert!(!ms.is_immutable("y"));
    }

    #[test]
    fn test_scoping() {
        let mut ms = MutabilitySystem::new();
        
        // Declare a variable in global scope
        ms.declare_immutable_variable("x".to_string(), "global".to_string()).unwrap();
        
        // Enter a new scope
        ms.enter_scope();
        
        // Declare a variable in inner scope
        ms.declare_mutable_variable("y".to_string(), "inner".to_string()).unwrap();
        
        // Both variables should be accessible
        assert_eq!(ms.get_variable_value("x"), Some(&"global".to_string()));
        assert_eq!(ms.get_variable_value("y"), Some(&"inner".to_string()));
        
        // Exit the scope
        ms.exit_scope();
        
        // Variable from inner scope should no longer exist
        assert!(ms.get_variable_value("y").is_none());
        
        // Variable from outer scope should still exist
        assert_eq!(ms.get_variable_value("x"), Some(&"global".to_string()));
    }
}