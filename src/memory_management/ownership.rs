//! Ownership and Borrowing System for Logos
//! Implements Rust-like ownership and borrowing semantics for memory safety without garbage collection

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents the ownership status of a value
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipStatus {
    Owned,           // The value is owned by a variable
    Borrowed,        // The value is borrowed immutably
    MutablyBorrowed, // The value is borrowed mutably
    Moved,           // The value has been moved to another owner
}

/// Represents a variable in the memory management system
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub ownership_status: OwnershipStatus,
    pub value_type: String, // Type of the value stored
    pub scope_level: usize, // Nesting level of the scope where the variable is defined
}

/// Represents a borrow relationship
#[derive(Debug, Clone)]
pub struct BorrowRecord {
    pub borrower: String,      // Name of the variable that is borrowing
    pub borrowed_at: usize,    // Scope level where the borrow happened
    pub is_mutable: bool,      // Whether the borrow is mutable
}

/// Main memory management system that enforces ownership and borrowing rules
pub struct OwnershipSystem {
    /// Map of variables to their ownership status
    variables: HashMap<String, Variable>,
    
    /// Map of variables to their active borrows
    active_borrows: HashMap<String, Vec<BorrowRecord>>,
    
    /// Stack of scopes to track variable lifetimes
    scopes: Vec<HashSet<String>>,
    
    /// Current scope level
    current_scope: usize,
}

