//! Trait system implementation for the Logos programming language
//! Provides Rust-like trait functionality with associated types, default implementations, and trait bounds

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::ast::*;

/// Represents a trait definition with methods and associated types
#[derive(Debug, Clone)]
pub struct TraitData {
    pub name: String,
    pub type_params: Vec<String>,           // Generic type parameters
    pub required_methods: Vec<FunctionDef>, // Methods that implementing types must provide
    pub provided_methods: Vec<FunctionDef>, // Default implementations
    pub associated_types: Vec<crate::ast::AssociatedTypeDef>, // Associated types
    pub super_traits: Vec<String>,          // Super traits (inheritance)
}


/// Implementation of a trait for a specific type
#[derive(Debug, Clone)]
pub struct TraitImplData {
    pub trait_name: String,
    pub for_type: String,
    pub type_params: Vec<String>,         // Generic type parameters
    pub methods: Vec<FunctionDef>,
    pub associated_types: Vec<(String, Type)>, // Implemented associated types
}

/// Trait resolver that manages trait definitions and implementations
#[derive(Debug, Clone)]
pub struct TraitResolver {
    traits: HashMap<String, TraitData>,
    implementations: HashMap<String, Vec<TraitImplData>>, // Keyed by trait name
    concrete_types: HashMap<String, Type>, // For type checking
    // Caching for performance optimization
    trait_implementation_cache: Arc<RwLock<HashMap<String, HashMap<String, TraitImplData>>>>, // (trait_name, type_name) -> TraitImplData
    trait_hierarchy_cache: Arc<RwLock<HashMap<String, Vec<String>>>>, // type_name -> [super_trait_names]
}

impl TraitResolver {
    /// Creates a new trait resolver instance
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            implementations: HashMap::new(),
            concrete_types: HashMap::new(),
            trait_implementation_cache: Arc::new(RwLock::new(HashMap::new())),
            trait_hierarchy_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a trait definition
    pub fn register_trait(&mut self, trait_def: TraitDef) -> Result<(), String> {
        let trait_data = TraitData {
            name: trait_def.name.clone(),
            type_params: trait_def.type_params.clone(),
            required_methods: trait_def.methods.clone(), // Assuming all methods in TraitDef are required
            provided_methods: vec![], // Default implementations would need to be identified separately
            associated_types: trait_def.associated_types.clone(),
            super_traits: trait_def.super_traits.clone(),
        };

        if self.traits.contains_key(&trait_def.name) {
            return Err(format!("Trait '{}' already defined", trait_def.name));
        }

        self.traits.insert(trait_def.name.clone(), trait_data);
        Ok(())
    }

    /// Registers a trait implementation
    pub fn register_implementation(&mut self, impl_def: ImplDef) -> Result<(), String> {
        // Check if the trait exists
        if !self.traits.contains_key(&impl_def.trait_name) {
            return Err(format!("Trait '{}' not found", impl_def.trait_name));
        }

        let impl_data = TraitImplData {
            trait_name: impl_def.trait_name.clone(),
            for_type: impl_def.for_type.clone(),
            type_params: impl_def.type_params.clone(),
            methods: impl_def.methods.clone(),
            associated_types: impl_def.associated_types.clone(),
        };

        // Add to the list of implementations for this trait
        self.implementations
            .entry(impl_def.trait_name.clone())
            .or_insert_with(Vec::new)
            .push(impl_data);

        Ok(())
    }

    /// Checks if a type implements a specific trait
    pub fn implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
        // Check the implementation cache first
        if let Ok(cache) = self.trait_implementation_cache.read() {
            if let Some(trait_cache) = cache.get(trait_name) {
                return trait_cache.contains_key(type_name);
            }
        }

