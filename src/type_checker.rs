// Logos Programming Language Type Checker
// This module performs static type checking on the AST to ensure type safety.

use crate::ast::*;
use crate::trait_system::{TraitResolver, validate_trait_impl};
use crate::effects::{Effect, EffectSet};
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
#[derive(Clone)]
pub struct TypeChecker {
    env: TypeEnv,
    /// Linear resource tracking (resource -> usage count)
    linear_resources: HashMap<String, u32>,
    /// Trait resolver for handling trait-related type checking
    trait_resolver: TraitResolver,
    /// Effect tracking for algebraic effects
    effects: EffectSet,
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
            effects: EffectSet::new(),
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

    /// Moves a linear resource (transfers ownership)
    fn move_linear_resource(&mut self, var_name: &str) -> Result<(), String> {
        // Check if the variable has a linear type
        if let Some(var_type) = self.env.get_type(var_name) {
            if self.is_linear_type(&var_type) {
                // Mark the resource as moved by setting its count to a special value
                // In a more sophisticated system, we'd track ownership differently
                self.linear_resources.insert(var_name.to_string(), 2); // Mark as moved (more than once to trigger error)

                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
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
                    effects: self.effects.clone(), // Inherit effects
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
            Statement::Class(class_def) => {
                self.check_class(class_def)?;
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
            Statement::Effect(effect_def) => {
                // Register the effect with the effect system
                // In a full implementation, we'd register the effect operations
                // For now, we'll just record that this effect exists
                self.effects.insert(Effect::Custom(effect_def.name.clone()));
                Ok(())
            },
            Statement::Enum(enum_def) => {
                // Register the enum with the type environment
                // For now, we just add the enum name as a type
                self.env.set_type(enum_def.name.clone(), Type::Named(enum_def.name.clone()));

                // Check each variant
                for variant in &enum_def.variants {
                    // For variants with associated data, we'd need to check the types
                    if let Some(data) = &variant.data {
                        match data {
                            VariantData::Tuple(types) => {
                                for ty in types {
                                    // Check that each type in the tuple is valid
                                    // For now, we just ensure the type is known
                                    if !self.is_known_type(ty) {
                                        return Err(format!("Unknown type in enum variant: {:?}", ty));
                                    }
                                }
                            },
                            VariantData::Struct(fields) => {
                                for field in fields {
                                    // Check that each field type is valid
                                    if !self.is_known_type(&field.type_annotation) {
                                        return Err(format!("Unknown type in enum field: {:?}", field.type_annotation));
                                    }
                                }
                            },
                            VariantData::Unit => {
                                // Unit variants have no associated data to check
                            }
                        }
                    }
                }

                Ok(())
            },
            Statement::TypeAlias(alias_def) => {
                // Check that the aliased type is valid
                if !self.is_known_type(&alias_def.aliased_type) {
                    return Err(format!("Unknown type in type alias: {:?}", alias_def.aliased_type));
                }

                // Register the alias in the environment
                self.env.set_type(alias_def.name.clone(), alias_def.aliased_type.clone());

                Ok(())
            },
            // Handle other statement types as needed
            _ => Ok(()), // For now, accept other statements without strict checking
        }
    }

    /// Checks if a type is known (defined in the current environment)
    fn is_known_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit => true,
            Type::Array(inner) => self.is_known_type(inner),
            Type::Tuple(types) => types.iter().all(|t| self.is_known_type(t)),
            Type::Function(params, ret) => {
                params.iter().all(|t| self.is_known_type(t)) && self.is_known_type(ret)
            },
            Type::Channel(element_type) => self.is_known_type(element_type),
            Type::Pi(param, ret_type) => {
                self.is_known_type(&param.type_annotation) && self.is_known_type(ret_type)
            },
            Type::Sigma(param, snd_type) => {
                self.is_known_type(&param.type_annotation) && self.is_known_type(snd_type)
            },
            Type::Universe(_) => true,
            Type::Equality(base_type, _, _) => self.is_known_type(base_type),
            Type::Linear(inner) => self.is_known_type(inner),
            Type::Named(name) => {
                // Check if the named type exists in the environment
                self.env.contains(name) || self.is_builtin_type(name)
            },
            Type::Generic(_) => true, // Generic types are considered known
            Type::GenericWithBounds { .. } => true, // Generic types with bounds are considered known
            Type::Option(inner) => self.is_known_type(inner),
            Type::Result(ok, err) => self.is_known_type(ok) && self.is_known_type(err),
            Type::Infer => true, // Infer is always considered known
        }
    }

