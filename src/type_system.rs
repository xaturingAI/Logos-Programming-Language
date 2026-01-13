// Logos Programming Language Enhanced Type System
// This module provides advanced type system features for the Logos programming language
// including dependent types, linear types, and algebraic effects.

use crate::ast;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents the enhanced type system with advanced features
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Basic types (wrapping AST types)
    Basic(ast::Type),

    // Advanced type system features
    /// Dependent function type (Pi type): (x: A) -> B(x)
    /// Where the return type B depends on the value x of type A
    Pi {
        parameter: ast::Parameter,
        return_type: Box<Type>,
    },

    /// Dependent pair type (Sigma type): (x: A, B(x))
    /// Where the type of the second element depends on the value of the first
    Sigma {
        fst_type: Box<Type>,
        snd_type: Box<Type>,
    },

    /// Universe levels for type hierarchy
    Universe(u32),

    /// Equality type: a =_A b (proof that a and b are equal in type A)
    Equality {
        type_of_elements: Box<Type>,
        left_expr: Box<ast::Expression>,
        right_expr: Box<ast::Expression>,
    },

    /// Linear type: consumes resources exactly once
    Linear(Box<Type>),

    /// Recursive type: mu X. T (where X can appear in T)
    Recursive(String, Box<Type>),

    /// Intersection type: A & B (value belongs to both A and B)
    Intersection(Vec<Type>),

    /// Union type: A | B (value belongs to either A or B)
    Union(Vec<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Basic(ast_type) => write!(f, "{}", ast_type),
            Type::Pi { parameter, return_type } => {
                write!(f, "({}: {}) -> {}", parameter.name, parameter.type_annotation, return_type)
            },
            Type::Sigma { fst_type, snd_type } => {
                write!(f, "Sigma({}, {})", fst_type, snd_type)
            },
            Type::Universe(level) => write!(f, "Type{}", level),
            Type::Equality { type_of_elements, .. } => {
                write!(f, "Equal<{}>", type_of_elements)
            },
            Type::Linear(t) => write!(f, "!{}", t),
            Type::Recursive(name, inner) => write!(f, "mu {}. {}", name, inner),
            Type::Intersection(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", type_strs.join(" & "))
            },
            Type::Union(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", type_strs.join(" | "))
            },
        }
    }
}

/// Type environment for tracking variable types during type checking
#[derive(Debug, Clone)]
pub struct TypeEnv {
    types: HashMap<String, Type>,
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    /// Creates a new type environment with optional parent
    pub fn new(parent: Option<TypeEnv>) -> Self {
        TypeEnv {
            types: HashMap::new(),
            parent: parent.map(Box::new),
        }
    }

    /// Gets the type of a variable from the environment
    pub fn get_type(&self, name: &str) -> Option<Type> {
        match self.types.get(name) {
            Some(ty) => Some(ty.clone()),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.get_type(name)
                } else {
                    None
                }
            }
        }
    }

    /// Sets the type of a variable in the environment
    pub fn set_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Checks if a variable exists in the environment
    pub fn contains(&self, name: &str) -> bool {
        if self.types.contains_key(name) {
            true
        } else if let Some(ref parent) = self.parent {
            parent.contains(name)
        } else {
            false
        }
    }
}

/// Advanced type checker with support for dependent and linear types
pub struct AdvancedTypeChecker {
    /// Current type environment (variable -> type mappings)
    env: TypeEnv,

    /// Linear resource tracking (resource -> usage count)
    linear_resources: HashMap<String, u32>,

    /// Effect tracking for algebraic effects
    active_effects: HashSet<String>,

    /// Type inference cache to improve performance
    inference_cache: HashMap<String, Type>,

    /// Constraint solving for generic types
    constraints: Vec<TypeConstraint>,
}

/// Type constraint for constraint-based type inference
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub left: Type,
    pub right: Type,
    pub relation: ConstraintRelation,
}

#[derive(Debug, Clone)]
pub enum ConstraintRelation {
    Equal,      // left = right
    Subtype,    // left <: right
    MemberOf,   // left ∈ right (for bounded quantification)
}

