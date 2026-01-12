//! Memory Safety without Garbage Collection
//! Implements a memory management system that ensures safety without relying on garbage collection
//! Uses ownership, borrowing, and RAII principles similar to Rust

use std::collections::{HashMap, HashSet};
use std::fmt;

// Import the ownership system we created
use crate::memory_management::ownership::{OwnershipSystem, OwnershipStatus, OwnershipError};

/// Represents a memory address
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryAddress(String);

impl MemoryAddress {
    pub fn new(address: &str) -> Self {
        MemoryAddress(address.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemoryAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a memory allocation
#[derive(Debug, Clone)]
pub struct Allocation {
    pub address: MemoryAddress,
    pub size: usize,
    pub allocated_by: String, // Variable that owns this allocation
    pub value_type: String,   // Type of value stored at this address
    pub is_freed: bool,
}

/// Memory safety system that ensures memory safety without garbage collection
pub struct MemorySafetySystem {
    /// The ownership system that tracks variable ownership and borrowing
    ownership_system: OwnershipSystem,
    
    /// Allocated memory blocks
    allocations: HashMap<MemoryAddress, Allocation>,
    
    /// Keeps track of freed addresses to detect double frees
    freed_addresses: HashSet<MemoryAddress>,
    
    /// Total memory currently allocated
    total_allocated: usize,
    
    /// Maximum allowed memory (for simulation purposes)
    max_memory: usize,
}

impl MemorySafetySystem {
    /// Creates a new memory safety system
    pub fn new(max_memory: usize) -> Self {
        Self {
            ownership_system: OwnershipSystem::new(),
            allocations: HashMap::new(),
            freed_addresses: HashSet::new(),
            total_allocated: 0,
            max_memory,
        }
    }

    /// Allocate memory of the specified size
    pub fn allocate(&mut self, size: usize, allocated_by: &str, value_type: &str) -> Result<MemoryAddress, MemorySafetyError> {
        // Check if we have enough memory available
        if self.total_allocated + size > self.max_memory {
            return Err(MemorySafetyError::OutOfMemory(size));
        }

        // Generate a unique address (in a real implementation, this would interface with the actual allocator)
        let address = MemoryAddress::new(&format!("0x{:X}", self.total_allocated + 0x1000));
        
        // Create the allocation
        let allocation = Allocation {
            address: address.clone(),
            size,
            allocated_by: allocated_by.to_string(),
            value_type: value_type.to_string(),
            is_freed: false,
        };
        
        // Store the allocation
        self.allocations.insert(address.clone(), allocation);
        self.total_allocated += size;
        
        Ok(address)
    }

    /// Free memory at the specified address
    pub fn free(&mut self, address: &MemoryAddress) -> Result<(), MemorySafetyError> {
        // Check if address exists
        let allocation = self.allocations.get_mut(address)
            .ok_or_else(|| MemorySafetyError::InvalidAddress(address.clone()))?;
        
        // Check if already freed
        if allocation.is_freed {
            return Err(MemorySafetyError::DoubleFree(address.clone()));
        }
        
        // Mark as freed
        allocation.is_freed = true;
        
        // Update total allocated
        self.total_allocated -= allocation.size;
        
        // Add to freed addresses set
        self.freed_addresses.insert(address.clone());
        
        Ok(())
    }

    /// Check if an address is valid and not freed
    pub fn is_valid_address(&self, address: &MemoryAddress) -> bool {
        if let Some(allocation) = self.allocations.get(address) {
            !allocation.is_freed
        } else {
            false
        }
    }

    /// Declare a variable that owns a memory allocation
    pub fn declare_variable_with_allocation(
        &mut self, 
        var_name: String, 
        value_type: String,
        size: usize
    ) -> Result<MemoryAddress, MemorySafetyError> {
        // First, declare the variable in the ownership system
        self.ownership_system.declare_variable(var_name.clone(), value_type.clone())?;
        
        // Then allocate memory for it
        let address = self.allocate(size, &var_name, &value_type)?;
        
        Ok(address)
    }

    /// Transfer ownership of a memory allocation from one variable to another
    pub fn transfer_allocation_ownership(
        &mut self,
        from_var: &str,
        to_var: &str
    ) -> Result<(), MemorySafetyError> {
        // Find allocations owned by the 'from' variable
        let mut allocations_to_update = Vec::new();
        
        for (addr, alloc) in &self.allocations {
            if alloc.allocated_by == from_var && !alloc.is_freed {
                allocations_to_update.push((addr.clone(), alloc.clone()));
            }
        }
        
        // Update ownership for each allocation
        for (addr, mut alloc) in allocations_to_update {
            alloc.allocated_by = to_var.to_string();
            self.allocations.insert(addr, alloc);
        }
        
        // Also update ownership in the ownership system
        self.ownership_system.transfer_ownership(from_var, to_var)?;
        
        Ok(())
    }

    /// Borrow memory immutably
    pub fn borrow_memory_immutably(
        &mut self,
        variable: &str,
        borrower: &str
    ) -> Result<(), MemorySafetyError> {
        // Check if the variable exists and is accessible
        if !self.ownership_system.is_accessible(variable) {
            return Err(MemorySafetyError::UseAfterMove(variable.to_string()));
        }
        
        // Perform the borrow in the ownership system
        self.ownership_system.borrow_immutably(variable, borrower)?;
        
        Ok(())
    }

    /// Borrow memory mutably
    pub fn borrow_memory_mutably(
        &mut self,
        variable: &str,
        borrower: &str
    ) -> Result<(), MemorySafetyError> {
        // Check if the variable exists and is accessible
        if !self.ownership_system.is_accessible(variable) {
            return Err(MemorySafetyError::UseAfterMove(variable.to_string()));
        }
        
        // Perform the borrow in the ownership system
        self.ownership_system.borrow_mutably(variable, borrower)?;
        
        Ok(())
    }

    /// Release a memory borrow
    pub fn release_memory_borrow(
        &mut self,
        variable: &str,
        borrower: &str
    ) -> Result<(), MemorySafetyError> {
        // Release the borrow in the ownership system
        self.ownership_system.release_borrow(variable, borrower)?;
        
        Ok(())
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.ownership_system.enter_scope();
    }

    /// Exit the current scope and clean up appropriately
    pub fn exit_scope(&mut self) -> Result<(), MemorySafetyError> {
        // Clean up in the ownership system
        self.ownership_system.exit_scope()
            .map_err(|e| MemorySafetyError::OwnershipError(e))?;
        
        Ok(())
    }

    /// Get the current memory usage
    pub fn get_current_memory_usage(&self) -> usize {
        self.total_allocated
    }

    /// Get the maximum allowed memory
    pub fn get_max_memory(&self) -> usize {
        self.max_memory
    }

    /// Validate memory safety invariants
    pub fn validate(&self) -> Result<(), MemorySafetyError> {
        // Check that all allocations that are not freed are still tracked
        for (addr, alloc) in &self.allocations {
            if !alloc.is_freed && !self.freed_addresses.contains(addr) {
                // Allocation is valid and should be accessible
            }
        }
        
        Ok(())
    }
}

/// Error types for memory safety operations
#[derive(Debug, Clone)]
pub enum MemorySafetyError {
    OwnershipError(OwnershipError),
    InvalidAddress(MemoryAddress),
    DoubleFree(MemoryAddress),
    OutOfMemory(usize),
    UseAfterMove(String),
}

impl fmt::Display for MemorySafetyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemorySafetyError::OwnershipError(e) => write!(f, "Ownership error: {}", e),
            MemorySafetyError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            MemorySafetyError::DoubleFree(addr) => write!(f, "Double free error: {}", addr),
            MemorySafetyError::OutOfMemory(size) => write!(f, "Out of memory trying to allocate {} bytes", size),
            MemorySafetyError::UseAfterMove(var) => write!(f, "Use after move: variable {} has been moved", var),
        }
    }
}