impl OwnershipSystem {
    /// Creates a new ownership system
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            active_borrows: HashMap::new(),
            scopes: vec![HashSet::new()],
            current_scope: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
        self.scopes.push(HashSet::new());
    }

    /// Exit the current scope and clean up variables
    pub fn exit_scope(&mut self) -> Result<(), OwnershipError> {
        if self.current_scope == 0 {
            return Err(OwnershipError::InvalidScopeOperation("Cannot exit global scope".to_string()));
        }

        // Get the variables in the current scope
        let current_scope_vars = self.scopes.pop().unwrap();
        
        // Check for any remaining borrows before removing variables
        for var_name in &current_scope_vars {
            if self.has_active_borrows(var_name) {
                return Err(OwnershipError::BorrowAfterScopeExit(var_name.clone()));
            }
            
            // Remove the variable
            self.variables.remove(var_name);
        }
        
        self.current_scope -= 1;
        
        Ok(())
    }

    /// Declare a new variable with ownership
    pub fn declare_variable(&mut self, name: String, value_type: String) -> Result<(), OwnershipError> {
        // Check if variable already exists in current scope
        if self.variables.contains_key(&name) {
            return Err(OwnershipError::VariableAlreadyExists(name));
        }

        // Create the variable with owned status
        let variable = Variable {
            name: name.clone(),
            ownership_status: OwnershipStatus::Owned,
            value_type,
            scope_level: self.current_scope,
        };

        // Add to variables map
        self.variables.insert(name.clone(), variable);

        // Add to current scope
        self.scopes.last_mut().unwrap().insert(name);

        Ok(())
    }

    /// Borrow a variable immutably
    pub fn borrow_immutably(&mut self, variable: &str, borrower: &str) -> Result<(), OwnershipError> {
        // Check if the variable exists
        let var = self.variables.get(variable)
            .ok_or_else(|| OwnershipError::VariableNotFound(variable.to_string()))?;

        // Check if the variable is owned or borrowed immutably (can have multiple immutable borrows)
        match var.ownership_status {
            OwnershipStatus::Owned => {
                // Check if there are any mutable borrows
                if self.has_mutable_borrows(variable) {
                    return Err(OwnershipError::InvalidBorrow(format!(
                        "Cannot borrow {} immutably while it's mutably borrowed", variable
                    )));
                }

                // Add the immutable borrow
                self.add_borrow_record(variable, borrower, false)?;
            },
            OwnershipStatus::Borrowed => {
                // Already borrowed immutably, add another immutable borrow
                self.add_borrow_record(variable, borrower, false)?;
            },
            OwnershipStatus::MutablyBorrowed => {
                return Err(OwnershipError::InvalidBorrow(format!(
                    "Cannot borrow {} immutably while it's mutably borrowed", variable
                )));
            },
            OwnershipStatus::Moved => {
                return Err(OwnershipError::UseAfterMove(variable.to_string()));
            }
        }

        Ok(())
    }

    /// Borrow a variable mutably
    pub fn borrow_mutably(&mut self, variable: &str, borrower: &str) -> Result<(), OwnershipError> {
        // Check if the variable exists
        let var = self.variables.get(variable)
            .ok_or_else(|| OwnershipError::VariableNotFound(variable.to_string()))?;

        // Check if the variable is owned and not already borrowed
        match var.ownership_status {
            OwnershipStatus::Owned => {
                // Check if there are any active borrows (mutable or immutable)
                if self.has_active_borrows(variable) {
                    return Err(OwnershipError::InvalidBorrow(format!(
                        "Cannot mutably borrow {} while it's already borrowed", variable
                    )));
                }

                // Add the mutable borrow
                self.add_borrow_record(variable, borrower, true)?;
            },
            OwnershipStatus::Borrowed => {
                return Err(OwnershipError::InvalidBorrow(format!(
                    "Cannot mutably borrow {} while it's already immutably borrowed", variable
                )));
            },
            OwnershipStatus::MutablyBorrowed => {
                return Err(OwnershipError::InvalidBorrow(format!(
                    "Cannot mutably borrow {} while it's already mutably borrowed", variable
                )));
            },
            OwnershipStatus::Moved => {
                return Err(OwnershipError::UseAfterMove(variable.to_string()));
            }
        }

        Ok(())
    }

    /// Release a borrow
    pub fn release_borrow(&mut self, variable: &str, borrower: &str) -> Result<(), OwnershipError> {
        if let Some(borrows) = self.active_borrows.get_mut(variable) {
            borrows.retain(|record| record.borrower != borrower);
            
            // If no more borrows, remove the entry
            if borrows.is_empty() {
                self.active_borrows.remove(variable);
            }
        }

        Ok(())
    }

    /// Transfer ownership of a variable to another
    pub fn transfer_ownership(&mut self, from: &str, to: &str) -> Result<(), OwnershipError> {
        // Check if 'from' variable exists
        let mut var = self.variables.get(from)
            .ok_or_else(|| OwnershipError::VariableNotFound(from.to_string()))?
            .clone();

        // Check if the variable is owned (not borrowed)
        if var.ownership_status != OwnershipStatus::Owned {
            return Err(OwnershipError::OwnershipTransferError(
                format!("Cannot transfer ownership of non-owned variable: {}", from)
            ));
        }

        // Check if 'from' has any active borrows
        if self.has_active_borrows(from) {
            return Err(OwnershipError::OwnershipTransferError(
                format!("Cannot transfer ownership of borrowed variable: {}", from)
            ));
        }

        // Update the original variable to moved status
        var.ownership_status = OwnershipStatus::Moved;
        self.variables.insert(from.to_string(), var);

        // Create the new variable with ownership
        let new_var = Variable {
            name: to.to_string(),
            ownership_status: OwnershipStatus::Owned,
            value_type: self.variables.get(from)
                .map(|v| v.value_type.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            scope_level: self.current_scope,
        };

        self.variables.insert(to.to_string(), new_var);

        Ok(())
    }

    /// Check if a variable has any active borrows
    fn has_active_borrows(&self, variable: &str) -> bool {
        self.active_borrows.contains_key(variable)
    }

    /// Check if a variable has any mutable borrows
    fn has_mutable_borrows(&self, variable: &str) -> bool {
        if let Some(borrows) = self.active_borrows.get(variable) {
            borrows.iter().any(|record| record.is_mutable)
        } else {
            false
        }
    }

    /// Add a borrow record
    fn add_borrow_record(&mut self, variable: &str, borrower: &str, is_mutable: bool) -> Result<(), OwnershipError> {
        let borrow_record = BorrowRecord {
            borrower: borrower.to_string(),
            borrowed_at: self.current_scope,
            is_mutable,
        };

        self.active_borrows
            .entry(variable.to_string())
            .or_insert_with(Vec::new)
            .push(borrow_record);

        Ok(())
    }

    /// Check if a variable is accessible (owned or borrowed)
    pub fn is_accessible(&self, variable: &str) -> bool {
        if let Some(var) = self.variables.get(variable) {
            match var.ownership_status {
                OwnershipStatus::Moved => false,
                _ => true,
            }
        } else {
            false
        }
    }

    /// Get the ownership status of a variable
    pub fn get_ownership_status(&self, variable: &str) -> Option<&OwnershipStatus> {
        self.variables.get(variable).map(|var| &var.ownership_status)
    }
}

