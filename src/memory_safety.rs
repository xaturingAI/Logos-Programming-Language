//! Enhanced memory safety module for Eux
//! Implements ownership, borrowing, and lifetime tracking

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Memory safety error types
#[derive(Debug, Clone)]
pub enum MemorySafetyError {
    DoubleFree(String),
    UseAfterFree(String),
    DanglingReference(String),
    InvalidBorrow(String),
    LifetimeViolation(String),
    OwnershipTransferError(String),
}

impl fmt::Display for MemorySafetyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemorySafetyError::DoubleFree(ptr) => write!(f, "Double free error: pointer {} freed twice", ptr),
            MemorySafetyError::UseAfterFree(ptr) => write!(f, "Use after free: pointer {} accessed after being freed", ptr),
            MemorySafetyError::DanglingReference(var) => write!(f, "Dangling reference: variable {} points to freed memory", var),
            MemorySafetyError::InvalidBorrow(desc) => write!(f, "Invalid borrow: {}", desc),
            MemorySafetyError::LifetimeViolation(desc) => write!(f, "Lifetime violation: {}", desc),
            MemorySafetyError::OwnershipTransferError(desc) => write!(f, "Ownership transfer error: {}", desc),
        }
    }
}

impl std::error::Error for MemorySafetyError {}

/// Ownership status of a value
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipStatus {
    Owned,           // Value is owned by current scope
    Borrowed,        // Value is borrowed (immutable)
    MutablyBorrowed, // Value is borrowed mutably
    Moved,           // Value has been moved to another owner
    Shared,          // Value is shared among multiple owners (reference counted)
}

/// Lifetime information for a variable
#[derive(Debug, Clone)]
pub struct Lifetime {
    pub scope_id: usize,
    pub depth: usize,
    pub region: String, // Function, block, thread, etc.
}

/// Memory safety environment that tracks ownership and lifetimes
#[derive(Debug, Clone)]
pub struct MemorySafetyEnv {
    /// Tracks ownership status of variables
    ownership_map: HashMap<String, OwnershipStatus>,
    
    /// Tracks lifetimes of variables
    lifetime_map: HashMap<String, Lifetime>,
    
    /// Tracks active borrows
    active_borrows: HashMap<String, Vec<String>>, // variable -> borrowers
    
    /// Tracks mutable borrows
    active_mutable_borrows: HashMap<String, Vec<String>>, // variable -> mutable borrowers
    
    /// Tracks allocated memory addresses
    allocated_addresses: HashSet<String>,
    
    /// Tracks freed memory addresses
    freed_addresses: HashSet<String>,
    
    /// Current scope depth
    current_scope_depth: usize,
    
    /// Current scope ID
    current_scope_id: usize,
}