impl AdvancedTypeChecker {
    /// Creates a new advanced type checker instance with built-in types
    pub fn new() -> Self {
        let mut env = TypeEnv::new(None);

        // Register built-in types
        env.set_type("Int".to_string(), Type::Basic(ast::Type::Int));
        env.set_type("Float".to_string(), Type::Basic(ast::Type::Float));
        env.set_type("Bool".to_string(), Type::Basic(ast::Type::Bool));
        env.set_type("String".to_string(), Type::Basic(ast::Type::String));
        env.set_type("Unit".to_string(), Type::Basic(ast::Type::Unit));

        Self {
            env,
            linear_resources: HashMap::new(),
            active_effects: HashSet::new(),
            inference_cache: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    /// Checks if a variable is a linear type
    pub fn is_linear_type(&self, var_name: &str) -> bool {
        if let Some(var_type) = self.env.get_type(var_name) {
            matches!(var_type, Type::Linear(_))
        } else {
            false
        }
    }

    /// Records the usage of a linear resource
    pub fn use_linear_resource(&mut self, var_name: &str) -> Result<(), String> {
        if self.is_linear_type(var_name) {
            let count = self.linear_resources.entry(var_name.to_string()).or_insert(0);
            *count += 1;

            // Linear types should be used exactly once
            if *count > 1 {
                return Err(format!("Linear resource '{}' used more than once", var_name));
            }
        }
        Ok(())
    }

    /// Validates that all linear resources have been used exactly once
    pub fn validate_linear_usage(&self) -> Result<(), String> {
        for (resource, count) in &self.linear_resources {
            if *count == 0 {
                return Err(format!("Linear resource '{}' was not used", resource));
            } else if *count > 1 {
                return Err(format!("Linear resource '{}' was used {} times (should be exactly once)", resource, count));
            }
        }
        Ok(())
    }

    /// Adds a type constraint for inference
    pub fn add_constraint(&mut self, left: Type, right: Type, relation: ConstraintRelation) {
        self.constraints.push(TypeConstraint { left, right, relation });
    }

    /// Solves all accumulated type constraints
    pub fn solve_constraints(&mut self) -> Result<(), String> {
        // Simple unification algorithm for constraint solving
        for constraint in &self.constraints {
            match &constraint.relation {
                ConstraintRelation::Equal => {
                    // For now, just check if types are compatible
                    if !self.types_compatible(&constraint.left, &constraint.right) {
                        return Err(format!("Cannot unify types: {:?} and {:?}",
                                         constraint.left, constraint.right));
                    }
                },
                ConstraintRelation::Subtype => {
                    // Check subtype relationship
                    if !self.is_subtype(&constraint.left, &constraint.right) {
                        return Err(format!("Type {:?} is not a subtype of {:?}",
                                         constraint.left, constraint.right));
                    }
                },
                ConstraintRelation::MemberOf => {
                    // Check membership in bounded quantification
                    // This is a simplified implementation
                    if !self.is_member_of(&constraint.left, &constraint.right) {
                        return Err(format!("Type {:?} is not a member of {:?}",
                                         constraint.left, constraint.right));
                    }
                },
            }
        }
        Ok(())
    }

    /// Checks if left type is a subtype of right type
    fn is_subtype(&self, left: &Type, right: &Type) -> bool {
        // For now, use the same compatibility check
        self.types_compatible(left, right)
    }

    /// Checks if left type is a member of the right type (for bounded quantification)
    fn is_member_of(&self, left: &Type, right: &Type) -> bool {
        // Simplified implementation - in a full implementation, this would check
        // if left is a member of the type set defined by right
        self.types_compatible(left, right)
    }

    /// Performs type inference on an expression
    pub fn infer_type(&mut self, expr: &ast::Expression) -> Result<Type, String> {
        // Check if we've already inferred the type for this expression
        if let ast::Expression::Identifier(name) = expr {
            if let Some(cached_type) = self.inference_cache.get(name) {
                return Ok(cached_type.clone());
            }
        }

        let inferred_type = match expr {
            ast::Expression::Integer(_) => Type::Basic(ast::Type::Int),
            ast::Expression::Float(_) => Type::Basic(ast::Type::Float),
            ast::Expression::String(_) => Type::Basic(ast::Type::String),
            ast::Expression::Boolean(_) => Type::Basic(ast::Type::Bool),
            ast::Expression::Identifier(name) => {
                if let Some(var_type) = self.env.get_type(name) {
                    var_type
                } else {
                    return Err(format!("Variable '{}' not found in environment", name));
                }
            },
            ast::Expression::BinaryOp(left, op, right) => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                // Add constraints to ensure both operands have compatible types
                self.add_constraint(left_type.clone(), right_type.clone(), ConstraintRelation::Equal);

                // For now, return the type of the left operand
                // In a full implementation, this would depend on the operation
                left_type
            },
            ast::Expression::Call(func_name, args) => {
                // For function calls, we need to look up the function signature
                // This is a simplified implementation
                Type::Basic(ast::Type::Infer) // Return an inferred type
            },
            // Add more cases as needed
            _ => Type::Basic(ast::Type::Infer), // Default to inferred type
        };

        // Cache the inferred type if it's for an identifier
        if let ast::Expression::Identifier(name) = expr {
            self.inference_cache.insert(name.clone(), inferred_type.clone());
        }

        Ok(inferred_type)
    }

    /// Checks if two types are compatible
    pub fn types_compatible(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            // Basic type compatibility
            (Type::Basic(ast1), Type::Basic(ast2)) => self.ast_types_compatible(ast1, ast2),
            
            // Linear type compatibility
            (Type::Linear(t1), Type::Linear(t2)) => self.types_compatible(t1, t2),
            (Type::Linear(t1), Type::Basic(basic_type)) => {
                // Linear type is compatible with basic type if the inner type matches
                matches!(t1.as_ref(), Type::Basic(inner) if inner == basic_type)
            },
            (Type::Basic(basic_type), Type::Linear(t2)) => {
                // Basic type is compatible with linear type if the inner type matches
                matches!(t2.as_ref(), Type::Basic(inner) if inner == basic_type)
            },
            
            // Pi type compatibility (simplified)
            (Type::Pi { parameter: p1, return_type: r1 }, Type::Pi { parameter: p2, return_type: r2 }) => {
                // For now, check that parameter types are the same and return types are compatible
                self.ast_types_compatible(&p1.type_annotation, &p2.type_annotation) &&
                self.types_compatible(r1, r2)
            },
            
            // Sigma type compatibility
            (Type::Sigma { fst_type: f1, snd_type: s1 }, Type::Sigma { fst_type: f2, snd_type: s2 }) => {
                self.types_compatible(f1, f2) && self.types_compatible(s1, s2)
            },
            
            // Universe compatibility (higher universes contain lower ones)
            (Type::Universe(l1), Type::Universe(l2)) => l1 >= l2,
            
            // Recursive type compatibility
            (Type::Recursive(name1, inner1), Type::Recursive(name2, inner2)) => {
                name1 == name2 && self.types_compatible(inner1, inner2)
            },
            
            // Intersection compatibility (simplified - all types in first must have matches in second)
            (Type::Intersection(types1), Type::Intersection(types2)) => {
                // Every type in types1 should be compatible with some type in types2
                types1.iter().all(|t1| 
                    types2.iter().any(|t2| self.types_compatible(t1, t2))
                ) && 
                // Every type in types2 should be compatible with some type in types1
                types2.iter().all(|t2| 
                    types1.iter().any(|t1| self.types_compatible(t1, t2))
                )
            },
            
            // Union compatibility (simplified - some type in first must be compatible with some type in second)
            (Type::Union(types1), Type::Union(types2)) => {
                types1.iter().any(|t1| 
                    types2.iter().any(|t2| self.types_compatible(t1, t2))
                )
            },
            
            // Default: not compatible
            _ => false,
        }
    }

