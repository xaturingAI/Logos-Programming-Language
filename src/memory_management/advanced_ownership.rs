//! Advanced Ownership System for Logos
//! Provides sophisticated ownership tracking and memory management

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipStatus {
    Owned,           // Value is owned by the current scope
    Borrowed,        // Value is borrowed (immutable reference)
    MutablyBorrowed, // Value is mutably borrowed (mutable reference)
    Moved,           // Value has been moved to another owner
    Shared,          // Value is shared among multiple owners (Arc/Rc)
    Linear,          // Value follows linear type semantics (used exactly once)
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnershipInfo {
    pub owner: String,              // Current owner of the value
    pub status: OwnershipStatus,    // Current ownership status
    pub borrow_count: usize,        // Number of active borrows (for reference counting)
    pub lifetime: Option<String>,   // Lifetime annotation if any
    pub linear_usage: Option<bool>, // For linear types - whether it's been used
}

#[derive(Debug)]
pub struct OwnershipSystem {
    /// Tracks ownership of values by their identifiers
    ownership_map: HashMap<String, OwnershipInfo>,

    /// Tracks scopes and their owned values
    scope_ownerships: HashMap<String, HashSet<String>>,

    /// Tracks borrow relationships
    borrow_tracker: HashMap<String, Vec<String>>, // value_id -> [borrower_ids]

    /// Tracks linear type usage
    linear_tracker: HashMap<String, bool>, // value_id -> used_once
}

#[derive(Debug)]
pub enum OwnershipError {
    InvalidOperation(String),
    DoubleBorrow(String),
    UseAfterMove(String),
    InvalidLinearUsage(String),
    CircularReference(String),
}

impl fmt::Display for OwnershipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OwnershipError::InvalidOperation(msg) => write!(f, "Invalid ownership operation: {}", msg),
            OwnershipError::DoubleBorrow(msg) => write!(f, "Double borrow error: {}", msg),
            OwnershipError::UseAfterMove(msg) => write!(f, "Use after move error: {}", msg),
            OwnershipError::InvalidLinearUsage(msg) => write!(f, "Invalid linear usage: {}", msg),
            OwnershipError::CircularReference(msg) => write!(f, "Circular reference error: {}", msg),
        }
    }
}

impl std::error::Error for OwnershipError {}

impl OwnershipSystem {
    /// Creates a new ownership system instance
    pub fn new() -> Self {
        Self {
            ownership_map: HashMap::new(),
            scope_ownerships: HashMap::new(),
            borrow_tracker: HashMap::new(),
            linear_tracker: HashMap::new(),
        }
    }

    /// Registers a new value as owned by the current scope
    pub fn register_owned(&mut self, value_id: &str, owner: &str, lifetime: Option<String>) -> Result<(), OwnershipError> {
        let info = OwnershipInfo {
            owner: owner.to_string(),
            status: OwnershipStatus::Owned,
            borrow_count: 0,
            lifetime,
            linear_usage: None,
        };

        self.ownership_map.insert(value_id.to_string(), info);

        // Track that this scope owns this value
        self.scope_ownerships
            .entry(owner.to_string())
            .or_insert_with(HashSet::new)
            .insert(value_id.to_string());

        Ok(())
    }

    /// Registers a new linear value (must be used exactly once)
    pub fn register_linear(&mut self, value_id: &str, owner: &str) -> Result<(), OwnershipError> {
        let info = OwnershipInfo {
            owner: owner.to_string(),
            status: OwnershipStatus::Linear,
            borrow_count: 0,
            lifetime: None,
            linear_usage: Some(false),
        };

        self.ownership_map.insert(value_id.to_string(), info);
        self.linear_tracker.insert(value_id.to_string(), false);
        
        // Track that this scope owns this value
        self.scope_ownerships
            .entry(owner.to_string())
            .or_insert_with(HashSet::new)
            .insert(value_id.to_string());

        Ok(())
    }