    /// Checks if a type name refers to a builtin type
    fn is_builtin_type(&self, name: &str) -> bool {
        matches!(name, "Int" | "Float" | "Bool" | "String" | "Unit" | "Option" | "Result")
    }

    /// Checks a class definition
    fn check_class(&mut self, class_def: &ClassDef) -> Result<(), String> {
        // Add the class name to the environment as a type
        self.env.set_type(class_def.name.clone(), Type::Named(class_def.name.clone()));

        // Add generic parameters to the environment if any
        for gen_param in &class_def.generics {
            self.env.set_type(gen_param.name.clone(), Type::Generic(gen_param.name.clone()));
        }

        // Check each field
        for field in &class_def.fields {
            if !self.is_known_type(&field.type_annotation) {
                return Err(format!("Unknown type in field '{}': {:?}", field.name, field.type_annotation));
            }
        }

        // Check each method
        for method in &class_def.methods {
            // Create a new environment for the method that includes the class context
            let mut method_env = TypeEnv::new(Some(self.env.clone()));

            // Add 'self' parameter if needed
            // For now, we'll just check the method as a standalone function

            // Add generic parameters to the method environment if any
            for gen_param in &method.generic_params {
                method_env.set_type(gen_param.name.clone(), Type::Generic(gen_param.name.clone()));
            }

            // Add parameters to the method environment
            for param in &method.parameters {
                method_env.set_type(param.name.clone(), param.type_annotation.clone());
            }

            // Check the method body with the new environment
            let mut method_checker = TypeChecker {
                env: method_env,
                linear_resources: HashMap::new(), // Each method gets its own linear resource tracker
                trait_resolver: self.trait_resolver.clone(), // Inherit trait resolver
                effects: self.effects.clone(), // Inherit effects
            };

            for stmt in &method.body {
                method_checker.check_statement(stmt)?;
            }

            // Validate linear type usage in the method
            method_checker.validate_linear_usage()?;
        }

        Ok(())
    }

