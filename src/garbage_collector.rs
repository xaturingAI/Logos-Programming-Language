//! Enhanced garbage collector with safety checks for Eux
//! Implements tracing garbage collection with memory safety validation

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::memory_safety::{MemorySafetyEnv, MemorySafetyError, OwnershipStatus};

#[derive(Debug, Clone)]
pub enum ObjectType {
    Integer,
    Float,
    String,
    Boolean,
    Array,
    Function,
    Closure,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Object {
    pub address: String,
    pub obj_type: ObjectType,
    pub size: usize,
    pub references: Vec<String>, // Addresses of objects this object references
    pub marked: bool,
    pub ownership: OwnershipStatus,
    pub lifetime_region: String,
}

impl Object {
    pub fn new(address: String, obj_type: ObjectType, size: usize) -> Self {
        Self {
            address,
            obj_type,
            size,
            references: Vec::new(),
            marked: false,
            ownership: OwnershipStatus::Owned,
            lifetime_region: "unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct GarbageCollector {
    objects: Arc<Mutex<HashMap<String, Object>>>,
    root_set: Arc<Mutex<HashSet<String>>>, // Objects accessible from global scope
    safety_env: Arc<Mutex<MemorySafetyEnv>>,
    heap_size: AtomicUsize,
    max_heap_size: usize,
}

impl GarbageCollector {
    pub fn new(max_heap_size: usize) -> Self {
        Self {
            objects: Arc::new(Mutex::new(HashMap::new())),
            root_set: Arc::new(Mutex::new(HashSet::new())),
            safety_env: Arc::new(Mutex::new(MemorySafetyEnv::new())),
            heap_size: AtomicUsize::new(0),
            max_heap_size,
        }
    }

    /// Register a new object in the garbage collector
    pub fn register_object(&self, address: String, obj_type: ObjectType, size: usize) -> Result<(), MemorySafetyError> {
        let mut objects = self.objects.lock().unwrap();
        let mut safety_env = self.safety_env.lock().unwrap();

        // Check if address is already registered
        if objects.contains_key(&address) {
            return Err(MemorySafetyError::DoubleFree(address));
        }

        // Track in memory safety environment
        safety_env.allocate_memory(address.clone())?;

        let obj = Object::new(address.clone(), obj_type, size);
        objects.insert(address, obj);
        self.heap_size.fetch_add(size, Ordering::SeqCst);

        Ok(())
    }

    /// Update object references
    pub fn update_references(&self, from_address: String, to_addresses: Vec<String>) -> Result<(), MemorySafetyError> {
        let mut objects = self.objects.lock().unwrap();

        if let Some(obj) = objects.get_mut(&from_address) {
            obj.references = to_addresses;
            Ok(())
        } else {
            Err(MemorySafetyError::UseAfterFree(from_address))
        }
    }

    /// Add an object to the root set (globally accessible)
    pub fn add_to_root_set(&self, address: String) -> Result<(), MemorySafetyError> {
        let mut root_set = self.root_set.lock().unwrap();

        if !self.object_exists(&address) {
            return Err(MemorySafetyError::UseAfterFree(address));
        }

        root_set.insert(address);
        Ok(())
    }

    /// Mark phase of mark-and-sweep algorithm
    fn mark_phase(&self) -> Result<(), MemorySafetyError> {
        let mut objects = self.objects.lock().unwrap();
        let root_set = self.root_set.lock().unwrap();

        // Reset all marks
        for obj in objects.values_mut() {
            obj.marked = false;
        }

        // Mark all reachable objects from root set
        for root_addr in root_set.iter() {
            self.mark_object(root_addr, &mut objects)?;
        }

        Ok(())
    }

    /// Mark an object and all objects it references
    fn mark_object(&self, addr: &str, objects: &mut HashMap<String, Object>) -> Result<(), MemorySafetyError> {
        // First, check if the object exists and is not already marked
        if let Some(obj) = objects.get(addr) {
            if obj.marked {
                return Ok(()); // Already marked
            }
        } else {
            return Err(MemorySafetyError::UseAfterFree(addr.to_string()));
        }

        // Now get a mutable reference and mark it
        if let Some(obj) = objects.get_mut(addr) {
            obj.marked = true;

            // Create a copy of the references to avoid borrowing issues
            let refs_to_mark: Vec<String> = obj.references.clone();

            // Mark all referenced objects
            for ref_addr in refs_to_mark {
                self.mark_object(&ref_addr, objects)?;
            }
        }

        Ok(())
    }

    /// Sweep phase of mark-and-sweep algorithm
    fn sweep_phase(&self) -> Result<usize, MemorySafetyError> {
        let mut objects = self.objects.lock().unwrap();
        let mut safety_env = self.safety_env.lock().unwrap();
        let mut root_set = self.root_set.lock().unwrap();
        let mut freed_count = 0;
        let mut freed_size = 0;

        let to_remove: Vec<String> = objects
            .iter()
            .filter(|(_, obj)| !obj.marked) // Unmarked objects are unreachable
            .map(|(addr, _)| addr.clone())
            .collect();

        for addr in to_remove {
            if let Some(obj) = objects.remove(&addr) {
                // Update heap size
                freed_size += obj.size;
                self.heap_size.fetch_sub(obj.size, Ordering::SeqCst);
                
                // Track in safety environment
                safety_env.free_memory(addr.clone())?;
                
                // Remove from root set if present
                root_set.remove(&addr);
                
                freed_count += 1;
            }
        }

        Ok(freed_count)
    }

    /// Perform garbage collection
    pub fn collect_garbage(&self) -> Result<usize, MemorySafetyError> {
        self.mark_phase()?;
        let freed_count = self.sweep_phase()?;
        Ok(freed_count)
    }

    /// Check if we should run garbage collection
    pub fn should_collect(&self) -> bool {
        self.heap_size.load(Ordering::SeqCst) > self.max_heap_size * 7 / 10 // Collect when 70% full
    }

    /// Check if an object exists
    fn object_exists(&self, addr: &str) -> bool {
        let objects = self.objects.lock().unwrap();
        objects.contains_key(addr)
    }

    /// Get object size
    pub fn get_object_size(&self, addr: &str) -> Option<usize> {
        let objects = self.objects.lock().unwrap();
        objects.get(addr).map(|obj| obj.size)
    }

    /// Get current heap usage
    pub fn get_heap_usage(&self) -> usize {
        self.heap_size.load(Ordering::SeqCst)
    }

    /// Get total number of objects
    pub fn get_object_count(&self) -> usize {
        let objects = self.objects.lock().unwrap();
        objects.len()
    }

    /// Validate memory safety invariants
    pub fn validate_safety(&self) -> Result<(), MemorySafetyError> {
        let objects = self.objects.lock().unwrap();
        let safety_env = self.safety_env.lock().unwrap();

        // Check that all objects in the GC are also tracked in the safety environment
        for (addr, _) in &*objects {
            if !safety_env.is_alive(addr) {
                return Err(MemorySafetyError::UseAfterFree(addr.clone()));
            }
        }

        Ok(())
    }

    /// Transfer ownership of an object
    pub fn transfer_ownership(&self, addr: &str, new_owner: &str) -> Result<(), MemorySafetyError> {
        let mut objects = self.objects.lock().unwrap();

        if let Some(obj) = objects.get_mut(addr) {
            // In a real implementation, we'd update ownership tracking
            // For now, just validate that the object exists
            Ok(())
        } else {
            Err(MemorySafetyError::UseAfterFree(addr.to_string()))
        }
    }

    /// Create a weak reference to an object
    pub fn create_weak_ref(&self, addr: &str) -> Result<String, MemorySafetyError> {
        if !self.object_exists(addr) {
            return Err(MemorySafetyError::UseAfterFree(addr.to_string()));
        }

        // In a real implementation, we'd create a weak reference
        // For now, just return the address
        Ok(format!("weak_{}", addr))
    }

    /// Check if an object is still alive
    pub fn is_alive(&self, addr: &str) -> bool {
        let objects = self.objects.lock().unwrap();
        objects.contains_key(addr) && objects[addr].marked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_gc_operations() {
        let gc = GarbageCollector::new(1024 * 1024); // 1MB max heap

        // Register some objects
        gc.register_object("0x1000".to_string(), ObjectType::Integer, 8)
            .expect("Should register object successfully");

        gc.register_object("0x1008".to_string(), ObjectType::String, 32)
            .expect("Should register object successfully");

        // Add one to root set
        gc.add_to_root_set("0x1000".to_string())
            .expect("Should add to root set successfully");

        // Update references
        gc.update_references("0x1000".to_string(), vec!["0x1008".to_string()])
            .expect("Should update references successfully");

        // Check heap usage
        assert!(gc.get_heap_usage() > 0);

        // Validate safety
        gc.validate_safety().expect("Should validate safety successfully");

        // Check object existence
        assert!(gc.object_exists("0x1000"));
        assert!(gc.object_exists("0x1008"));

        // Check if objects are alive
        assert!(gc.is_alive("0x1000")); // This should be alive as it's in root set
        // Note: 0x1008 might not be marked yet since we haven't run GC
    }

    #[test]
    fn test_garbage_collection() {
        let gc = GarbageCollector::new(1024 * 1024); // 1MB max heap

        // Register objects
        gc.register_object("0x2000".to_string(), ObjectType::Integer, 8)
            .expect("Should register object successfully");

        gc.register_object("0x2008".to_string(), ObjectType::Integer, 8)
            .expect("Should register object successfully");

        // Add one to root set
        gc.add_to_root_set("0x2000".to_string())
            .expect("Should add to root set successfully");

        // Update references so 0x2008 is reachable from 0x2000
        gc.update_references("0x2000".to_string(), vec!["0x2008".to_string()])
            .expect("Should update references successfully");

        // Run garbage collection
        let freed_count = gc.collect_garbage()
            .expect("Should run garbage collection successfully");

        // Since both objects are reachable, none should be collected
        assert_eq!(freed_count, 0);

        // Check that both objects are still alive
        assert!(gc.is_alive("0x2000"));
        assert!(gc.is_alive("0x2008"));
    }

    #[test]
    fn test_unreachable_objects() {
        let gc = GarbageCollector::new(1024 * 1024); // 1MB max heap

        // Register objects
        gc.register_object("0x3000".to_string(), ObjectType::Integer, 8)
            .expect("Should register object successfully");

        gc.register_object("0x3008".to_string(), ObjectType::Integer, 8)
            .expect("Should register object successfully");

        // Only add one to root set, making the other unreachable
        gc.add_to_root_set("0x3000".to_string())
            .expect("Should add to root set successfully");

        // Run garbage collection
        let freed_count = gc.collect_garbage()
            .expect("Should run garbage collection successfully");

        // One object should be collected (0x3008)
        assert_eq!(freed_count, 1);

        // Check that the reachable object is still alive
        assert!(gc.is_alive("0x3000"));
        // The unreachable object should not be alive
        assert!(!gc.is_alive("0x3008"));
    }
}