    /// Attempts to borrow a value immutably
    pub fn borrow_immutably(&mut self, value_id: &str, borrower: &str) -> Result<(), OwnershipError> {
        if let Some(info) = self.ownership_map.get_mut(value_id) {
            match info.status {
                OwnershipStatus::Owned | OwnershipStatus::Borrowed => {
                    // Allow immutable borrowing of owned or already borrowed values
                    info.status = OwnershipStatus::Borrowed;
                    info.borrow_count += 1;
                    
                    // Track the borrow relationship
                    self.borrow_tracker
                        .entry(value_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(borrower.to_string());
                    
                    Ok(())
                },
                OwnershipStatus::MutablyBorrowed => {
                    Err(OwnershipError::InvalidOperation(
                        format!("Cannot borrow {} immutably while it's mutably borrowed", value_id)
                    ))
                },
                OwnershipStatus::Moved => {
                    Err(OwnershipError::InvalidOperation(
                        format!("Cannot borrow {} after it has been moved", value_id)
                    ))
                },
                OwnershipStatus::Linear => {
                    if let Some(used) = info.linear_usage {
                        if used {
                            return Err(OwnershipError::InvalidOperation(
                                format!("Linear value {} already used", value_id)
                            ));
                        }
                    }
                    Err(OwnershipError::InvalidOperation(
                        format!("Cannot borrow linear value {}", value_id)
                    ))
                },
                _ => Err(OwnershipError::InvalidOperation(
                    format!("Cannot borrow value {} in current state", value_id)
                ))
            }
        } else {
            Err(OwnershipError::InvalidOperation(
                format!("Value {} not registered in ownership system", value_id)
            ))
        }
    }

    /// Attempts to borrow a value mutably
    pub fn borrow_mutably(&mut self, value_id: &str, borrower: &str) -> Result<(), OwnershipError> {
        if let Some(info) = self.ownership_map.get_mut(value_id) {
            match info.status {
                OwnershipStatus::Owned => {
                    if info.borrow_count == 0 {
                        // Allow mutable borrowing if no other borrows exist
                        info.status = OwnershipStatus::MutablyBorrowed;
                        info.borrow_count = 1; // Count as one borrow
                        
                        // Track the borrow relationship
                        self.borrow_tracker
                            .entry(value_id.to_string())
                            .or_insert_with(Vec::new)
                            .push(borrower.to_string());
                        
                        Ok(())
                    } else {
                        Err(OwnershipError::DoubleBorrow(
                            format!("Cannot mutably borrow {} while it's already borrowed", value_id)
                        ))
                    }
                },
                OwnershipStatus::Borrowed => {
                    Err(OwnershipError::DoubleBorrow(
                        format!("Cannot mutably borrow {} while it's immutably borrowed", value_id)
                    ))
                },
                OwnershipStatus::MutablyBorrowed => {
                    Err(OwnershipError::DoubleBorrow(
                        format!("Cannot mutably borrow {} while it's already mutably borrowed", value_id)
                    ))
                },
                OwnershipStatus::Moved => {
                    Err(OwnershipError::UseAfterMove(
                        format!("Cannot borrow {} after it has been moved", value_id)
                    ))
                },
                OwnershipStatus::Linear => {
                    if let Some(used) = info.linear_usage {
                        if used {
                            return Err(OwnershipError::UseAfterMove(
                                format!("Linear value {} already used", value_id)
                            ));
                        }
                    }
                    Err(OwnershipError::InvalidLinearUsage(
                        format!("Cannot mutably borrow linear value {}", value_id)
                    ))
                },
                _ => Err(OwnershipError::InvalidOperation(
                    format!("Cannot mutably borrow value {} in current state", value_id)
                ))
            }
        } else {
            Err(OwnershipError::InvalidOperation(
                format!("Value {} not registered in ownership system", value_id)
            ))
        }
    }

    /// Moves ownership of a value to a new owner
    pub fn move_ownership(&mut self, value_id: &str, new_owner: &str) -> Result<(), OwnershipError> {
        if let Some(info) = self.ownership_map.get_mut(value_id) {
            match info.status {
                OwnershipStatus::Owned | OwnershipStatus::Borrowed => {
                    if info.borrow_count == 0 {
                        // Can only move if not borrowed
                        info.status = OwnershipStatus::Moved;
                        let old_owner = info.owner.clone();
                        info.owner = new_owner.to_string();
                        
                        // Update scope ownership tracking
                        if let Some(owner_values) = self.scope_ownerships.get_mut(&old_owner) {
                            owner_values.remove(value_id);
                        }
                        
                        self.scope_ownerships
                            .entry(new_owner.to_string())
                            .or_insert_with(HashSet::new)
                            .insert(value_id.to_string());
                        
                        Ok(())
                    } else {
                        Err(OwnershipError::InvalidOperation(
                            format!("Cannot move {} while it's borrowed", value_id)
                        ))
                    }
                },
                OwnershipStatus::Linear => {
                    if let Some(used) = info.linear_usage {
                        if used {
                            return Err(OwnershipError::UseAfterMove(
                                format!("Linear value {} already used", value_id)
                            ));
                        }
                    }
                    // Mark as used and move
                    info.linear_usage = Some(true);
                    info.status = OwnershipStatus::Moved;
                    let old_owner = info.owner.clone();
                    info.owner = new_owner.to_string();
                    
                    // Update scope ownership tracking
                    if let Some(owner_values) = self.scope_ownerships.get_mut(&old_owner) {
                        owner_values.remove(value_id);
                    }
                    
                    self.scope_ownerships
                        .entry(new_owner.to_string())
                        .or_insert_with(HashSet::new)
                        .insert(value_id.to_string());
                    
                    Ok(())
                },
                _ => Err(OwnershipError::InvalidOperation(
                    format!("Cannot move value {} in current state", value_id)
                ))
            }
        } else {
            Err(OwnershipError::InvalidOperation(
                format!("Value {} not registered in ownership system", value_id)
            ))
        }
    }

    /// Releases a value from the ownership system
    pub fn release(&mut self, value_id: &str) -> Result<(), OwnershipError> {
        // First, check if the value exists and get its info
        let info_clone = match self.ownership_map.get(value_id) {
            Some(info) => info.clone(),
            None => {
                return Err(OwnershipError::InvalidOperation(
                    format!("Value {} not registered in ownership system", value_id)
                ));
            }
        };

        if info_clone.borrow_count == 0 {
            // Remove from ownership map
            self.ownership_map.remove(value_id);

            // Remove from scope tracking
            if let Some(owner_values) = self.scope_ownerships.get_mut(&info_clone.owner) {
                owner_values.remove(value_id);
            }

            // Remove from borrow tracker
            self.borrow_tracker.remove(value_id);

            // Remove from linear tracker if it was linear
            self.linear_tracker.remove(value_id);

            Ok(())
        } else {
            Err(OwnershipError::InvalidOperation(
                format!("Cannot release {} while it's still borrowed", value_id)
            ))
        }
    }

    /// Checks if a value can be accessed (not moved or linearly used)
    pub fn can_access(&self, value_id: &str) -> bool {
        if let Some(info) = self.ownership_map.get(value_id) {
            match info.status {
                OwnershipStatus::Moved => false,
                OwnershipStatus::Linear => {
                    if let Some(used) = info.linear_usage {
                        !used
                    } else {
                        false // Linear values without usage tracking are considered used
                    }
                },
                _ => true,
            }
        } else {
            false // Value not in system
        }
    }

    /// Gets the current owner of a value
    pub fn get_owner(&self, value_id: &str) -> Option<String> {
        self.ownership_map.get(value_id).map(|info| info.owner.clone())
    }

    /// Gets the ownership status of a value
    pub fn get_status(&self, value_id: &str) -> Option<OwnershipStatus> {
        self.ownership_map.get(value_id).map(|info| info.status.clone())
    }

    /// Gets the number of active borrows for a value
    pub fn get_borrow_count(&self, value_id: &str) -> usize {
        self.ownership_map.get(value_id).map(|info| info.borrow_count).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ownership() {
        let mut system = OwnershipSystem::new();
        
        // Register a value as owned
        assert!(system.register_owned("var1", "scope1", None).is_ok());
        
        // Check ownership
        assert_eq!(system.get_owner("var1"), Some("scope1".to_string()));
        assert_eq!(system.get_status("var1"), Some(OwnershipStatus::Owned));
        
        // Borrow immutably
        assert!(system.borrow_immutably("var1", "borrower1").is_ok());
        assert_eq!(system.get_status("var1"), Some(OwnershipStatus::Borrowed));
        assert_eq!(system.get_borrow_count("var1"), 1);
        
        // Release the value
        assert!(system.release("var1").is_err()); // Should fail because it's borrowed
    }

    #[test]
    fn test_linear_ownership() {
        let mut system = OwnershipSystem::new();
        
        // Register a linear value
        assert!(system.register_linear("linear_var", "scope1").is_ok());
        
        // Check that it's linear
        assert_eq!(system.get_status("linear_var"), Some(OwnershipStatus::Linear));
        
        // Try to access before use
        assert!(system.can_access("linear_var"));
        
        // Move the linear value
        assert!(system.move_ownership("linear_var", "scope2").is_ok());
        
        // Check that it can't be accessed anymore
        assert!(!system.can_access("linear_var"));
    }
}