    /// Checks the types in a function definition
    fn check_function(&mut self, func_def: &FunctionDef) -> Result<(), String> {
        // Create a new environment for the function body
        let mut func_env = TypeEnv::new(Some(self.env.clone()));

        // Add generic parameters to the function environment if any
        for gen_param in &func_def.generic_params {
            // Add the generic parameter as a type in the environment
            func_env.set_type(gen_param.name.clone(), Type::Generic(gen_param.name.clone()));
        }

        // Add parameters to the function environment
        for param in &func_def.parameters {
            func_env.set_type(param.name.clone(), param.type_annotation.clone());
        }

        // Check the function body with the new environment
        let mut checker = TypeChecker {
            env: func_env,
            linear_resources: HashMap::new(), // Each function gets its own linear resource tracker
            trait_resolver: self.trait_resolver.clone(), // Inherit trait resolver
            effects: self.effects.clone(), // Inherit effects
        };

        for stmt in &func_def.body {
            checker.check_statement(stmt)?;
        }

        // Validate linear type usage in the function
        checker.validate_linear_usage()?;

        // Update the function's effect annotations based on what was detected
        // In a full implementation, we'd match the detected effects to the declared ones
        // For now, we'll just note the effects that were used

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
                    effects: self.effects.clone(), // Inherit effects
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
                effects: self.effects.clone(), // Inherit effects
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
            Expression::FieldAccess(obj_expr, field_name) => {
                let obj_type = self.check_expression(obj_expr)?;

                // For now, we'll handle field access on basic types
                // In a full implementation, we'd check if the object type has the specified field
                match &obj_type {
                    Type::Named(_) => {
                        // For user-defined types, we'd need to look up the field type
                        // For now, return a placeholder
                        Ok(Type::Infer)
                    },
                    Type::Linear(inner_type) => {
                        // When accessing a field of a linear type, we're consuming the linear resource
                        if let Expression::Identifier(var_name) = obj_expr.as_ref() {
                            self.move_linear_resource(var_name)?;
                        }
                        // Return the type of the field (placeholder for now)
                        Ok(Type::Infer)
                    },
                    _ => Ok(Type::Infer), // Placeholder for other types
                }
            },
            Expression::MethodCall(obj_expr, method_name, args) => {
                let obj_type = self.check_expression(obj_expr)?;

                // Check all arguments
                for arg in args {
                    self.check_expression(arg)?;
                }

                // For linear types, method calls might consume the resource
                if self.is_linear_type(&obj_type) {
                    if let Expression::Identifier(var_name) = obj_expr.as_ref() {
                        self.move_linear_resource(var_name)?;
                    }
                }

                // For now, return a placeholder type
                // In a full implementation, we'd look up the method signature
                Ok(Type::Infer)
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
                    BinaryOp::PipeForward => {
                        // Pipeline operator: left |> right
                        // This applies the function 'right' to the value 'left'
                        // 'right' should be a function that accepts 'left' as its first argument

                        // Check if 'right' is a function type
                        if let Type::Function(params, return_type) = &right_type {
                            if !params.is_empty() {
                                // Check if the left type matches the first parameter of the function
                                if self.types_compatible(&left_type, &params[0]) {
                                    // Return the return type of the function
                                    Ok(*return_type.clone())
                                } else {
                                    Err(format!(
                                        "Pipeline operator: left operand type {:?} does not match function's first parameter type {:?}",
                                        left_type, params[0]
                                    ))
                                }
                            } else {
                                Err("Pipeline operator: function has no parameters".to_string())
                            }
                        } else {
                            Err(format!(
                                "Pipeline operator: right operand must be a function, found {:?}",
                                right_type
                            ))
                        }
                    },
                    BinaryOp::PipeBackward => {
                        // Backward pipeline operator: left <| right
                        // This is similar to |> but with reversed arguments
                        // 'left' should be a function that accepts 'right' as its first argument

                        // Check if 'left' is a function type
                        if let Type::Function(params, return_type) = &left_type {
                            if !params.is_empty() {
                                // Check if the right type matches the first parameter of the function
                                if self.types_compatible(&right_type, &params[0]) {
                                    // Return the return type of the function
                                    Ok(*return_type.clone())
                                } else {
                                    Err(format!(
                                        "Backward pipeline operator: right operand type {:?} does not match function's first parameter type {:?}",
                                        right_type, params[0]
                                    ))
                                }
                            } else {
                                Err("Backward pipeline operator: function has no parameters".to_string())
                            }
                        } else {
                            Err(format!(
                                "Backward pipeline operator: left operand must be a function, found {:?}",
                                left_type
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
                    effects: self.effects.clone(), // Inherit effects
                };
                for stmt in then_stmts {
                    then_checker.check_statement(stmt)?;
                }

                // Check else branch with a new environment
                let mut else_checker = TypeChecker {
                    env: self.env.clone(),
                    linear_resources: self.linear_resources.clone(),
                    trait_resolver: self.trait_resolver.clone(),
                    effects: self.effects.clone(), // Inherit effects
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
                        effects: self.effects.clone(), // Inherit effects
                    };

                    // Bind pattern variables to the arm's environment
                    self.bind_pattern_variables_to_env(pattern, &mut arm_checker.env)?;

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
            Expression::Lambda(params, body) => {
                // Create a new environment for the lambda
                let mut lambda_env = TypeEnv::new(Some(self.env.clone()));

                // Add parameters to the lambda environment
                for param in params {
                    lambda_env.set_type(param.name.clone(), param.type_annotation.clone());
                }

                // Check the lambda body with the new environment
                let mut lambda_checker = TypeChecker {
                    env: lambda_env,
                    linear_resources: self.linear_resources.clone(), // Inherit linear resources
                    trait_resolver: self.trait_resolver.clone(), // Inherit trait resolver
                    effects: self.effects.clone(), // Inherit effects
                };

                for stmt in body {
                    lambda_checker.check_statement(stmt)?;
                }

                // For now, return a function type based on parameters and a placeholder return type
                // In a full implementation, we'd determine the actual return type
                let param_types: Vec<Type> = params.iter()
                    .map(|param| param.type_annotation.clone())
                    .collect();
                Ok(Type::Function(param_types, Box::new(Type::Infer)))
            },
            Expression::ChannelCreate(element_type) => {
                // ChannelCreate takes a Type directly, not an expression
                // So we just wrap the type in a Channel type
                Ok(Type::Channel(element_type.clone()))
            },
            Expression::ChannelSend(channel_expr, value_expr) => {
                // Check both expressions
                let channel_type = self.check_expression(channel_expr)?;
                let value_type = self.check_expression(value_expr)?;

                // Verify that the channel type is indeed a channel type
                if let Type::Channel(expected_elem_type) = channel_type {
                    // Verify that the value type matches the channel's element type
                    if !self.types_compatible(&value_type, expected_elem_type.as_ref()) {
                        return Err(format!(
                            "Channel send: expected {:?}, found {:?}",
                            expected_elem_type.as_ref(),
                            value_type
                        ));
                    }
                    Ok(Type::Unit) // Channel send returns Unit
                } else {
                    Err(format!("Expected channel type, found {:?}", channel_type))
                }
            },
            Expression::ChannelReceive(channel_expr) => {
                // Check the channel expression
                let channel_type = self.check_expression(channel_expr)?;

                // Verify that the channel type is indeed a channel type
                if let Type::Channel(elem_type) = channel_type {
                    Ok(*elem_type) // Channel receive returns the element type
                } else {
                    Err(format!("Expected channel type, found {:?}", channel_type))
                }
            },
            Expression::Select(select_arms) => {
                // Check each select arm
                for arm in select_arms {
                    match &arm.channel_operation {
                        ChannelOperation::Send { channel, value } => {
                            let channel_type = self.check_expression(channel)?;
                            let value_type = self.check_expression(value)?;

                            if let Type::Channel(expected_elem_type) = channel_type {
                                if !self.types_compatible(&value_type, expected_elem_type.as_ref()) {
                                    return Err(format!(
                                        "Select send: expected {:?}, found {:?}",
                                        expected_elem_type.as_ref(),
                                        value_type
                                    ));
                                }
                            } else {
                                return Err(format!("Expected channel type in select, found {:?}", channel_type));
                            }
                        },
                        ChannelOperation::Receive { channel } => {
                            let channel_type = self.check_expression(channel)?;
                            if !matches!(channel_type, Type::Channel(_)) {
                                return Err(format!("Expected channel type in select, found {:?}", channel_type));
                            }
                        },
                        ChannelOperation::Close { channel } => {
                            let channel_type = self.check_expression(channel)?;
                            if !matches!(channel_type, Type::Channel(_)) {
                                return Err(format!("Expected channel type in select close, found {:?}", channel_type));
                            }
                        },
                    }

                    // Check the body of each arm
                    for stmt in &arm.body {
                        self.check_statement(stmt)?;
                    }
                }

                // For now, return a placeholder type
                Ok(Type::Infer)
            },
            Expression::DestructureAssignment(pattern, value, statement) => {
                // Check the value being destructured
                let value_type = self.check_expression(value)?;

                // Check that the pattern matches the value type
                // For now, we'll just verify that the pattern is valid
                // In a full implementation, we'd check compatibility
                match &**pattern {
                    Pattern::Identifier(_) | Pattern::Wildcard | Pattern::Tuple(_) | Pattern::Array(_) | Pattern::Struct(_, _) | Pattern::Or(_, _) => {
                        // These patterns are valid
                    },
                    _ => {
                        // Other patterns might need more complex type checking
                    }
                }

                // Create a new environment with the variables bound by the pattern
                let mut destructure_env = TypeEnv::new(Some(self.env.clone()));

                // Bind pattern variables to the environment
                self.bind_pattern_variables_to_env(pattern, &mut destructure_env)?;

                // Check the statement with the new environment
                let mut stmt_checker = TypeChecker {
                    env: destructure_env,
                    linear_resources: self.linear_resources.clone(),
                    trait_resolver: self.trait_resolver.clone(),
                    effects: self.effects.clone(),
                };

                stmt_checker.check_statement(statement)?;

                // Copy back any linear resource usage from the statement checker
                for (resource, count) in &stmt_checker.linear_resources {
                    self.linear_resources.insert(resource.clone(), *count);
                }

                // Destructuring assignment evaluates to Unit
                Ok(Type::Unit)
            },
            Expression::MultiLangCall(lang, code) => {
                // For multi-language calls, we need to determine the return type
                // This is challenging since we're calling code in another language
                // For now, we'll return a generic type that represents the result of the foreign call
                // In a full implementation, we'd need to analyze the foreign code to determine its return type

                // Add the effect of performing a foreign language call
                self.effects.insert(Effect::Custom(format!("ForeignCall_{}", lang)));

                // For now, return a generic type - in practice, this would depend on the language and code
                Ok(Type::Infer) // Could be any type depending on the foreign code
            },
            Expression::MultiLangImport(lang, resource, resource_type) => {
                // Handle importing resources from other languages
                // This would typically bring types, functions, or values from the other language into scope

                // Add the effect of importing from another language
                self.effects.insert(Effect::Custom(format!("Import_{}_{}", lang, resource_type.clone().unwrap_or_else(|| "unknown".to_string()))));

                // Return a type representing the imported resource
                // In a real implementation, this would be the actual type of the imported resource
                Ok(Type::Infer)
            },
            Expression::MultiLangIndex(lang, resource) => {
                // Handle indexing resources from other languages
                // This might be for semantic analysis, documentation, or code understanding

                // Add the effect of indexing another language's resources
                self.effects.insert(Effect::Custom(format!("Index_{}", lang)));

                // Return a type representing the indexed information
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
            // Dependent type compatibility
            (Type::Pi(param1, ret1), Type::Pi(param2, ret2)) => {
                // For Pi types, parameters must be compatible and return types must be compatible
                // considering the dependency on the parameter
                self.types_compatible(&param1.type_annotation, &param2.type_annotation) &&
                self.types_compatible(ret1, ret2)
            },
            (Type::Sigma(param1, snd1), Type::Sigma(param2, snd2)) => {
                // For Sigma types, first components must be compatible and second components must be compatible
                self.types_compatible(&param1.type_annotation, &param2.type_annotation) &&
                self.types_compatible(snd1, snd2)
            },
            (Type::Universe(l1), Type::Universe(l2)) => l1 == l2,
            (Type::Equality(ty1, _, _), Type::Equality(ty2, _, _)) => self.types_compatible(ty1, ty2),
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

    /// Checks if a type is a dependent type
    fn is_dependent_type(&self, ty: &Type) -> bool {
        matches!(ty, Type::Pi(_, _) | Type::Sigma(_, _) | Type::Universe(_))
    }

    /// Checks if two types are equivalent
    fn types_equivalent(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Unit, Type::Unit) => true,
            (Type::Array(a), Type::Array(b)) => self.types_equivalent(a, b),
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    false
                } else {
                    t1.iter().zip(t2).all(|(a, b)| self.types_equivalent(a, b))
                }
            },
            (Type::Named(n1), Type::Named(n2)) => n1 == n2,
            (Type::Pi(p1, r1), Type::Pi(p2, r2)) => {
                // For Pi types (dependent function types), check parameter and return types
                self.types_equivalent(&p1.type_annotation, &p2.type_annotation) &&
                self.types_equivalent(r1, r2)
            },
            (Type::Sigma(p1, s1), Type::Sigma(p2, s2)) => {
                // For Sigma types (dependent pairs), check parameter and second component
                self.types_equivalent(&p1.type_annotation, &p2.type_annotation) &&
                self.types_equivalent(s1, s2)
            },
            (Type::Universe(l1), Type::Universe(l2)) => l1 == l2,
            (Type::Linear(t1), Type::Linear(t2)) => self.types_equivalent(t1, t2),
            _ => false,
        }
    }

    /// Checks if a pattern matches a given type
    fn check_pattern_against_type(&self, pattern: &Pattern, expected_type: &Type) -> Result<(), String> {
        match pattern {
            Pattern::Identifier(_) => {
                // An identifier pattern matches any type
                Ok(())
            },
            Pattern::Literal(expr) => {
                // For now, we'll just acknowledge that literal patterns exist
                // In a full implementation, we'd check if the literal type matches expected_type
                Ok(())
            },
            Pattern::Wildcard => {
                // Wildcard matches any type
                Ok(())
            },
            Pattern::Tuple(pattern_items) => {
                // Check if the expected type is a tuple type
                if let Type::Tuple(expected_items) = expected_type {
                    if pattern_items.len() != expected_items.len() {
                        return Err(format!("Tuple pattern has {} elements but expected {}", pattern_items.len(), expected_items.len()));
                    }

                    // Recursively check each element
                    for (pat, exp_type) in pattern_items.iter().zip(expected_items.iter()) {
                        self.check_pattern_against_type(pat, exp_type)?;
                    }

                    Ok(())
                } else {
                    Err(format!("Tuple pattern cannot match non-tuple type {:?}", expected_type))
                }
            },
            Pattern::Array(pattern_items) => {
                // Check if the expected type is an array type
                if let Type::Array(expected_elem_type) = expected_type {
                    // For simplicity, we'll check that all pattern items match the element type
                    for pat in pattern_items {
                        self.check_pattern_against_type(pat, expected_elem_type)?;
                    }

                    Ok(())
                } else {
                    Err(format!("Array pattern cannot match non-array type {:?}", expected_type))
                }
            },
            Pattern::Struct(name, fields) => {
                // For struct patterns, we'd need to check against the actual struct definition
                // For now, we'll just verify that the expected type is a named type with the same name
                if let Type::Named(struct_name) = expected_type {
                    if struct_name != name {
                        return Err(format!("Struct pattern {} does not match type {}", name, struct_name));
                    }
                    // In a full implementation, we'd check each field against the struct's field types
                    Ok(())
                } else {
                    Err(format!("Struct pattern cannot match non-named type {:?}", expected_type))
                }
            },
            Pattern::Or(left, right) => {
                // Both sides of an or pattern should be compatible with the expected type
                self.check_pattern_against_type(left, expected_type)?;
                self.check_pattern_against_type(right, expected_type)?;
                Ok(())
            },
            Pattern::Enum(enum_name, variant_name, sub_patterns) => {
                // For enum patterns, we'd need to check against the actual enum definition
                // For now, we'll just acknowledge the pattern
                if let Some(sub_pats) = sub_patterns {
                    // If there are sub-patterns, we'd need to check them against the variant's data type
                    // This requires looking up the enum definition to know the variant's structure
                    // For now, we'll just check each sub-pattern recursively
                    for sub_pat in sub_pats {
                        // We can't properly type-check without knowing the variant's type
                        // This would require a more sophisticated type environment
                    }
                }
                Ok(())
            },
            Pattern::Range(_, _) => {
                // Range patterns are typically used in match expressions with integer types
                if !matches!(expected_type, Type::Int) {
                    return Err(format!("Range pattern should match integer type, found {:?}", expected_type));
                }
                Ok(())
            },
            Pattern::Irrefutable(inner_pattern) => {
                // An irrefutable pattern should always match
                self.check_pattern_against_type(inner_pattern, expected_type)
            },
            Pattern::Guard(pattern, guard_expr) => {
                // Check the pattern part
                self.check_pattern_against_type(pattern, expected_type)?;

                // For now, we'll just acknowledge that guard expressions exist
                // In a full implementation, we'd check if the guard expression is boolean
                Ok(())
            },
        }
    }

    /// Binds variables in a pattern to an environment
    fn bind_pattern_variables_to_env(&self, pattern: &Pattern, env: &mut TypeEnv) -> Result<(), String> {
        match pattern {
            Pattern::Identifier(name) => {
                // For now, bind with Infer type - in a full implementation, we'd infer the type from context
                env.set_type(name.clone(), Type::Infer);
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
                    self.bind_pattern_variables_to_env(item_pattern, env)?;
                }
                Ok(())
            },
            Pattern::Array(pattern_items) => {
                // Bind each element in the array pattern
                for item_pattern in pattern_items {
                    self.bind_pattern_variables_to_env(item_pattern, env)?;
                }
                Ok(())
            },
            Pattern::Struct(_, fields) => {
                // Bind each field in the struct pattern
                for (_, field_pattern) in fields {
                    self.bind_pattern_variables_to_env(field_pattern, env)?;
                }
                Ok(())
            },
            Pattern::Or(left, right) => {
                // Bind variables from both sides of the or pattern
                self.bind_pattern_variables_to_env(left, env)?;
                self.bind_pattern_variables_to_env(right, env)?;
                Ok(())
            },
            Pattern::Enum(_, _, sub_patterns) => {
                // Bind variables from enum sub-patterns if any
                if let Some(sub_pats) = sub_patterns {
                    for sub_pat in sub_pats {
                        self.bind_pattern_variables_to_env(sub_pat, env)?;
                    }
                }
                Ok(())
            },
            Pattern::Range(_, _) => {
                // Range patterns don't bind variables
                Ok(())
            },
            Pattern::Irrefutable(inner_pattern) => {
                // Bind variables from the inner pattern
                self.bind_pattern_variables_to_env(inner_pattern, env)
            },
            Pattern::Guard(pattern, _) => {
                // Bind variables from the pattern part (guard doesn't bind new variables)
                self.bind_pattern_variables_to_env(pattern, env)
            },
        }
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
            Pattern::Enum(_, _, sub_patterns) => {
                // Bind variables from enum sub-patterns if any
                if let Some(sub_pats) = sub_patterns {
                    for sub_pat in sub_pats {
                        self.bind_pattern_variables(sub_pat, checker)?;
                    }
                }
                Ok(())
            },
            Pattern::Range(_, _) => {
                // Range patterns don't bind variables
                Ok(())
            },
            Pattern::Irrefutable(inner_pattern) => {
                // Bind variables from the inner pattern
                self.bind_pattern_variables(inner_pattern, checker)
            },
            Pattern::Guard(pattern, _) => {
                // Bind variables from the pattern part (guard doesn't bind new variables)
                self.bind_pattern_variables(pattern, checker)
            },
        }
    }