    /// Checks compatibility between AST types
    fn ast_types_compatible(&self, ty1: &ast::Type, ty2: &ast::Type) -> bool {
        match (ty1, ty2) {
            // Basic compatibility
            (ast::Type::Int, ast::Type::Int) => true,
            (ast::Type::Float, ast::Type::Float) => true,
            (ast::Type::Bool, ast::Type::Bool) => true,
            (ast::Type::String, ast::Type::String) => true,
            (ast::Type::Unit, ast::Type::Unit) => true,
            
            // Array compatibility
            (ast::Type::Array(t1), ast::Type::Array(t2)) => self.ast_types_compatible(t1, t2),
            
            // Tuple compatibility
            (ast::Type::Tuple(types1), ast::Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    false
                } else {
                    types1.iter().zip(types2.iter()).all(|(a, b)| self.ast_types_compatible(a, b))
                }
            },
            
            // Function compatibility (simplified - parameters and return types must match)
            (ast::Type::Function(params1, ret1), ast::Type::Function(params2, ret2)) => {
                if params1.len() != params2.len() {
                    false
                } else {
                    // In a real implementation, parameters should be contravariant and return types covariant
                    // For simplicity, we'll check exact match
                    params1.iter().zip(params2.iter()).all(|(a, b)| self.ast_types_compatible(a, b)) && 
                    self.ast_types_compatible(ret1, ret2)
                }
            },
            
            // Option compatibility
            (ast::Type::Option(t1), ast::Type::Option(t2)) => self.ast_types_compatible(t1, t2),
            
            // Result compatibility
            (ast::Type::Result(ok1, err1), ast::Type::Result(ok2, err2)) => {
                self.ast_types_compatible(ok1, ok2) && self.ast_types_compatible(err1, err2)
            },
            
            // Named type compatibility (structural for now)
            (ast::Type::Named(name1), ast::Type::Named(name2)) => name1 == name2,
            
            // Generic compatibility (for same parameter name)
            (ast::Type::Generic(name1), ast::Type::Generic(name2)) => name1 == name2,
            
            // Allow type inference to match anything
            (ast::Type::Infer, _) | (_, ast::Type::Infer) => true,
            
            // Channel compatibility
            (ast::Type::Channel(t1), ast::Type::Channel(t2)) => self.ast_types_compatible(t1, t2),
            
            // Default: not compatible
            _ => false,
        }
    }
}

/// Value representations for dependent type evaluation
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
    Tuple(Vec<Value>),
    Array(Vec<Value>),
    Function(String, Vec<ast::Parameter>, Vec<ast::Statement>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Tuple(values) => {
                let value_strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", value_strs.join(", "))
            },
            Value::Array(values) => {
                let value_strs: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", value_strs.join(", "))
            },
            Value::Function(name, _, _) => write!(f, "<function {}>", name),
        }
    }
}