impl MemorySafetyEnv {
    pub fn new() -> Self {
        Self {
            ownership_map: HashMap::new(),
            lifetime_map: HashMap::new(),
            active_borrows: HashMap::new(),
            active_mutable_borrows: HashMap::new(),
            allocated_addresses: HashSet::new(),
            freed_addresses: HashSet::new(),
            current_scope_depth: 0,
            current_scope_id: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_scope_depth += 1;
        self.current_scope_id += 1;
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) -> Result<(), MemorySafetyError> {
        // Clean up variables that go out of scope
        let to_remove: Vec<String> = self.lifetime_map
            .iter()
            .filter(|(_, lifetime)| lifetime.scope_id == self.current_scope_id)
            .map(|(var, _)| var.clone())
            .collect();

        for var in to_remove {
            self.cleanup_variable(&var)?;
        }

        if self.current_scope_depth > 0 {
            self.current_scope_depth -= 1;
        }
        self.current_scope_id = self.current_scope_depth; // Reset to depth-based ID
        
        Ok(())
    }

    /// Register a new variable with ownership
    pub fn register_variable(&mut self, name: String, ownership: OwnershipStatus) -> Result<(), MemorySafetyError> {
        // Check if variable already exists
        if self.ownership_map.contains_key(&name) {
            return Err(MemorySafetyError::InvalidBorrow(
                format!("Variable {} already exists in current scope", name)
            ));
        }

        let lifetime = Lifetime {
            scope_id: self.current_scope_id,
            depth: self.current_scope_depth,
            region: "local".to_string(),
        };

        self.ownership_map.insert(name.clone(), ownership);
        self.lifetime_map.insert(name, lifetime);
        Ok(())
    }

    /// Transfer ownership of a variable
    pub fn transfer_ownership(&mut self, from: &str, to: &str) -> Result<(), MemorySafetyError> {
        if let Some(status) = self.ownership_map.get(from) {
            if *status != OwnershipStatus::Owned {
                return Err(MemorySafetyError::OwnershipTransferError(
                    format!("Cannot transfer ownership of non-owned variable: {}", from)
                ));
            }

            // Check if 'from' has any active borrows
            if self.has_active_borrows(from) {
                return Err(MemorySafetyError::OwnershipTransferError(
                    format!("Cannot transfer ownership of borrowed variable: {}", from)
                ));
            }

            // Update ownership
            self.ownership_map.remove(from);
            self.ownership_map.insert(to.to_string(), OwnershipStatus::Owned);
            
            // Transfer lifetime info
            if let Some(lifetime) = self.lifetime_map.remove(from) {
                self.lifetime_map.insert(to.to_string(), lifetime);
            }

            Ok(())
        } else {
            Err(MemorySafetyError::UseAfterFree(from.to_string()))
        }
    }

    /// Borrow a variable immutably
    pub fn borrow_immutably(&mut self, variable: &str, borrower: &str) -> Result<(), MemorySafetyError> {
        if !self.ownership_map.contains_key(variable) {
            return Err(MemorySafetyError::UseAfterFree(variable.to_string()));
        }

        // Check if variable is already mutably borrowed
        if self.active_mutable_borrows.contains_key(variable) {
            return Err(MemorySafetyError::InvalidBorrow(
                format!("Cannot borrow {} immutably while it's mutably borrowed", variable)
            ));
        }

        // Add to active borrows
        self.active_borrows.entry(variable.to_string())
            .or_insert_with(Vec::new)
            .push(borrower.to_string());

        Ok(())
    }

    /// Borrow a variable mutably
    pub fn borrow_mutably(&mut self, variable: &str, borrower: &str) -> Result<(), MemorySafetyError> {
        if !self.ownership_map.contains_key(variable) {
            return Err(MemorySafetyError::UseAfterFree(variable.to_string()));
        }

        // Check if variable is already borrowed (immutably or mutably)
        if self.active_borrows.contains_key(variable) || self.active_mutable_borrows.contains_key(variable) {
            return Err(MemorySafetyError::InvalidBorrow(
                format!("Cannot mutably borrow {} while it's already borrowed", variable)
            ));
        }

        // Check if variable is owned
        if let Some(status) = self.ownership_map.get(variable) {
            if *status == OwnershipStatus::Moved {
                return Err(MemorySafetyError::UseAfterFree(variable.to_string()));
            }
        }

        // Add to active mutable borrows
        self.active_mutable_borrows.entry(variable.to_string())
            .or_insert_with(Vec::new)
            .push(borrower.to_string());

        Ok(())
    }

    /// Release an immutable borrow
    pub fn release_borrow(&mut self, variable: &str, borrower: &str) {
        if let Some(borrowers) = self.active_borrows.get_mut(variable) {
            borrowers.retain(|b| b != borrower);
            if borrowers.is_empty() {
                self.active_borrows.remove(variable);
            }
        }
    }

    /// Release a mutable borrow
    pub fn release_mutable_borrow(&mut self, variable: &str, borrower: &str) {
        if let Some(borrowers) = self.active_mutable_borrows.get_mut(variable) {
            borrowers.retain(|b| b != borrower);
            if borrowers.is_empty() {
                self.active_mutable_borrows.remove(variable);
            }
        }
    }

    /// Check if a variable has active borrows
    fn has_active_borrows(&self, variable: &str) -> bool {
        self.active_borrows.contains_key(variable) || 
        self.active_mutable_borrows.contains_key(variable)
    }

    /// Mark a variable as moved
    pub fn mark_moved(&mut self, variable: &str) -> Result<(), MemorySafetyError> {
        if !self.ownership_map.contains_key(variable) {
            return Err(MemorySafetyError::UseAfterFree(variable.to_string()));
        }

        // Check if variable has active borrows
        if self.has_active_borrows(variable) {
            return Err(MemorySafetyError::InvalidBorrow(
                format!("Cannot move variable {} while it's borrowed", variable)
            ));
        }

        *self.ownership_map.get_mut(variable).unwrap() = OwnershipStatus::Moved;
        Ok(())
    }

    /// Check if access to a variable is safe
    pub fn check_access(&self, variable: &str) -> Result<(), MemorySafetyError> {
        if !self.ownership_map.contains_key(variable) {
            return Err(MemorySafetyError::UseAfterFree(variable.to_string()));
        }

        let status = self.ownership_map.get(variable).unwrap();
        match status {
            OwnershipStatus::Moved => {
                Err(MemorySafetyError::UseAfterFree(variable.to_string()))
            },
            _ => Ok(()),
        }
    }

    /// Allocate memory and track it
    pub fn allocate_memory(&mut self, address: String) -> Result<(), MemorySafetyError> {
        if self.freed_addresses.contains(&address) {
            return Err(MemorySafetyError::LifetimeViolation(
                format!("Attempting to reuse freed address: {}", address)
            ));
        }

        self.allocated_addresses.insert(address);
        Ok(())
    }

    /// Free memory and track it
    pub fn free_memory(&mut self, address: String) -> Result<(), MemorySafetyError> {
        if !self.allocated_addresses.contains(&address) {
            return Err(MemorySafetyError::DoubleFree(address));
        }

        if self.freed_addresses.contains(&address) {
            return Err(MemorySafetyError::DoubleFree(address));
        }

        self.allocated_addresses.remove(&address);
        self.freed_addresses.insert(address);
        Ok(())
    }

    /// Cleanup a variable when it goes out of scope
    fn cleanup_variable(&mut self, variable: &str) -> Result<(), MemorySafetyError> {
        if let Some(status) = self.ownership_map.get(variable) {
            match status {
                OwnershipStatus::Owned => {
                    // Variable goes out of scope, check if it has active borrows
                    if self.has_active_borrows(variable) {
                        return Err(MemorySafetyError::LifetimeViolation(
                            format!("Variable {} goes out of scope while still borrowed", variable)
                        ));
                    }
                },
                OwnershipStatus::Borrowed | OwnershipStatus::MutablyBorrowed => {
                    // When a borrowed variable goes out of scope, we need to release the borrow
                    self.active_borrows.remove(variable);
                    self.active_mutable_borrows.remove(variable);
                },
                OwnershipStatus::Moved => {
                    // Already moved, nothing to do
                },
                OwnershipStatus::Shared => {
                    // For shared values, we might need reference counting
                    // This is a simplified implementation
                },
            }
        }

        self.ownership_map.remove(variable);
        self.lifetime_map.remove(variable);
        Ok(())
    }

    /// Get the ownership status of a variable
    pub fn get_ownership_status(&self, variable: &str) -> Option<&OwnershipStatus> {
        self.ownership_map.get(variable)
    }

    /// Check if a variable is alive (not freed)
    pub fn is_alive(&self, address: &str) -> bool {
        self.allocated_addresses.contains(address) && !self.freed_addresses.contains(address)
    }
}

/// RAII guard for automatic resource management
pub struct RAIIGuard<'a> {
    env: &'a mut MemorySafetyEnv,
    variable: String,
}