        // If not in cache, check the main storage
        if let Some(implementations) = self.implementations.get(trait_name) {
            implementations.iter().any(|imp| imp.for_type == type_name)
        } else {
            false
        }
    }

    /// Gets the implementation of a trait for a specific type
    pub fn get_implementation(&self, type_name: &str, trait_name: &str) -> Option<&TraitImplData> {
        // First, check the main storage
        let result = if let Some(implementations) = self.implementations.get(trait_name) {
            implementations.iter().find(|imp| imp.for_type == type_name)
        } else {
            None
        };

        // If found in main storage, cache it and return
        if let Some(impl_data) = result {
            // Cache the result if found
            if let Ok(mut cache) = self.trait_implementation_cache.write() {
                cache.entry(trait_name.to_string())
                    .or_insert_with(HashMap::new)
                    .insert(type_name.to_string(), impl_data.clone());
            }
            return result;
        }

        // If not in main storage, check the cache
        if let Ok(cache) = self.trait_implementation_cache.read() {
            if let Some(trait_cache) = cache.get(trait_name) {
                if let Some(impl_data) = trait_cache.get(type_name) {
                    // Find the original reference in the main storage
                    if let Some(implementations) = self.implementations.get(trait_name) {
                        return implementations.iter().find(|imp| imp.for_type == type_name);
                    }
                }
            }
        }

        None
    }

    /// Gets the definition of a trait
    pub fn get_trait(&self, trait_name: &str) -> Option<&TraitData> {
        self.traits.get(trait_name)
    }

    /// Checks if a type satisfies trait bounds
    pub fn check_trait_bounds(&self, type_name: &str, bounds: &[String]) -> Result<(), String> {
        for bound in bounds {
            if !self.implements_trait(type_name, bound) {
                return Err(format!("Type '{}' does not implement trait '{}'", type_name, bound));
            }
        }
        Ok(())
    }

    /// Resolves a method call on a type implementing a trait
    pub fn resolve_trait_method(&self, type_name: &str, trait_name: &str, method_name: &str) -> Option<&FunctionDef> {
        if let Some(implementation) = self.get_implementation(type_name, trait_name) {
            implementation.methods.iter().find(|m| m.name == method_name)
        } else {
            // If not found in implementation, check default methods in trait
            if let Some(trait_data) = self.get_trait(trait_name) {
                trait_data.provided_methods.iter().find(|m| m.name == method_name)
            } else {
                None
            }
        }
    }
}

/// Trait constraint for generic type parameters
#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub param: String,      // The generic parameter name
    pub bounds: Vec<String>, // Trait bounds for the parameter
}

/// Trait solver for resolving complex trait relationships
#[derive(Debug, Clone)]
pub struct TraitSolver {
    resolver: TraitResolver,
}

impl TraitSolver {
    pub fn new(resolver: TraitResolver) -> Self {
        Self { resolver }
    }

    /// Solves trait constraints for a given type
    pub fn solve_constraints(&self, type_name: &str, constraints: &[TraitConstraint]) -> Result<(), String> {
        for constraint in constraints {
            for bound in &constraint.bounds {
                if !self.resolver.implements_trait(type_name, bound) {
                    return Err(format!(
                        "Type '{}' does not satisfy trait constraint '{}: {}'",
                        type_name, constraint.param, bound
                    ));
                }
            }
        }
        Ok(())
    }

    /// Finds all traits implemented by a type
    pub fn find_implemented_traits(&self, type_name: &str) -> Vec<String> {
        let mut implemented = Vec::new();

        for (trait_name, implementations) in &self.resolver.implementations {
            if implementations.iter().any(|imp| imp.for_type == type_name) {
                implemented.push(trait_name.clone());
            }
        }

        implemented
    }

    /// Resolves associated types for a given implementation
    pub fn resolve_associated_type(&self, type_name: &str, trait_name: &str, assoc_type_name: &str) -> Option<Type> {
        if let Some(implementation) = self.resolver.get_implementation(type_name, trait_name) {
            for (name, ty) in &implementation.associated_types {
                if name == assoc_type_name {
                    return Some(ty.clone());
                }
            }
        }
        None
    }

