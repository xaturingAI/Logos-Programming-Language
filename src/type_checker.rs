// Logos Programming Language Type Checker
// This module performs static type checking on the AST to ensure type safety.

use crate::ast::*;
use crate::trait_system::{TraitResolver, validate_trait_impl};
use std::collections::HashMap;

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

/// Type checker for ensuring type safety in the AST
pub struct TypeChecker {
    env: TypeEnv,
    /// Linear resource tracking (resource -> usage count)
    linear_resources: HashMap<String, u32>,
    /// Trait resolver for handling trait-related type checking
    trait_resolver: TraitResolver,
}

impl TypeChecker {
    /// Creates a new type checker instance with built-in types
    pub fn new() -> Self {
        let mut env = TypeEnv::new(None);

        // Register built-in types
        env.set_type("Int".to_string(), Type::Int);
        env.set_type("Float".to_string(), Type::Float);
        env.set_type("Bool".to_string(), Type::Bool);
        env.set_type("String".to_string(), Type::String);
        env.set_type("Unit".to_string(), Type::Unit);

        TypeChecker {
            env,
            linear_resources: HashMap::new(),
            trait_resolver: TraitResolver::new(),
        }
    }

    /// Checks if a type is a linear type
    fn is_linear_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Linear(_))
    }

    /// Records the usage of a linear resource
    fn use_linear_resource(&mut self, var_name: &str) -> Result<(), String> {
        // Check if the variable has a linear type
        if let Some(var_type) = self.env.get_type(var_name) {
            if self.is_linear_type(&var_type) {
                let count = self.linear_resources.entry(var_name.to_string()).or_insert(0);
                *count += 1;

                // Linear types should be used exactly once
                if *count > 1 {
                    return Err(format!("Linear resource '{}' used more than once", var_name));
                }
            }
        }
        Ok(())
    }

    /// Validates that all linear resources have been used exactly once
    fn validate_linear_usage(&self) -> Result<(), String> {
        for (resource, count) in &self.linear_resources {
            if *count == 0 {
                return Err(format!("Linear resource '{}' was not used", resource));
            } else if *count > 1 {
                return Err(format!("Linear resource '{}' was used {} times (should be exactly once)", resource, count));
            }
        }
        Ok(())
    }

    /// Checks the types in a program
    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        for statement in &program.statements {
            self.check_statement(statement)?;
        }

        // Validate linear type usage after checking all statements
        self.validate_linear_usage()?;
        Ok(())
    }

    /// Checks the types in a statement
    fn check_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            },
            Statement::LetBinding { mutable: _, name, type_annotation, value, ownership_modifier: _, lifetime_annotation: _ } => {
                let value_type = self.check_expression(value)?;

                if let Some(expected_type) = type_annotation {
                    if !self.types_compatible(&value_type, expected_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?} for variable '{}'",
                            expected_type, value_type, name
                        ));
                    }
                }

                // Add the variable to the environment
                let final_type = type_annotation.clone().unwrap_or(value_type.clone());
                self.env.set_type(name.clone(), final_type.clone());

                // If it's a linear type, initialize its usage count to 0
                if self.is_linear_type(&final_type) {
                    self.linear_resources.insert(name.clone(), 0);
                }

                Ok(())
            },
            Statement::ConstBinding { name, type_annotation, value } => {
                let value_type = self.check_expression(value)?;
                
                if let Some(expected_type) = type_annotation {
                    if !self.types_compatible(&value_type, expected_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?} for constant '{}'",
                            expected_type, value_type, name
                        ));
                    }
                }
                
                // Add the constant to the environment
                let final_type = type_annotation.clone().unwrap_or(value_type);
                self.env.set_type(name.clone(), final_type);
                
                Ok(())
            },
            Statement::Function(func_def) => {
                self.check_function(func_def)?;
                Ok(())
            },
            Statement::Block(statements) => {
                // Check the block with a new environment
                let mut checker = TypeChecker {
                    env: TypeEnv::new(Some(self.env.clone())),
                    linear_resources: self.linear_resources.clone(), // Inherit linear resources
                    trait_resolver: self.trait_resolver.clone(), // Inherit trait resolver
                };

                for stmt in statements {
                    checker.check_statement(stmt)?;
                }

                // Copy back the linear resource usage from the block
                for (resource, count) in &checker.linear_resources {
                    self.linear_resources.insert(resource.clone(), *count);
                }

                Ok(())
            },
            Statement::Trait(trait_def) => {
                // Check trait definition
                self.check_trait(trait_def)?;
                Ok(())
            },
            Statement::Implementation(impl_def) => {
                // Check implementation block
                self.check_implementation(impl_def)?;
                Ok(())
            },
            Statement::MacroDefinition(macro_def) => {
                // For now, just acknowledge the macro definition
                // In a full implementation, we would validate the macro template
                Ok(())
            },
            Statement::Return(expr) => {
                match expr {
                    Some(e) => {
                        self.check_expression(e)?;
                        Ok(())
                    },
                    None => Ok(()),
                }
            },
            // Handle other statement types as needed
            _ => Ok(()), // For now, accept other statements without strict checking
        }
    }

    /// Checks the types in a function definition
    fn check_function(&mut self, func_def: &FunctionDef) -> Result<(), String> {
        // Create a new environment for the function body
        let mut func_env = TypeEnv::new(Some(self.env.clone()));

        // Add parameters to the function environment
        for param in &func_def.parameters {
            func_env.set_type(param.name.clone(), param.type_annotation.clone());
        }

        // Check the function body with the new environment
        let mut checker = TypeChecker {
            env: func_env,
            linear_resources: HashMap::new(), // Each function gets its own linear resource tracker
            trait_resolver: self.trait_resolver.clone(), // Inherit trait resolver
        };
        for stmt in &func_def.body {
            checker.check_statement(stmt)?;
        }

        // Validate linear type usage in the function
        checker.validate_linear_usage()?;

        Ok(())
    }

    /// Checks a trait definition
    fn check_trait(&mut self, trait_def: &TraitDef) -> Result<(), String> {
        // Register the trait with the trait resolver
        self.trait_resolver.register_trait(trait_def.clone())?;

        // Create a new environment for the trait
        let mut trait_env = TypeEnv::new(Some(self.env.clone()));

        // Check each method signature in the trait
        for method in &trait_def.methods {
            // Add method parameters to the trait environment
            for param in &method.parameters {
                trait_env.set_type(param.name.clone(), param.type_annotation.clone());
            }

            // Check the method body if it exists (traits may have default implementations)
            if !method.body.is_empty() {
                let mut method_checker = TypeChecker {
                    env: trait_env.clone(),
                    linear_resources: HashMap::new(),
                    trait_resolver: TraitResolver::new(), // Fresh resolver for method checking
                };

                for stmt in &method.body {
                    method_checker.check_statement(stmt)?;
                }

                // Validate linear usage in the method
                method_checker.validate_linear_usage()?;
            }
        }

        // Register the trait in the environment
        self.env.set_type(trait_def.name.clone(), Type::Named(trait_def.name.clone()));

        Ok(())
    }

    /// Checks an implementation block
    fn check_implementation(&mut self, impl_def: &ImplDef) -> Result<(), String> {
        // Validate the implementation against the trait definition
        validate_trait_impl(impl_def, &self.trait_resolver)?;

        // Register the implementation with the trait resolver
        self.trait_resolver.register_implementation(impl_def.clone())?;

        // Create a new environment for the implementation
        let mut impl_env = TypeEnv::new(Some(self.env.clone()));

        // Check each implemented method
        for method in &impl_def.methods {
            // Add method parameters to the implementation environment
            for param in &method.parameters {
                impl_env.set_type(param.name.clone(), param.type_annotation.clone());
            }

            // Check the method body
            let mut method_checker = TypeChecker {
                env: impl_env.clone(),
                linear_resources: HashMap::new(),
                trait_resolver: TraitResolver::new(), // Fresh resolver for method checking
            };

            for stmt in &method.body {
                method_checker.check_statement(stmt)?;
            }

            // Validate linear usage in the method
            method_checker.validate_linear_usage()?;
        }

        Ok(())
    }

    /// Checks the types in an expression
    fn check_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Integer(_) => Ok(Type::Int),
            Expression::Float(_) => Ok(Type::Float),
            Expression::String(_) => Ok(Type::String),
            Expression::Boolean(_) => Ok(Type::Bool),
            Expression::Nil => Ok(Type::Unit),
            Expression::Identifier(name) => {
                match self.env.get_type(name) {
                    Some(ty) => {
                        // Record usage of linear resources
                        self.use_linear_resource(name)?;
                        Ok(ty)
                    },
                    None => Err(format!("Undefined variable: {}", name)),
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;
                
                // Check if the operation is valid for the types
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        // These operations require numeric types
                        if self.is_numeric_type(&left_type) && self.is_numeric_type(&right_type) {
                            // Return the wider type (Float if either is Float)
                            if self.is_float_type(&left_type) || self.is_float_type(&right_type) {
                                Ok(Type::Float)
                            } else {
                                Ok(Type::Int)
                            }
                        } else {
                            Err(format!(
                                "Operator {:?} requires numeric operands, found {:?} and {:?}",
                                op, left_type, right_type
                            ))
                        }
                    },
                    BinaryOp::Eq | BinaryOp::Ne => {
                        // Equality operations return Bool
                        if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Cannot compare {:?} with {:?}", 
                                left_type, right_type
                            ))
                        }
                    },
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        // Comparison operations return Bool and require comparable types
                        if self.is_comparable_type(&left_type) && self.is_comparable_type(&right_type) &&
                           self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Cannot compare {:?} with {:?} using {:?}", 
                                left_type, right_type, op
                            ))
                        }
                    },
                    BinaryOp::And | BinaryOp::Or => {
                        // Logical operations require Bool types
                        if self.is_boolean_type(&left_type) && self.is_boolean_type(&right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Operator {:?} requires boolean operands, found {:?} and {:?}",
                                op, left_type, right_type
                            ))
                        }
                    },
                    _ => {
                        // For other operations, return a general type or handle specifically
                        Ok(Type::Infer) // Using Infer as a placeholder
                    }
                }
            },
            Expression::UnaryOp(op, expr) => {
                let expr_type = self.check_expression(expr)?;
                
                match op {
                    UnaryOp::Neg => {
                        if self.is_numeric_type(&expr_type) {
                            Ok(expr_type) // Return the same numeric type
                        } else {
                            Err(format!("Unary minus requires numeric operand, found {:?}", expr_type))
                        }
                    },
                    UnaryOp::Not => {
                        if self.is_boolean_type(&expr_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Unary not requires boolean operand, found {:?}", expr_type))
                        }
                    },
                    _ => Ok(Type::Infer), // Placeholder for other unary ops
                }
            },
            Expression::Call(name, args) => {
                // For now, we'll assume built-in functions have known types
                match name.as_str() {
                    "print" => {
                        // print can take any type of argument
                        for arg in args {
                            self.check_expression(arg)?;
                        }
                        Ok(Type::Unit)
                    },
                    "len" => {
                        // len expects a collection/string type
                        if args.len() != 1 {
                            return Err("len() expects exactly one argument".to_string());
                        }
                        let arg_type = self.check_expression(&args[0])?;
                        if self.is_collection_type(&arg_type) {
                            Ok(Type::Int)
                        } else {
                            Err("len() expects a collection or string".to_string())
                        }
                    },
                    "str" => {
                        // str can convert any type to string
                        if args.len() != 1 {
                            return Err("str() expects exactly one argument".to_string());
                        }
                        self.check_expression(&args[0])?;
                        Ok(Type::String)
                    },
                    "int" => {
                        // int can convert compatible types to integer
                        if args.len() != 1 {
                            return Err("int() expects exactly one argument".to_string());
                        }
                        self.check_expression(&args[0])?;
                        Ok(Type::Int)
                    },
                    "float" => {
                        // float can convert compatible types to float
                        if args.len() != 1 {
                            return Err("float() expects exactly one argument".to_string());
                        }
                        self.check_expression(&args[0])?;
                        Ok(Type::Float)
                    },
                    _ => {
                        // For user-defined functions, we'd need to look up the function signature
                        // For now, return a placeholder type
                        Ok(Type::Infer)
                    }
                }
            },
            Expression::If(condition, then_stmts, else_stmts) => {
                // Check the condition
                let cond_type = self.check_expression(condition)?;
                if !self.is_boolean_type(&cond_type) {
                    return Err(format!("If condition must be boolean, found {:?}", cond_type));
                }

                // Check then branch with a new environment
                let mut then_checker = TypeChecker {
                    env: self.env.clone(),
                    linear_resources: self.linear_resources.clone(),
                    trait_resolver: self.trait_resolver.clone(),
                };
                for stmt in then_stmts {
                    then_checker.check_statement(stmt)?;
                }

                // Check else branch with a new environment
                let mut else_checker = TypeChecker {
                    env: self.env.clone(),
                    linear_resources: self.linear_resources.clone(),
                    trait_resolver: self.trait_resolver.clone(),
                };
                for stmt in else_stmts {
                    else_checker.check_statement(stmt)?;
                }

                // For linear resources that exist in both branches, ensure they're used the same way
                // In a full implementation, we'd merge the linear resource usage
                // For now, we'll just validate both branches separately
                then_checker.validate_linear_usage()?;
                else_checker.validate_linear_usage()?;

                // For now, return a placeholder type for if expressions
                // In a full implementation, we'd need to determine the unified type of both branches
                Ok(Type::Infer)
            },
            Expression::Tuple(items) => {
                let mut item_types = Vec::new();
                for item in items {
                    item_types.push(self.check_expression(item)?);
                }
                Ok(Type::Tuple(item_types))
            },
            Expression::Match(expr, arms) => {
                // Check the expression being matched
                let match_expr_type = self.check_expression(expr)?;

                // Check each arm
                for (pattern, guard, body) in arms {
                    // We need to validate that the pattern is compatible with the match expression type
                    // For now, we'll just check the body statements
                    let mut arm_checker = TypeChecker {
                        env: self.env.clone(),
                        linear_resources: self.linear_resources.clone(),
                        trait_resolver: self.trait_resolver.clone(),
                    };

                    // Bind pattern variables to the arm's environment
                    self.bind_pattern_variables(pattern, &mut arm_checker)?;

                    // Check the guard if it exists
                    if let Some(guard_expr) = guard {
                        let guard_type = arm_checker.check_expression(guard_expr)?;
                        if !self.is_boolean_type(&guard_type) {
                            return Err("Guard expression must be of boolean type".to_string());
                        }
                    }

                    // Check the body statements
                    for stmt in body {
                        arm_checker.check_statement(stmt)?;
                    }

                    // Validate linear usage in this arm
                    arm_checker.validate_linear_usage()?;
                }

                // For now, return a placeholder type
                // In a full implementation, we'd need to determine the unified type of all arms
                Ok(Type::Infer)
            },
            // Handle other expression types as needed
            _ => Ok(Type::Infer), // Placeholder for unhandled expressions
        }
    }

    /// Checks if two types are compatible
    fn types_compatible(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Unit, Type::Unit) => true,
            (Type::Int, Type::Float) => true,  // Int can be promoted to Float
            (Type::Float, Type::Int) => true,  // For some operations, though not always safe
            (Type::Array(t1), Type::Array(t2)) => self.types_compatible(t1, t2),
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    false
                } else {
                    types1.iter().zip(types2).all(|(a, b)| self.types_compatible(a, b))
                }
            },
            (Type::Named(n1), Type::Named(n2)) => n1 == n2,
            (Type::Infer, _) | (_, Type::Infer) => true,  // Infer is compatible with everything
            // Linear type compatibility
            (Type::Linear(t1), Type::Linear(t2)) => self.types_compatible(t1, t2),
            (Type::Linear(t1), other) => self.types_compatible(t1, other),
            (other, Type::Linear(t2)) => self.types_compatible(other, t2),
            _ => false,
        }
    }

    /// Checks if a type is numeric
    fn is_numeric_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float)
    }

    /// Checks if a type is a float
    fn is_float_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Float)
    }

    /// Checks if a type is boolean
    fn is_boolean_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Bool)
    }

    /// Checks if a type is comparable
    fn is_comparable_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int | Type::Float | Type::String | Type::Bool)
    }

    /// Checks if a type is a collection
    fn is_collection_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::String | Type::Array(_))
    }

    /// Binds variables in a pattern to the type checker environment
    fn bind_pattern_variables(&self, pattern: &Pattern, checker: &mut TypeChecker) -> Result<(), String> {
        match pattern {
            Pattern::Identifier(name) => {
                // For now, bind with Infer type - in a full implementation, we'd infer the type from context
                checker.env.set_type(name.clone(), Type::Infer);
                // Initialize linear resource tracking for linear types
                checker.linear_resources.insert(name.clone(), 0);
                Ok(())
            },
            Pattern::Literal(_) => {
                // Literals don't bind any variables
                Ok(())
            },
            Pattern::Wildcard => {
                // Wildcard doesn't bind any variables
                Ok(())
            },
            Pattern::Tuple(pattern_items) => {
                // Bind each element in the tuple pattern
                for item_pattern in pattern_items {
                    self.bind_pattern_variables(item_pattern, checker)?;
                }
                Ok(())
            },
            Pattern::Array(pattern_items) => {
                // Bind each element in the array pattern
                for item_pattern in pattern_items {
                    self.bind_pattern_variables(item_pattern, checker)?;
                }
                Ok(())
            },
            Pattern::Struct(_, fields) => {
                // Bind each field in the struct pattern
                for (_, field_pattern) in fields {
                    self.bind_pattern_variables(field_pattern, checker)?;
                }
                Ok(())
            },
            Pattern::Or(left, right) => {
                // Bind variables from both sides of the or pattern
                self.bind_pattern_variables(left, checker)?;
                self.bind_pattern_variables(right, checker)?;
                Ok(())
            },
        }
    }
}

/// Checks the types in a program
pub fn check_types(program: &Program) -> Result<(), String> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}