impl<'a> RAIIGuard<'a> {
    pub fn new(env: &'a mut MemorySafetyEnv, variable: String) -> Self {
        Self { env, variable }
    }
}

impl<'a> Drop for RAIIGuard<'a> {
    fn drop(&mut self) {
        // Automatically handle cleanup when guard goes out of scope
        let _ = self.env.cleanup_variable(&self.variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ownership() {
        let mut env = MemorySafetyEnv::new();
        
        // Register a variable as owned
        env.register_variable("x".to_string(), OwnershipStatus::Owned)
            .expect("Should register variable successfully");
        
        // Check ownership status
        assert_eq!(env.get_ownership_status("x"), Some(&OwnershipStatus::Owned));
        
        // Transfer ownership
        env.transfer_ownership("x", "y")
            .expect("Should transfer ownership successfully");
        
        assert!(env.get_ownership_status("x").is_none());
        assert_eq!(env.get_ownership_status("y"), Some(&OwnershipStatus::Owned));
    }

    #[test]
    fn test_borrowing_rules() {
        let mut env = MemorySafetyEnv::new();
        
        // Register a variable as owned
        env.register_variable("data".to_string(), OwnershipStatus::Owned)
            .expect("Should register variable successfully");
        
        // Borrow immutably
        env.borrow_immutably("data", "reader1")
            .expect("Should allow immutable borrow");
        
        env.borrow_immutably("data", "reader2")
            .expect("Should allow multiple immutable borrows");
        
        // Try to borrow mutably (should fail)
        assert!(env.borrow_mutably("data", "writer").is_err());
        
        // Release immutable borrows
        env.release_borrow("data", "reader1");
        env.release_borrow("data", "reader2");
        
        // Now we should be able to borrow mutably
        env.borrow_mutably("data", "writer")
            .expect("Should allow mutable borrow when no immutable borrows exist");
    }

    #[test]
    fn test_move_semantics() {
        let mut env = MemorySafetyEnv::new();
        
        // Register a variable as owned
        env.register_variable("owned_data".to_string(), OwnershipStatus::Owned)
            .expect("Should register variable successfully");
        
        // Try to access after move
        env.mark_moved("owned_data")
            .expect("Should mark as moved successfully");
        
        assert!(env.check_access("owned_data").is_err());
    }

    #[test]
    fn test_memory_management() {
        let mut env = MemorySafetyEnv::new();
        
        // Allocate memory
        env.allocate_memory("0x1000".to_string())
            .expect("Should allocate memory successfully");
        
        assert!(env.is_alive("0x1000"));
        
        // Free memory
        env.free_memory("0x1000".to_string())
            .expect("Should free memory successfully");
        
        assert!(!env.is_alive("0x1000"));
        
        // Try to free again (should fail)
        assert!(env.free_memory("0x1000".to_string()).is_err());
    }
}