/// Error types for ownership operations
#[derive(Debug, Clone)]
pub enum OwnershipError {
    VariableNotFound(String),
    VariableAlreadyExists(String),
    InvalidBorrow(String),
    UseAfterMove(String),
    OwnershipTransferError(String),
    InvalidScopeOperation(String),
    BorrowAfterScopeExit(String),
}

impl fmt::Display for OwnershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnershipError::VariableNotFound(var) => write!(f, "Variable not found: {}", var),
            OwnershipError::VariableAlreadyExists(var) => write!(f, "Variable already exists: {}", var),
            OwnershipError::InvalidBorrow(desc) => write!(f, "Invalid borrow: {}", desc),
            OwnershipError::UseAfterMove(var) => write!(f, "Use after move: variable {} has been moved", var),
            OwnershipError::OwnershipTransferError(desc) => write!(f, "Ownership transfer error: {}", desc),
            OwnershipError::InvalidScopeOperation(desc) => write!(f, "Invalid scope operation: {}", desc),
            OwnershipError::BorrowAfterScopeExit(var) => write!(f, "Borrow after scope exit: variable {} has active borrows after scope ended", var),
        }
    }
}

impl std::error::Error for OwnershipError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_declaration() {
        let mut os = OwnershipSystem::new();
        
        // Should be able to declare a variable
        assert!(os.declare_variable("x".to_string(), "int".to_string()).is_ok());
        
        // Should not be able to declare the same variable again
        assert!(os.declare_variable("x".to_string(), "int".to_string()).is_err());
    }

    #[test]
    fn test_ownership_transfer() {
        let mut os = OwnershipSystem::new();
        
        // Declare a variable
        os.declare_variable("x".to_string(), "int".to_string()).unwrap();
        
        // Transfer ownership to another variable
        assert!(os.transfer_ownership("x", "y").is_ok());
        
        // Original variable should no longer be accessible
        assert!(!os.is_accessible("x"));
        
        // New variable should be owned
        assert_eq!(os.get_ownership_status("y"), Some(&OwnershipStatus::Owned));
    }

    #[test]
    fn test_immutable_borrowing() {
        let mut os = OwnershipSystem::new();
        
        // Declare a variable
        os.declare_variable("x".to_string(), "int".to_string()).unwrap();
        
        // Should be able to borrow immutably
        assert!(os.borrow_immutably("x", "a").is_ok());
        assert!(os.borrow_immutably("x", "b").is_ok()); // Multiple immutable borrows allowed
        
        // Should not be able to borrow mutably while immutably borrowed
        assert!(os.borrow_mutably("x", "c").is_err());
    }

    #[test]
    fn test_mutable_borrowing() {
        let mut os = OwnershipSystem::new();
        
        // Declare a variable
        os.declare_variable("x".to_string(), "int".to_string()).unwrap();
        
        // Should be able to borrow mutably
        assert!(os.borrow_mutably("x", "a").is_ok());
        
        // Should not be able to borrow again while mutably borrowed
        assert!(os.borrow_immutably("x", "b").is_err());
        assert!(os.borrow_mutably("x", "c").is_err());
    }

    #[test]
    fn test_scoping() {
        let mut os = OwnershipSystem::new();
        
        // Declare a variable in global scope
        os.declare_variable("x".to_string(), "int".to_string()).unwrap();
        
        // Enter a new scope
        os.enter_scope();
        
        // Declare a variable in inner scope
        os.declare_variable("y".to_string(), "int".to_string()).unwrap();
        
        // Exit the scope
        assert!(os.exit_scope().is_ok());
        
        // Variable from inner scope should no longer exist
        assert!(!os.is_accessible("y"));
        
        // Variable from outer scope should still exist
        assert!(os.is_accessible("x"));
    }
}