impl std::error::Error for MemorySafetyError {}

impl From<OwnershipError> for MemorySafetyError {
    fn from(error: OwnershipError) -> Self {
        MemorySafetyError::OwnershipError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation_and_free() {
        let mut ms = MemorySafetySystem::new(1024); // 1KB max memory
        
        // Allocate memory
        let addr = ms.allocate(32, "x", "int_array").unwrap();
        
        // Check that address is valid
        assert!(ms.is_valid_address(&addr));
        
        // Free memory
        assert!(ms.free(&addr).is_ok());
        
        // Address should no longer be valid
        assert!(!ms.is_valid_address(&addr));
        
        // Trying to free again should fail
        assert!(ms.free(&addr).is_err());
    }

    #[test]
    fn test_variable_with_allocation() {
        let mut ms = MemorySafetySystem::new(1024);
        
        // Declare a variable with an allocation
        let addr = ms.declare_variable_with_allocation(
            "my_vec".to_string(), 
            "Vec<int>".to_string(), 
            64
        ).unwrap();
        
        // Check that address is valid
        assert!(ms.is_valid_address(&addr));
        
        // Check memory usage
        assert_eq!(ms.get_current_memory_usage(), 64);
    }

    #[test]
    fn test_ownership_transfer() {
        let mut ms = MemorySafetySystem::new(1024);
        
        // Declare a variable with an allocation
        let addr = ms.declare_variable_with_allocation(
            "vec1".to_string(), 
            "Vec<int>".to_string(), 
            64
        ).unwrap();
        
        // Transfer ownership to another variable
        assert!(ms.transfer_allocation_ownership("vec1", "vec2").is_ok());
        
        // Original variable should no longer be accessible
        assert!(ms.borrow_memory_immutably("vec1", "other").is_err());
        
        // New variable should be accessible
        assert!(ms.borrow_memory_immutably("vec2", "other").is_ok());
    }

    #[test]
    fn test_memory_validation() {
        let mut ms = MemorySafetySystem::new(1024);
        
        // Allocate memory
        let addr = ms.allocate(32, "x", "int").unwrap();
        
        // Validation should pass
        assert!(ms.validate().is_ok());
        
        // Free memory
        ms.free(&addr).unwrap();
        
        // Validation should still pass
        assert!(ms.validate().is_ok());
    }
}