    /// Checks if a type implements all required traits and their super-traits
    pub fn check_trait_hierarchy(&self, type_name: &str, trait_name: &str) -> Result<(), String> {
        // Check if the type implements the trait directly
        if !self.resolver.implements_trait(type_name, trait_name) {
            return Err(format!("Type '{}' does not implement trait '{}'", type_name, trait_name));
        }

        // Check if the trait exists
        let trait_data = self.resolver.get_trait(trait_name)
            .ok_or_else(|| format!("Trait '{}' not found", trait_name))?;

        // Recursively check super-traits
        for super_trait in &trait_data.super_traits {
            self.check_trait_hierarchy(type_name, super_trait)?;
        }

        Ok(())
    }

    /// Gets all methods available for a type through trait implementations
    pub fn get_all_methods(&self, type_name: &str) -> Vec<FunctionDef> {
        let mut all_methods = Vec::new();

        // Find all traits implemented by this type
        for trait_name in self.find_implemented_traits(type_name) {
            if let Some(implementation) = self.resolver.get_implementation(type_name, &trait_name) {
                all_methods.extend_from_slice(&implementation.methods);
            }

            // Also include default methods from the trait
            if let Some(trait_data) = self.resolver.get_trait(&trait_name) {
                all_methods.extend_from_slice(&trait_data.provided_methods);
            }
        }

        all_methods
    }
}

/// Validates trait implementations against trait definitions
pub fn validate_trait_impl(impl_def: &ImplDef, trait_resolver: &TraitResolver) -> Result<(), String> {
    let trait_data = trait_resolver.get_trait(&impl_def.trait_name)
        .ok_or_else(|| format!("Trait '{}' not found", impl_def.trait_name))?;

    // Check that all required methods are implemented
    for required_method in &trait_data.required_methods {
        if !impl_def.methods.iter().any(|m| m.name == required_method.name) {
            return Err(format!(
                "Missing implementation for required method '{}' in trait '{}'",
                required_method.name, impl_def.trait_name
            ));
        }
    }

    // Check that implemented methods match the signature in the trait
    for impl_method in &impl_def.methods {
        if let Some(trait_method) = trait_data.required_methods.iter()
            .chain(trait_data.provided_methods.iter())
            .find(|tm| tm.name == impl_method.name) {

            // Check parameter types match
            if impl_method.parameters.len() != trait_method.parameters.len() {
                return Err(format!(
                    "Method '{}' has wrong number of parameters in implementation",
                    impl_method.name
                ));
            }

            // Additional checks for parameter and return types would go here
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_registration() {
        let mut resolver = TraitResolver::new();
        
        // Create a simple trait definition
        let trait_def = TraitDef {
            name: "Display".to_string(),
            methods: vec![
                FunctionDef {
                    name: "display".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::String),
                    body: vec![],
                    is_async: false,
                    is_public: true,
                    is_awaitable: false,
                    effect_annotations: vec![],
                }
            ],
        };
        
        assert!(resolver.register_trait(trait_def).is_ok());
        assert!(resolver.get_trait("Display").is_some());
    }

    #[test]
    fn test_trait_implementation() {
        let mut resolver = TraitResolver::new();
        
        // Register a trait first
        let trait_def = TraitDef {
            name: "Display".to_string(),
            methods: vec![
                FunctionDef {
                    name: "display".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::String),
                    body: vec![],
                    is_async: false,
                    is_public: true,
                    is_awaitable: false,
                    effect_annotations: vec![],
                }
            ],
        };
        resolver.register_trait(trait_def).unwrap();
        
        // Create an implementation
        let impl_def = ImplDef {
            trait_name: "Display".to_string(),
            for_type: "Person".to_string(),
            methods: vec![
                FunctionDef {
                    name: "display".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::String),
                    body: vec![],
                    is_async: false,
                    is_public: true,
                    is_awaitable: false,
                    effect_annotations: vec![],
                }
            ],
        };
        
        assert!(resolver.register_implementation(impl_def).is_ok());
        assert!(resolver.implements_trait("Person", "Display"));
    }
}