    /// Evaluates a type that might depend on runtime values (for dependent types)
    fn evaluate_dependent_type(&mut self, ty: &Type) -> Result<Type, String> {
        match ty {
            Type::Pi(param, ret_type) => {
                // For Pi types (dependent function types), we need to check that the parameter
                // type is well-formed and the return type depends correctly on the parameter
                let param_type = self.evaluate_dependent_type(&param.type_annotation)?;

                // Create a temporary environment with the parameter to check the return type
                let mut temp_env = TypeEnv::new(Some(self.env.clone()));
                temp_env.set_type(param.name.clone(), param_type);

                // Temporarily update the environment to check the return type
                let old_env = std::mem::replace(&mut self.env, temp_env);

                let evaluated_ret_type = self.evaluate_dependent_type(ret_type)?;

                // Restore the original environment
                self.env = old_env;

                Ok(Type::Pi(
                    param.clone(),
                    Box::new(evaluated_ret_type)
                ))
            },
            Type::Sigma(param, snd_type) => {
                // For Sigma types (dependent pairs), check both components
                let param_type = self.evaluate_dependent_type(&param.type_annotation)?;

                // Create a temporary environment with the parameter to check the second component
                let mut temp_env = TypeEnv::new(Some(self.env.clone()));
                temp_env.set_type(param.name.clone(), param_type);

                // Temporarily update the environment to check the second component
                let old_env = std::mem::replace(&mut self.env, temp_env);

                let evaluated_snd_type = self.evaluate_dependent_type(snd_type)?;

                // Restore the original environment
                self.env = old_env;

                Ok(Type::Sigma(
                    param.clone(),
                    Box::new(evaluated_snd_type)
                ))
            },
            Type::Equality(base_type, _, _) => {
                // For equality types, ensure the base type is well-formed
                let evaluated_base = self.evaluate_dependent_type(base_type)?;
                Ok(Type::Equality(
                    Box::new(evaluated_base),
                    Box::new(Expression::Nil), // Placeholder
                    Box::new(Expression::Nil)  // Placeholder
                ))
            },
            _ => Ok(ty.clone()), // Non-dependent types are returned as-is
        }
    }
}

/// Checks the types in a program
pub fn check_types(program: &Program) -> Result<(), String> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}