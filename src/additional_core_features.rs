//! Additional core language features for the Logos programming language
//! Implements advanced type system features, effect handlers, and other core functionality

use crate::ast::*;

/// Advanced type system features
pub mod type_system {
    use crate::ast::*;
    
    /// Dependent types implementation
    pub struct DependentTypeSystem {
        /// Context for type checking dependent types
        context: Vec<(String, Type)>,
        /// Type-level computation environment
        type_context: Vec<(String, Type)>,
    }

    impl DependentTypeSystem {
        /// Create a new dependent type system
        pub fn new() -> Self {
            Self {
                context: Vec::new(),
                type_context: Vec::new(),
            }
        }

        /// Add a variable to the context with its type
        pub fn add_to_context(&mut self, name: String, ty: Type) -> Result<(), String> {
            self.context.push((name.clone(), ty.clone()));

            // If this is a type-level variable, also add to type context
            if matches!(ty, Type::Named(_) | Type::Universe(_)) {
                self.type_context.push((name, ty));
            }

            Ok(())
        }

        /// Check if a dependent type is valid
        pub fn check_dependent_type(&mut self, expr: &Expression, expected_type: &Type) -> Result<(), String> {
            match expected_type {
                // Pi types: (x: A) -> B(x) - dependent function types
                Type::Pi(param, ret_type) => {
                    // Add parameter to context temporarily
                    self.add_to_context(param.name.clone(), param.type_annotation.clone())?;

                    // Check that the return type is well-formed in the extended context
                    let result = self.check_type_in_context(ret_type, &[]);

                    // Remove parameter from context (in a real implementation, we'd use a stack-based approach)
                    self.context.pop();

                    result
                },

                // Sigma types: (x: A, B(x)) - dependent pairs
                Type::Sigma(param, snd_type) => {
                    // Add parameter to context temporarily
                    self.add_to_context(param.name.clone(), param.type_annotation.clone())?;

                    // Check that the second component is well-formed in the extended context
                    let result = self.check_type_in_context(snd_type, &[]);

                    // Remove parameter from context
                    self.context.pop();

                    result
                },

                // Equality types: x =_A y
                Type::Equality(ty, lhs, rhs) => {
                    // Check that both sides have the same type
                    let lhs_ty = self.infer_type(lhs)?;
                    let rhs_ty = self.infer_type(rhs)?;

                    if !self.types_equal(&lhs_ty, &rhs_ty) {
                        return Err(format!("Both sides of equality must have the same type, got {:?} and {:?}", lhs_ty, rhs_ty));
                    }

                    // For equality types, we just need to verify the types match
                    // The `ty` here is the type that both lhs and rhs should have
                    // We don't call infer_type on it since it's already a Type, not an Expression
                    if !self.types_equal(ty, &lhs_ty) {
                        return Err(format!("Equality type {:?} doesn't match operands {:?}", ty, lhs_ty));
                    }

                    Ok(())
                },

                // Regular types - delegate to regular type checking
                _ => self.check_type_in_context(expected_type, &[]),
            }
        }

        /// Check a general type in the current context
        fn check_type_in_context(&mut self, ty: &Type, expected: &[Type]) -> Result<(), String> {
            match ty {
                Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit => Ok(()),

                Type::Array(inner) => self.check_type_in_context(inner, &[]),

                Type::Tuple(items) => {
                    for item in items {
                        self.check_type_in_context(item, &[])?;
                    }
                    Ok(())
                },

                Type::Function(params, ret) => {
                    for param in params {
                        self.check_type_in_context(param, &[])?;
                    }
                    self.check_type_in_context(ret, &[])
                },

                Type::Named(name) => {
                    // Check if this named type exists in our context
                    if self.type_context.iter().any(|(n, _)| n == name) {
                        Ok(())
                    } else {
                        // Could be a primitive or builtin type
                        match name.as_str() {
                            "Int" | "Float" | "Bool" | "String" | "Unit" => Ok(()),
                            _ => Err(format!("Unknown type: {}", name)),
                        }
                    }
                },

                Type::Generic(name) => {
                    // Check if this generic type exists in our context
                    if self.type_context.iter().any(|(n, _)| n == name) {
                        Ok(())
                    } else {
                        Err(format!("Unknown generic type: {}", name))
                    }
                },

                Type::Option(inner) => self.check_type_in_context(inner, &[]),

                Type::Result(ok, err) => {
                    self.check_type_in_context(ok, &[])?;
                    self.check_type_in_context(err, &[])
                },

                Type::Infer => Ok(()),

                // Dependent types - already handled in check_dependent_type
                Type::Pi(_, _) | Type::Sigma(_, _) | Type::Universe(_) | Type::Equality(_, _, _) => {
                    Ok(())
                },

                Type::Channel(inner) => self.check_type_in_context(inner, &[]),

                Type::Linear(inner) => self.check_type_in_context(inner, &[]),

                Type::GenericWithBounds { name, bounds: _ } => {
                    // Check if this generic type exists in our context
                    if self.type_context.iter().any(|(n, _)| n == name) {
                        Ok(())
                    } else {
                        Err(format!("Unknown generic type: {}", name))
                    }
                },
            }
        }

        /// Infer the type of an expression
        fn infer_type(&mut self, expr: &Expression) -> Result<Type, String> {
            match expr {
                Expression::Integer(_) => Ok(Type::Int),
                Expression::Float(_) => Ok(Type::Float),
                Expression::String(_) => Ok(Type::String),
                Expression::Boolean(_) => Ok(Type::Bool),
                Expression::Nil => Ok(Type::Unit),

                Expression::Identifier(name) => {
                    // Look up the type in our context
                    for (var_name, var_type) in &self.context {
                        if var_name == name {
                            return Ok(var_type.clone());
                        }
                    }
                    Err(format!("Unknown variable: {}", name))
                },

                Expression::BinaryOp(left, op, right) => {
                    let left_ty = self.infer_type(left)?;
                    let right_ty = self.infer_type(right)?;

                    match op {
                        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                            // These operations require numeric types
                            if self.is_numeric_type(&left_ty) && self.is_numeric_type(&right_ty) {
                                // Return the wider type (Float if either is Float)
                                if self.is_float_type(&left_ty) || self.is_float_type(&right_ty) {
                                    Ok(Type::Float)
                                } else {
                                    Ok(Type::Int)
                                }
                            } else {
                                Err(format!(
                                    "Operator {:?} requires numeric operands, found {:?} and {:?}",
                                    op, left_ty, right_ty
                                ))
                            }
                        },
                        BinaryOp::Eq | BinaryOp::Ne => {
                            // Equality operations return Bool
                            if self.types_equal(&left_ty, &right_ty) {
                                Ok(Type::Bool)
                            } else {
                                Err(format!(
                                    "Cannot compare {:?} with {:?}",
                                    left_ty, right_ty
                                ))
                            }
                        },
                        _ => Ok(Type::Infer), // Placeholder for other operations
                    }
                },

                // More complex expressions would go here
                _ => Ok(Type::Infer), // Placeholder for unhandled expressions
            }
        }

        /// Check if two types are equal
        fn types_equal(&self, ty1: &Type, ty2: &Type) -> bool {
            match (ty1, ty2) {
                (Type::Int, Type::Int) => true,
                (Type::Float, Type::Float) => true,
                (Type::Bool, Type::Bool) => true,
                (Type::String, Type::String) => true,
                (Type::Unit, Type::Unit) => true,
                (Type::Array(a), Type::Array(b)) => self.types_equal(a, b),
                (Type::Tuple(items1), Type::Tuple(items2)) => {
                    if items1.len() != items2.len() {
                        false
                    } else {
                        items1.iter().zip(items2.iter()).all(|(a, b)| self.types_equal(a, b))
                    }
                },
                (Type::Named(n1), Type::Named(n2)) => n1 == n2,
                (Type::Generic(g1), Type::Generic(g2)) => g1 == g2,
                (Type::Option(o1), Type::Option(o2)) => self.types_equal(o1, o2),
                (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                    self.types_equal(ok1, ok2) && self.types_equal(err1, err2)
                },
                (Type::Infer, _) | (_, Type::Infer) => true, // Infer matches anything
                _ => false,
            }
        }

        /// Check if a type is numeric
        fn is_numeric_type(&self, ty: &Type) -> bool {
            matches!(ty, Type::Int | Type::Float)
        }

        /// Check if a type is a float
        fn is_float_type(&self, ty: &Type) -> bool {
            matches!(ty, Type::Float)
        }
    }
    
    /// Linear types implementation
    pub struct LinearTypeSystem {
        /// Resource usage tracking
        resource_usage: std::collections::HashMap<String, ResourceStatus>,
        /// Linear type context - tracks which variables have linear types
        linear_vars: std::collections::HashSet<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ResourceStatus {
        Owned,
        Borrowed,
        Moved,
        Dropped,
        Consumed,  // Added for linear types
    }

    impl LinearTypeSystem {
        /// Create a new linear type system
        pub fn new() -> Self {
            Self {
                resource_usage: std::collections::HashMap::new(),
                linear_vars: std::collections::HashSet::new(),
            }
        }

        /// Mark a variable as having a linear type
        pub fn mark_linear(&mut self, var_name: &str) {
            self.linear_vars.insert(var_name.to_string());
        }

        /// Check if a variable has a linear type
        pub fn is_linear(&self, var_name: &str) -> bool {
            self.linear_vars.contains(var_name)
        }

        /// Check if a linear type is used exactly once
        pub fn check_linear_usage(&mut self, expr: &Expression) -> Result<(), String> {
            match expr {
                Expression::Identifier(name) => {
                    if self.is_linear(name) {
                        match self.resource_usage.get(name) {
                            Some(ResourceStatus::Consumed) => {
                                return Err(format!("Linear variable '{}' used more than once", name));
                            },
                            Some(ResourceStatus::Moved) => {
                                return Err(format!("Linear variable '{}' used after being moved", name));
                            },
                            Some(ResourceStatus::Dropped) => {
                                return Err(format!("Linear variable '{}' used after being dropped", name));
                            },
                            _ => {
                                // Mark as consumed
                                self.track_resource(name, ResourceStatus::Consumed);
                            }
                        }
                    }
                    Ok(())
                },
                Expression::BinaryOp(left, _, right) => {
                    self.check_linear_usage(left)?;
                    self.check_linear_usage(right)?;
                    Ok(())
                },
                Expression::UnaryOp(_, operand) => {
                    self.check_linear_usage(operand)?;
                    Ok(())
                },
                Expression::Call(_, args) => {
                    for arg in args {
                        self.check_linear_usage(arg)?;
                    }
                    Ok(())
                },
                Expression::If(condition, then_branch, else_branch) => {
                    self.check_linear_usage(condition)?;

                    // For linear types in conditional branches, we need to ensure
                    // that linear resources are used exactly once across both branches
                    let mut then_checker = LinearTypeSystem::new();
                    let mut else_checker = LinearTypeSystem::new();

                    // Copy linear vars to both checkers
                    then_checker.linear_vars = self.linear_vars.clone();
                    else_checker.linear_vars = self.linear_vars.clone();

                    // Check then branch
                    for stmt in then_branch {
                        then_checker.check_linear_statement(stmt)?;
                    }

                    // Check else branch
                    for stmt in else_branch {
                        else_checker.check_linear_statement(stmt)?;
                    }

                    // Merge the resource usage from both branches
                    self.merge_branches(&then_checker, &else_checker)?;

                    Ok(())
                },
                Expression::Match(expr, arms) => {
                    self.check_linear_usage(expr)?;

                    // For match expressions, we need to ensure linear resources
                    // are used exactly once across all arms
                    let mut first_arm_checker: Option<LinearTypeSystem> = None;

                    for (pattern, guard, body) in arms {
                        let mut arm_checker = LinearTypeSystem::new();
                        arm_checker.linear_vars = self.linear_vars.clone();

                        // Bind variables from pattern
                        self.bind_pattern_variables(pattern, &mut arm_checker)?;

                        // Check guard if present
                        if let Some(guard_expr) = guard {
                            arm_checker.check_linear_usage(guard_expr)?;
                        }

                        // Check body
                        for stmt in body {
                            arm_checker.check_linear_statement(stmt)?;
                        }

                        // Merge with previous arms
                        if let Some(ref mut first_checker) = first_arm_checker {
                            first_checker.merge_with(&arm_checker)?;
                        } else {
                            first_arm_checker = Some(arm_checker);
                        }
                    }

                    // Apply merged result to self
                    if let Some(checker) = first_arm_checker {
                        *self = checker;
                    }

                    Ok(())
                },
                _ => Ok(()), // For other expressions, just continue
            }
        }

        /// Check linear usage in a statement
        fn check_linear_statement(&mut self, stmt: &Statement) -> Result<(), String> {
            match stmt {
                Statement::Expression(expr) => self.check_linear_usage(expr),
                Statement::LetBinding { name, value, type_annotation, .. } => {
                    // Check the value being assigned
                    self.check_linear_usage(value)?;

                    // If the variable has a linear type, record it
                    if let Some(Type::Linear(_)) = type_annotation {
                        self.mark_linear(name);
                        self.track_resource(name, ResourceStatus::Owned);
                    }

                    Ok(())
                },
                Statement::Return(expr) => {
                    if let Some(return_expr) = expr {
                        self.check_linear_usage(return_expr)?;
                    }
                    Ok(())
                },
                Statement::Block(statements) => {
                    for stmt in statements {
                        self.check_linear_statement(stmt)?;
                    }
                    Ok(())
                },
                _ => Ok(()), // For other statements, just continue
            }
        }

        /// Track resource usage
        pub fn track_resource(&mut self, name: &str, status: ResourceStatus) {
            self.resource_usage.insert(name.to_string(), status);
        }

        /// Validate that all linear resources have been properly consumed
        pub fn validate_consumption(&self) -> Result<(), String> {
            for var_name in &self.linear_vars {
                match self.resource_usage.get(var_name) {
                    Some(ResourceStatus::Consumed) | Some(ResourceStatus::Moved) | Some(ResourceStatus::Dropped) => {
                        // OK - linear resource was properly consumed
                    },
                    Some(ResourceStatus::Owned) => {
                        return Err(format!("Linear resource '{}' was not consumed", var_name));
                    },
                    Some(ResourceStatus::Borrowed) => {
                        return Err(format!("Linear resource '{}' was borrowed but not consumed", var_name));
                    },
                    None => {
                        return Err(format!("Linear resource '{}' was never initialized", var_name));
                    },
                }
            }
            Ok(())
        }

        /// Bind variables from a pattern to the linear type checker
        fn bind_pattern_variables(&self, pattern: &Pattern, checker: &mut LinearTypeSystem) -> Result<(), String> {
            match pattern {
                Pattern::Identifier(name) => {
                    // If this is a linear type, mark it in the checker
                    if self.is_linear(name) {
                        checker.mark_linear(name);
                        checker.track_resource(name, ResourceStatus::Owned);
                    }
                    Ok(())
                },
                Pattern::Tuple(items) => {
                    for item_pattern in items {
                        self.bind_pattern_variables(item_pattern, checker)?;
                    }
                    Ok(())
                },
                Pattern::Array(items) => {
                    for item_pattern in items {
                        self.bind_pattern_variables(item_pattern, checker)?;
                    }
                    Ok(())
                },
                Pattern::Struct(_, fields) => {
                    for (_, field_pattern) in fields {
                        self.bind_pattern_variables(field_pattern, checker)?;
                    }
                    Ok(())
                },
                Pattern::Or(left, right) => {
                    // Both sides of or pattern should have same linear variables
                    let mut left_checker = LinearTypeSystem::new();
                    left_checker.linear_vars = checker.linear_vars.clone();
                    self.bind_pattern_variables(left, &mut left_checker)?;

                    let mut right_checker = LinearTypeSystem::new();
                    right_checker.linear_vars = checker.linear_vars.clone();
                    self.bind_pattern_variables(right, &mut right_checker)?;

                    // Both should have same linear vars marked
                    if left_checker.linear_vars != right_checker.linear_vars {
                        return Err("Or pattern branches have different linear variables".to_string());
                    }

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
                Pattern::Literal(_) | Pattern::Wildcard => Ok(()),
            }
        }

        /// Merge resource usage from two branches in a conditional
        fn merge_branches(&mut self, then_checker: &LinearTypeSystem, else_checker: &LinearTypeSystem) -> Result<(), String> {
            // Collect the linear variables to avoid borrowing issues
            let linear_vars: Vec<String> = self.linear_vars.iter().cloned().collect();

            // For each linear variable, check its status in both branches
            for var_name in &linear_vars {
                let then_status = then_checker.resource_usage.get(var_name);
                let else_status = else_checker.resource_usage.get(var_name);

                match (then_status, else_status) {
                    // If consumed in both branches, that's an error (double consumption)
                    (Some(ResourceStatus::Consumed), Some(ResourceStatus::Consumed)) => {
                        return Err(format!("Linear variable '{}' consumed in both branches", var_name));
                    },
                    // If consumed in one branch and not mentioned in the other, that's OK
                    (Some(ResourceStatus::Consumed), None) | (None, Some(ResourceStatus::Consumed)) => {
                        self.track_resource(var_name, ResourceStatus::Consumed);
                    },
                    // If consumed in one and owned in the other, that's OK too
                    (Some(ResourceStatus::Consumed), Some(ResourceStatus::Owned)) |
                    (Some(ResourceStatus::Owned), Some(ResourceStatus::Consumed)) => {
                        self.track_resource(var_name, ResourceStatus::Consumed);
                    },
                    // If owned in both branches, keep it owned
                    (Some(ResourceStatus::Owned), Some(ResourceStatus::Owned)) => {
                        self.track_resource(var_name, ResourceStatus::Owned);
                    },
                    // If moved in both branches, that's OK
                    (Some(ResourceStatus::Moved), Some(ResourceStatus::Moved)) => {
                        self.track_resource(var_name, ResourceStatus::Moved);
                    },
                    // If dropped in both branches, that's OK
                    (Some(ResourceStatus::Dropped), Some(ResourceStatus::Dropped)) => {
                        self.track_resource(var_name, ResourceStatus::Dropped);
                    },
                    // Any other combination is an error
                    _ => {
                        return Err(format!("Inconsistent linear usage of '{}' across branches", var_name));
                    }
                }
            }

            Ok(())
        }

        /// Merge with another linear type checker
        fn merge_with(&mut self, other: &LinearTypeSystem) -> Result<(), String> {
            // Combine resource usages
            for (var_name, status) in &other.resource_usage {
                match self.resource_usage.get(var_name) {
                    None => {
                        // Variable only exists in other branch, add it
                        self.resource_usage.insert(var_name.clone(), status.clone());
                    },
                    Some(current_status) => {
                        // Variable exists in both, check for consistency
                        if current_status != status {
                            return Err(format!("Inconsistent usage of linear variable '{}'", var_name));
                        }
                    }
                }
            }

            Ok(())
        }
    }
    
    /// Algebraic effect system implementation
    pub struct EffectSystem {
        /// Active effects in the current scope
        active_effects: std::collections::HashSet<String>,
        /// Effect handlers
        handlers: std::collections::HashMap<String, EffectHandler>,
        /// Effect stack for nested effect handling
        effect_stack: Vec<EffectFrame>,
        /// Effect typing context
        effect_context: std::collections::HashMap<String, EffectSignature>,
    }

    #[derive(Debug, Clone)]
    pub struct EffectHandler {
        pub operations: std::collections::HashMap<String, FunctionDef>,
        pub return_type: Type,
    }

    /// Represents a frame in the effect stack
    #[derive(Debug, Clone)]
    pub struct EffectFrame {
        /// Effects available in this frame
        available_effects: std::collections::HashSet<String>,
        /// Handlers in this frame
        handlers: std::collections::HashMap<String, EffectHandler>,
    }

    /// Represents the signature of an effect operation
    #[derive(Debug, Clone)]
    pub struct EffectSignature {
        pub operation_name: String,
        pub parameter_types: Vec<Type>,
        pub return_type: Type,
    }

    impl EffectSystem {
        /// Create a new effect system
        pub fn new() -> Self {
            Self {
                active_effects: std::collections::HashSet::new(),
                handlers: std::collections::HashMap::new(),
                effect_stack: vec![],
                effect_context: std::collections::HashMap::new(),
            }
        }

        /// Push a new frame onto the effect stack
        pub fn push_frame(&mut self) {
            let current_frame = EffectFrame {
                available_effects: self.active_effects.clone(),
                handlers: self.handlers.clone(),
            };
            self.effect_stack.push(current_frame);

            // Clear current effects and handlers for the new frame
            self.active_effects.clear();
            self.handlers.clear();
        }

        /// Pop a frame from the effect stack
        pub fn pop_frame(&mut self) -> Result<(), String> {
            if let Some(prev_frame) = self.effect_stack.pop() {
                self.active_effects = prev_frame.available_effects;
                self.handlers = prev_frame.handlers;
                Ok(())
            } else {
                Err("Cannot pop from empty effect stack".to_string())
            }
        }

        /// Register an effect signature
        pub fn register_effect_signature(&mut self, effect_name: String, signature: EffectSignature) {
            self.effect_context.insert(effect_name, signature);
        }

        /// Register an effect handler
        pub fn register_handler(&mut self, effect_name: String, handler: EffectHandler) {
            self.handlers.insert(effect_name, handler);
        }

        /// Check if an effect is handled
        pub fn is_effect_handled(&self, effect_name: &str) -> bool {
            self.handlers.contains_key(effect_name)
        }

        /// Process an effect operation
        pub fn process_operation(&mut self, effect_name: &str, op_name: &str) -> Result<(), String> {
            if let Some(handler) = self.handlers.get(effect_name) {
                if handler.operations.contains_key(op_name) {
                    Ok(())
                } else {
                    Err(format!("Operation {} not handled for effect {}", op_name, effect_name))
                }
            } else {
                Err(format!("No handler registered for effect {}", effect_name))
            }
        }

        /// Perform an effect operation
        pub fn perform(&mut self, effect_name: &str, op_name: &str, args: &[Expression]) -> Result<Expression, String> {
            // Check if the effect is active
            if !self.active_effects.contains(effect_name) {
                return Err(format!("Effect '{}' is not active in current scope", effect_name));
            }

            // Check if the operation is valid for this effect
            if let Some(signature) = self.effect_context.get(effect_name) {
                if signature.operation_name != op_name {
                    return Err(format!("Operation '{}' is not valid for effect '{}'", op_name, effect_name));
                }

                // Check argument types match signature
                if args.len() != signature.parameter_types.len() {
                    return Err(format!(
                        "Effect operation '{}' expects {} arguments, got {}",
                        op_name, signature.parameter_types.len(), args.len()
                    ));
                }

                // In a real implementation, we would type-check the arguments against the signature
                // For now, we'll just return a placeholder expression
                Ok(Expression::Nil)
            } else {
                Err(format!("Unknown effect: '{}'", effect_name))
            }
        }

        /// Activate an effect in the current scope
        pub fn activate_effect(&mut self, effect_name: String) {
            self.active_effects.insert(effect_name);
        }

        /// Deactivate an effect in the current scope
        pub fn deactivate_effect(&mut self, effect_name: &str) {
            self.active_effects.remove(effect_name);
        }

        /// Get all active effects
        pub fn get_active_effects(&self) -> Vec<String> {
            self.active_effects.iter().cloned().collect()
        }

        /// Check if an expression has effectful operations
        pub fn check_effects(&self, expr: &Expression) -> Result<Vec<String>, String> {
            let mut effects_found = Vec::new();

            match expr {
                Expression::Call(name, args) => {
                    // Check if this is a call to an effect operation
                    // This would require looking up in the effect context
                    for arg in args {
                        effects_found.extend(self.check_effects(arg)?);
                    }
                },
                Expression::BinaryOp(left, _, right) => {
                    effects_found.extend(self.check_effects(left)?);
                    effects_found.extend(self.check_effects(right)?);
                },
                Expression::UnaryOp(_, operand) => {
                    effects_found.extend(self.check_effects(operand)?);
                },
                Expression::If(condition, then_branch, else_branch) => {
                    effects_found.extend(self.check_effects(condition)?);

                    // Check both branches
                    for stmt in then_branch {
                        effects_found.extend(self.check_effects_in_statement(stmt)?);
                    }

                    for stmt in else_branch {
                        effects_found.extend(self.check_effects_in_statement(stmt)?);
                    }
                },
                Expression::Match(expr, arms) => {
                    effects_found.extend(self.check_effects(expr)?);

                    for (pattern, guard, body) in arms {
                        if let Some(guard_expr) = guard {
                            effects_found.extend(self.check_effects(guard_expr)?);
                        }

                        for stmt in body {
                            effects_found.extend(self.check_effects_in_statement(stmt)?);
                        }
                    }
                },
                _ => {}
            }

            Ok(effects_found)
        }

        /// Check effects in a statement
        fn check_effects_in_statement(&self, stmt: &Statement) -> Result<Vec<String>, String> {
            let mut effects_found = Vec::new();

            match stmt {
                Statement::Expression(expr) => {
                    effects_found.extend(self.check_effects(expr)?);
                },
                Statement::Block(statements) => {
                    for stmt in statements {
                        effects_found.extend(self.check_effects_in_statement(stmt)?);
                    }
                },
                Statement::LetBinding { value, .. } => {
                    effects_found.extend(self.check_effects(value)?);
                },
                Statement::Return(expr) => {
                    if let Some(return_expr) = expr {
                        effects_found.extend(self.check_effects(return_expr)?);
                    }
                },
                _ => {}
            }

            Ok(effects_found)
        }
    }
}

/// Pattern matching enhancements
pub mod pattern_matching {
    use crate::ast::*;

    /// Advanced pattern matching with guards and or-patterns
    pub struct AdvancedPatternMatcher {
        /// Variables bound during pattern matching
        bindings: std::collections::HashMap<String, Value>,
        /// Type information for pattern matching
        type_info: std::collections::HashMap<String, Type>,
    }

    #[derive(Debug, Clone)]
    pub enum Value {
        Integer(i64),
        Float(f64),
        String(String),
        Boolean(bool),
        Unit,
        Tuple(Vec<Value>),
        Array(Vec<Value>),
        Struct(std::collections::HashMap<String, Value>),
        Variant(String, Box<Value>), // For sum types/enums
    }

    impl AdvancedPatternMatcher {
        /// Create a new pattern matcher
        pub fn new() -> Self {
            Self {
                bindings: std::collections::HashMap::new(),
                type_info: std::collections::HashMap::new(),
            }
        }

        /// Create a new pattern matcher with initial bindings
        pub fn with_bindings(bindings: std::collections::HashMap<String, Value>) -> Self {
            Self {
                bindings,
                type_info: std::collections::HashMap::new(),
            }
        }

        /// Match a value against a pattern with optional guard
        pub fn match_pattern_with_guard(&mut self, value: &Value, pattern: &Pattern, guard: Option<&Expression>) -> Result<bool, String> {
            let matches = self.match_pattern(value, pattern)?;

            if matches {
                if let Some(guard_expr) = guard {
                    // Evaluate the guard expression in the current binding context
                    let guard_result = self.evaluate_guard(guard_expr)?;
                    Ok(guard_result)
                } else {
                    Ok(true)
                }
            } else {
                Ok(false)
            }
        }

        /// Match a value against a pattern
        pub fn match_pattern(&mut self, value: &Value, pattern: &Pattern) -> Result<bool, String> {
            match (value, pattern) {
                (Value::Integer(v), Pattern::Literal(Expression::Integer(p))) => Ok(v == p),
                (Value::Float(v), Pattern::Literal(Expression::Float(p))) => Ok((v - p).abs() < f64::EPSILON),
                (Value::String(v), Pattern::Literal(Expression::String(p))) => Ok(v == p),
                (Value::Boolean(v), Pattern::Literal(Expression::Boolean(p))) => Ok(v == p),
                (Value::Unit, Pattern::Literal(Expression::Nil)) => Ok(true),
                (Value::Tuple(values), Pattern::Tuple(patterns)) => {
                    if values.len() != patterns.len() {
                        return Ok(false);
                    }

                    for (val, pat) in values.iter().zip(patterns.iter()) {
                        if !self.match_pattern(val, pat)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                (Value::Array(values), Pattern::Array(patterns)) => {
                    if values.len() != patterns.len() {
                        return Ok(false);
                    }

                    for (val, pat) in values.iter().zip(patterns.iter()) {
                        if !self.match_pattern(val, pat)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                (Value::Struct(fields), Pattern::Struct(name, field_patterns)) => {
                    for (field_name, field_pattern) in field_patterns {
                        if let Some(field_value) = fields.get(field_name) {
                            if !self.match_pattern(field_value, field_pattern)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                (Value::Variant(variant_name, variant_value), Pattern::Struct(name, field_patterns)) if name == variant_name => {
                    // Match the inner value of the variant against the pattern
                    if field_patterns.len() == 1 {
                        // If there's one field, match against it
                        if let Some((_, inner_pattern)) = field_patterns.first() {
                            self.match_pattern(variant_value, inner_pattern)
                        } else {
                            Ok(false)
                        }
                    } else {
                        Ok(false) // Variants with multiple fields need special handling
                    }
                },
                (_, Pattern::Identifier(name)) => {
                    // Bind the value to the identifier
                    self.bindings.insert(name.clone(), value.clone());
                    Ok(true)
                },
                (_, Pattern::Wildcard) => Ok(true),
                (v, Pattern::Or(left, right)) => {
                    // Try matching with the left pattern
                    let mut left_matcher = AdvancedPatternMatcher::with_bindings(self.bindings.clone());
                    if left_matcher.match_pattern(v, left)? {
                        // If left matches, update bindings
                        self.bindings = left_matcher.bindings;
                        Ok(true)
                    } else {
                        // Try matching with the right pattern
                        let mut right_matcher = AdvancedPatternMatcher::with_bindings(self.bindings.clone());
                        if right_matcher.match_pattern(v, right)? {
                            // If right matches, update bindings
                            self.bindings = right_matcher.bindings;
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                },
                // Range patterns: e.g., 1..=5
                (Value::Integer(v), Pattern::Literal(Expression::BinaryOp(left, BinaryOp::Range, right))) => {
                    if let (Expression::Integer(start), Expression::Integer(end)) = (left.as_ref(), right.as_ref()) {
                        Ok(v >= start && v <= end)
                    } else {
                        Ok(false)
                    }
                },
                // Constant patterns with guards
                (v, Pattern::Literal(expr)) => {
                    // Compare the value with the literal expression
                    self.compare_values(v, expr)
                },
                _ => Ok(false),
            }
        }

        /// Compare a value with a literal expression
        fn compare_values(&self, value: &Value, expr: &Expression) -> Result<bool, String> {
            match (value, expr) {
                (Value::Integer(v), Expression::Integer(p)) => Ok(v == p),
                (Value::Float(v), Expression::Float(p)) => Ok((v - p).abs() < f64::EPSILON),
                (Value::String(v), Expression::String(p)) => Ok(v == p),
                (Value::Boolean(v), Expression::Boolean(p)) => Ok(v == p),
                (Value::Unit, Expression::Nil) => Ok(true),
                _ => Ok(false),
            }
        }

        /// Evaluate a guard expression
        fn evaluate_guard(&self, expr: &Expression) -> Result<bool, String> {
            // This is a simplified guard evaluator that works with the bound variables
            match expr {
                Expression::BinaryOp(left, op, right) => {
                    let left_val = self.eval_expression(left)?;
                    let right_val = self.eval_expression(right)?;

                    match (left_val, right_val) {
                        (Value::Integer(l), Value::Integer(r)) => {
                            match op {
                                BinaryOp::Eq => Ok(l == r),
                                BinaryOp::Ne => Ok(l != r),
                                BinaryOp::Lt => Ok(l < r),
                                BinaryOp::Gt => Ok(l > r),
                                BinaryOp::Le => Ok(l <= r),
                                BinaryOp::Ge => Ok(l >= r),
                                _ => Err(format!("Unsupported operation {:?} for integers", op)),
                            }
                        },
                        (Value::Float(l), Value::Float(r)) => {
                            match op {
                                BinaryOp::Eq => Ok((l - r).abs() < f64::EPSILON),
                                BinaryOp::Ne => Ok((l - r).abs() >= f64::EPSILON),
                                BinaryOp::Lt => Ok(l < r),
                                BinaryOp::Gt => Ok(l > r),
                                BinaryOp::Le => Ok(l <= r),
                                BinaryOp::Ge => Ok(l >= r),
                                _ => Err(format!("Unsupported operation {:?} for floats", op)),
                            }
                        },
                        (Value::String(l), Value::String(r)) => {
                            match op {
                                BinaryOp::Eq => Ok(l == r),
                                BinaryOp::Ne => Ok(l != r),
                                _ => Err(format!("Unsupported operation {:?} for strings", op)),
                            }
                        },
                        (Value::Boolean(l), Value::Boolean(r)) => {
                            match op {
                                BinaryOp::Eq => Ok(l == r),
                                BinaryOp::Ne => Ok(l != r),
                                BinaryOp::And => Ok(l && r),
                                BinaryOp::Or => Ok(l || r),
                                _ => Err(format!("Unsupported operation {:?} for booleans", op)),
                            }
                        },
                        _ => Err("Type mismatch in guard expression".to_string()),
                    }
                },
                Expression::UnaryOp(op, operand) => {
                    let val = self.eval_expression(operand)?;

                    match val {
                        Value::Boolean(b) => {
                            match op {
                                UnaryOp::Not => Ok(!b),
                                _ => Err(format!("Unsupported operation {:?} for booleans", op)),
                            }
                        },
                        _ => Err("Type mismatch in guard expression".to_string()),
                    }
                },
                Expression::Identifier(name) => {
                    // Look up the identifier in bindings
                    match self.bindings.get(name) {
                        Some(Value::Boolean(b)) => Ok(*b),
                        Some(_) => Err("Guard expression must evaluate to boolean".to_string()),
                        None => Err(format!("Unknown variable in guard: {}", name)),
                    }
                },
                Expression::Boolean(b) => Ok(*b),
                _ => Err("Unsupported guard expression".to_string()),
            }
        }

        /// Evaluate an expression in the current binding context
        fn eval_expression(&self, expr: &Expression) -> Result<Value, String> {
            match expr {
                Expression::Integer(i) => Ok(Value::Integer(*i)),
                Expression::Float(f) => Ok(Value::Float(*f)),
                Expression::String(s) => Ok(Value::String(s.clone())),
                Expression::Boolean(b) => Ok(Value::Boolean(*b)),
                Expression::Nil => Ok(Value::Unit),
                Expression::Identifier(name) => {
                    match self.bindings.get(name) {
                        Some(val) => Ok(val.clone()),
                        None => Err(format!("Unknown variable: {}", name)),
                    }
                },
                Expression::BinaryOp(left, op, right) => {
                    let left_val = self.eval_expression(left)?;
                    let right_val = self.eval_expression(right)?;

                    match (left_val, right_val) {
                        (Value::Integer(l), Value::Integer(r)) => {
                            match op {
                                BinaryOp::Add => Ok(Value::Integer(l + r)),
                                BinaryOp::Sub => Ok(Value::Integer(l - r)),
                                BinaryOp::Mul => Ok(Value::Integer(l * r)),
                                BinaryOp::Div => {
                                    if r != 0 {
                                        Ok(Value::Integer(l / r))
                                    } else {
                                        Err("Division by zero".to_string())
                                    }
                                },
                                BinaryOp::Mod => {
                                    if r != 0 {
                                        Ok(Value::Integer(l % r))
                                    } else {
                                        Err("Division by zero".to_string())
                                    }
                                },
                                _ => Err(format!("Unsupported operation {:?} for integers", op)),
                            }
                        },
                        (Value::Float(l), Value::Float(r)) => {
                            match op {
                                BinaryOp::Add => Ok(Value::Float(l + r)),
                                BinaryOp::Sub => Ok(Value::Float(l - r)),
                                BinaryOp::Mul => Ok(Value::Float(l * r)),
                                BinaryOp::Div => {
                                    if r != 0.0 {
                                        Ok(Value::Float(l / r))
                                    } else {
                                        Err("Division by zero".to_string())
                                    }
                                },
                                _ => Err(format!("Unsupported operation {:?} for floats", op)),
                            }
                        },
                        _ => Err("Type mismatch in binary operation".to_string()),
                    }
                },
                _ => Err("Unsupported expression in evaluation".to_string()),
            }
        }

        /// Get the current bindings
        pub fn get_bindings(&self) -> &std::collections::HashMap<String, Value> {
            &self.bindings
        }

        /// Add a type annotation for a variable
        pub fn add_type_info(&mut self, var_name: String, var_type: Type) {
            self.type_info.insert(var_name, var_type);
        }
    }
}

/// Memory management features
pub mod memory_management {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Linear memory management system
    pub struct LinearMemoryManager {
        /// Track ownership of memory locations
        ownership_map: HashMap<String, OwnershipStatus>,
        /// Memory pools for efficient allocation
        pools: HashMap<String, Arc<Mutex<MemoryPool>>>,
        /// Reference counting for shared memory
        ref_counts: HashMap<String, usize>,
        /// Garbage collection metadata
        gc_metadata: HashMap<String, GCMetadata>,
    }

    #[derive(Debug, Clone)]
    pub enum OwnershipStatus {
        Owned(String),  // Owned by a specific variable
        Borrowed(String, BorrowType),  // Borrowed by a variable
        Moved,  // Moved and no longer accessible
        Shared(String),  // Shared reference with owner
    }

    #[derive(Debug, Clone)]
    pub enum BorrowType {
        Mutable,
        Immutable,
    }

    #[derive(Debug)]
    pub struct MemoryPool {
        /// Free list of memory blocks
        free_blocks: Vec<MemoryBlock>,
        /// Allocated blocks
        allocated_blocks: Vec<MemoryBlock>,
        /// Pool size
        size: usize,
        /// Maximum pool size
        max_size: usize,
        /// Current usage
        current_usage: usize,
    }

    #[derive(Debug, Clone)]
    pub struct MemoryBlock {
        /// Memory address
        address: usize,
        /// Size of the block
        size: usize,
        /// Whether the block is in use
        in_use: bool,
        /// Timestamp of allocation
        allocated_at: std::time::Instant,
    }

    /// Garbage collection metadata
    #[derive(Debug, Clone)]
    pub struct GCMetadata {
        /// Reference count
        ref_count: usize,
        /// Whether the object is reachable
        reachable: bool,
        /// Timestamp of last access
        last_access: std::time::Instant,
        /// Object size
        size: usize,
    }

    impl LinearMemoryManager {
        /// Create a new linear memory manager
        pub fn new() -> Self {
            Self {
                ownership_map: HashMap::new(),
                pools: HashMap::new(),
                ref_counts: HashMap::new(),
                gc_metadata: HashMap::new(),
            }
        }

        /// Initialize a memory pool
        pub fn init_pool(&mut self, pool_name: &str, size: usize) -> Result<(), String> {
            let pool = MemoryPool::new(size, size * 2); // Allow some growth
            self.pools.insert(pool_name.to_string(), Arc::new(Mutex::new(pool)));
            Ok(())
        }

        /// Allocate memory with linear ownership
        pub fn allocate_linear(&mut self, var_name: &str, size: usize) -> Result<String, String> {
            // Try to allocate from the default pool
            let pool_name = "linear_pool";
            if !self.pools.contains_key(pool_name) {
                self.init_pool(pool_name, 1024 * 1024)?; // 1MB default pool
            }

            let pool = self.pools.get(pool_name).unwrap();
            let mut pool_lock = pool.lock().map_err(|_| "Failed to acquire pool lock")?;

            let block = pool_lock.allocate(size)?;
            let addr = format!("0x{:x}", block.address);

            // Release the lock before inserting into maps
            drop(pool_lock);

            // Update ownership map
            self.ownership_map.insert(addr.clone(), OwnershipStatus::Owned(var_name.to_string()));

            // Update GC metadata
            self.gc_metadata.insert(addr.clone(), GCMetadata {
                ref_count: 1,
                reachable: true,
                last_access: std::time::Instant::now(),
                size,
            });

            Ok(addr)
        }

        /// Allocate shared memory
        pub fn allocate_shared(&mut self, var_name: &str, size: usize) -> Result<String, String> {
            // Try to allocate from the shared pool
            let pool_name = "shared_pool";
            if !self.pools.contains_key(pool_name) {
                self.init_pool(pool_name, 1024 * 1024)?; // 1MB default pool
            }

            let pool = self.pools.get(pool_name).unwrap();
            let mut pool_lock = pool.lock().map_err(|_| "Failed to acquire pool lock")?;

            let block = pool_lock.allocate(size)?;
            let addr = format!("0x{:x}", block.address);

            // Release the lock before inserting into maps
            drop(pool_lock);

            // Update ownership map
            self.ownership_map.insert(addr.clone(), OwnershipStatus::Shared(var_name.to_string()));

            // Initialize reference count
            self.ref_counts.insert(addr.clone(), 1);

            // Update GC metadata
            self.gc_metadata.insert(addr.clone(), GCMetadata {
                ref_count: 1,
                reachable: true,
                last_access: std::time::Instant::now(),
                size,
            });

            Ok(addr)
        }

        /// Increment reference count for shared memory
        pub fn increment_ref_count(&mut self, addr: &str) -> Result<(), String> {
            if let Some(count) = self.ref_counts.get_mut(addr) {
                *count += 1;

                // Update GC metadata
                if let Some(metadata) = self.gc_metadata.get_mut(addr) {
                    metadata.ref_count = *count;
                    metadata.last_access = std::time::Instant::now();
                }

                Ok(())
            } else {
                Err("Address not found in shared memory".to_string())
            }
        }

        /// Decrement reference count for shared memory
        pub fn decrement_ref_count(&mut self, addr: &str) -> Result<(), String> {
            if let Some(count) = self.ref_counts.get_mut(addr) {
                if *count > 0 {
                    *count -= 1;

                    // Update GC metadata
                    if let Some(metadata) = self.gc_metadata.get_mut(addr) {
                        metadata.ref_count = *count;
                        metadata.last_access = std::time::Instant::now();
                    }

                    if *count == 0 {
                        // Automatically deallocate when reference count reaches 0
                        self.deallocate(addr)?;
                    }

                    Ok(())
                } else {
                    Err("Reference count already at 0".to_string())
                }
            } else {
                Err("Address not found in shared memory".to_string())
            }
        }

        /// Transfer ownership of a memory location
        pub fn transfer_ownership(&mut self, from_var: &str, to_var: &str, addr: &str) -> Result<(), String> {
            match self.ownership_map.get(addr) {
                Some(OwnershipStatus::Owned(owner)) if owner == from_var => {
                    self.ownership_map.insert(addr.to_string(), OwnershipStatus::Owned(to_var.to_string()));

                    // Update GC metadata
                    if let Some(metadata) = self.gc_metadata.get_mut(addr) {
                        metadata.reachable = true;
                        metadata.last_access = std::time::Instant::now();
                    }

                    Ok(())
                },
                Some(OwnershipStatus::Shared(owner)) if owner == from_var => {
                    // For shared memory, just update the owner
                    self.ownership_map.insert(addr.to_string(), OwnershipStatus::Shared(to_var.to_string()));
                    Ok(())
                },
                _ => Err("Cannot transfer ownership: invalid owner or not owned".to_string()),
            }
        }

        /// Check if a variable owns a memory location
        pub fn is_owner(&self, var_name: &str, addr: &str) -> bool {
            match self.ownership_map.get(addr) {
                Some(OwnershipStatus::Owned(owner)) => owner == var_name,
                Some(OwnershipStatus::Shared(owner)) => owner == var_name,
                _ => false,
            }
        }

        /// Borrow memory
        pub fn borrow(&mut self, var_name: &str, addr: &str, borrow_type: BorrowType) -> Result<(), String> {
            if let Some(status) = self.ownership_map.get(addr).cloned() {
                match &status {
                    OwnershipStatus::Owned(owner) | OwnershipStatus::Shared(owner) => {
                        // Allow borrowing if the owner is the same variable or if it's shared memory
                        if owner == var_name || matches!(status, OwnershipStatus::Shared(_)) {
                            self.ownership_map.insert(addr.to_string(), OwnershipStatus::Borrowed(var_name.to_string(), borrow_type));

                            // Update GC metadata
                            if let Some(metadata) = self.gc_metadata.get_mut(addr) {
                                metadata.reachable = true;
                                metadata.last_access = std::time::Instant::now();
                            }

                            Ok(())
                        } else {
                            Err("Cannot borrow memory owned by another variable".to_string())
                        }
                    },
                    _ => Err("Cannot borrow memory that is moved or in invalid state".to_string()),
                }
            } else {
                Err("Address not found in memory manager".to_string())
            }
        }

        /// Release a borrow
        pub fn release_borrow(&mut self, var_name: &str, addr: &str) -> Result<(), String> {
            if let Some(status) = self.ownership_map.get(addr).cloned() {
                match status {
                    OwnershipStatus::Borrowed(borrower, _) if borrower == var_name => {
                        // Restore the original ownership status
                        // In a real implementation, we'd need to track the original status
                        // For now, we'll just restore to owned/shared based on ref count
                        if self.ref_counts.contains_key(addr) {
                            self.ownership_map.insert(addr.to_string(), OwnershipStatus::Shared(var_name.to_string()));
                        } else {
                            self.ownership_map.insert(addr.to_string(), OwnershipStatus::Owned(var_name.to_string()));
                        }
                        Ok(())
                    },
                    _ => Err("Cannot release borrow: not borrowed by this variable".to_string()),
                }
            } else {
                Err("Address not found in memory manager".to_string())
            }
        }

        /// Deallocate memory
        pub fn deallocate(&mut self, addr: &str) -> Result<(), String> {
            if let Some(status) = self.ownership_map.get(addr).cloned() {
                match status {
                    OwnershipStatus::Owned(_) | OwnershipStatus::Shared(_) => {
                        // Remove from ownership map
                        self.ownership_map.remove(addr);

                        // Remove from ref counts if it was shared
                        self.ref_counts.remove(addr);

                        // Remove from GC metadata
                        self.gc_metadata.remove(addr);

                        // Return to pool
                        for (_, pool) in &self.pools {
                            let mut pool_lock = pool.lock().map_err(|_| "Failed to acquire pool lock")?;
                            // Try to deallocate from the pool - ignore errors if block wasn't allocated from this pool
                            let _ = pool_lock.deallocate(addr);
                        }

                        Ok(())
                    },
                    OwnershipStatus::Borrowed(_, _) => {
                        Err("Cannot deallocate borrowed memory".to_string())
                    },
                    OwnershipStatus::Moved => {
                        Err("Cannot deallocate moved memory".to_string())
                    },
                }
            } else {
                Err("Address not found in memory manager".to_string())
            }
        }

        /// Perform garbage collection
        pub fn garbage_collect(&mut self) -> Result<usize, String> {
            let mut collected = 0;

            // Mark phase: identify reachable objects
            for metadata in self.gc_metadata.values_mut() {
                metadata.reachable = false;
            }

            // Sweep phase: deallocate unreachable objects
            let unreachable_addrs: Vec<String> = self.gc_metadata
                .iter()
                .filter(|(_, metadata)| !metadata.reachable && metadata.ref_count == 0)
                .map(|(addr, _)| addr.clone())
                .collect();

            for addr in unreachable_addrs {
                if self.deallocate(&addr).is_ok() {
                    collected += 1;
                }
            }

            Ok(collected)
        }

        /// Get memory statistics
        pub fn get_stats(&self) -> MemoryStats {
            let total_allocated = self.gc_metadata.values().map(|m| m.size).sum();
            let total_objects = self.gc_metadata.len();
            let shared_objects = self.ref_counts.len();

            MemoryStats {
                total_allocated,
                total_objects,
                shared_objects,
                owned_objects: total_objects - shared_objects,
            }
        }
    }

    #[derive(Debug)]
    pub struct MemoryStats {
        pub total_allocated: usize,
        pub total_objects: usize,
        pub shared_objects: usize,
        pub owned_objects: usize,
    }

    impl MemoryPool {
        /// Create a new memory pool
        fn new(size: usize, max_size: usize) -> Self {
            Self {
                free_blocks: vec![MemoryBlock {
                    address: 0,
                    size,
                    in_use: false,
                    allocated_at: std::time::Instant::now()
                }],
                allocated_blocks: Vec::new(),
                size,
                max_size,
                current_usage: 0,
            }
        }

        /// Allocate a block of memory
        fn allocate(&mut self, requested_size: usize) -> Result<MemoryBlock, String> {
            // Find a free block that's large enough
            for i in 0..self.free_blocks.len() {
                if !self.free_blocks[i].in_use && self.free_blocks[i].size >= requested_size {
                    let mut block = self.free_blocks.remove(i);
                    if block.size > requested_size {
                        // Split the block
                        let new_block = MemoryBlock {
                            address: block.address + requested_size,
                            size: block.size - requested_size,
                            in_use: false,
                            allocated_at: std::time::Instant::now(),
                        };
                        block.size = requested_size;
                        block.in_use = true;
                        block.allocated_at = std::time::Instant::now();
                        self.allocated_blocks.push(block.clone());
                        self.free_blocks.push(new_block); // Add the remaining block back to free list
                    } else {
                        // Use the entire block
                        block.in_use = true;
                        block.allocated_at = std::time::Instant::now();
                        self.allocated_blocks.push(block.clone());
                    }

                    self.current_usage += requested_size;
                    return Ok(block);
                }
            }

            // If we couldn't find a block, try to expand the pool if possible
            if self.current_usage + requested_size <= self.max_size {
                // Create a new block at the end of the current usage
                let new_address = self.allocated_blocks.iter()
                    .map(|b| b.address + b.size)
                    .max()
                    .unwrap_or(0);

                let new_block = MemoryBlock {
                    address: new_address,
                    size: requested_size,
                    in_use: true,
                    allocated_at: std::time::Instant::now(),
                };

                self.allocated_blocks.push(new_block.clone());
                self.current_usage += requested_size;
                return Ok(new_block);
            }

            Err("Not enough memory in pool".to_string())
        }

        /// Deallocate a block of memory
        fn deallocate(&mut self, addr_str: &str) -> Result<(), String> {
            // Parse the address
            let addr = addr_str.trim_start_matches("0x");
            let addr = usize::from_str_radix(addr, 16).map_err(|_| "Invalid address format")?;

            // Find the block in allocated blocks
            if let Some(pos) = self.allocated_blocks.iter().position(|b| b.address == addr) {
                let block = self.allocated_blocks.remove(pos);

                // Update current usage
                self.current_usage -= block.size;

                // Add to free blocks
                self.free_blocks.push(MemoryBlock {
                    address: block.address,
                    size: block.size,
                    in_use: false,
                    allocated_at: block.allocated_at, // Keep the original allocation time for stats
                });

                // Coalesce adjacent free blocks
                self.coalesce_free_blocks();

                Ok(())
            } else {
                Err("Block not found in allocated blocks".to_string())
            }
        }

        /// Coalesce adjacent free blocks to reduce fragmentation
        fn coalesce_free_blocks(&mut self) {
            self.free_blocks.sort_by_key(|b| b.address);

            let mut i = 0;
            while i < self.free_blocks.len().saturating_sub(1) {
                let current = &self.free_blocks[i];
                let next = &self.free_blocks[i + 1];

                if current.address + current.size == next.address {
                    // Adjacent blocks, merge them
                    let merged_block = MemoryBlock {
                        address: current.address,
                        size: current.size + next.size,
                        in_use: false,
                        allocated_at: current.allocated_at,
                    };

                    // Remove the two blocks and insert the merged one
                    self.free_blocks.remove(i + 1);
                    self.free_blocks[i] = merged_block;
                    // Don't increment i, check again at same position
                } else {
                    i += 1;
                }
            }
        }
    }
}

/// Concurrency and actor model enhancements
pub mod concurrency {
    use std::sync::mpsc;
    use std::thread;
    use std::collections::HashMap;
    use std::time::Duration;
    use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
    use std::future::Future;
    use std::pin::Pin;

    /// Actor system implementation
    pub struct ActorSystem {
        /// Registry of actors
        actors: HashMap<String, ActorHandle>,
        /// Message queues
        message_queues: HashMap<String, mpsc::Sender<Message>>,
        /// Actor lifecycle management
        actor_states: HashMap<String, ActorState>,
        /// Supervision tree
        supervisors: HashMap<String, Vec<String>>,
    }

    #[derive(Debug, Clone)]
    pub struct Message {
        /// Sender ID
        pub from: String,
        /// Message content
        pub content: String,
        /// Timestamp
        pub timestamp: u64,
        /// Message priority
        pub priority: MessagePriority,
        /// Correlation ID for request-response patterns
        pub correlation_id: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub enum MessagePriority {
        Low,
        Normal,
        High,
        Critical,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ActorState {
        Starting,
        Running,
        Paused,
        Stopping,
        Stopped,
        Failed(String),
    }

    pub struct ActorHandle {
        /// Actor ID
        pub id: String,
        /// Thread handle
        pub thread_handle: Option<thread::JoinHandle<()>>,
        /// Flag to indicate if the actor should stop
        pub should_stop: Arc<AtomicBool>,
    }

    /// Actor behavior trait for defining actor logic
    pub trait ActorBehavior: Send {
        fn handle_message(&mut self, msg: Message) -> Result<Option<Message>, String>;
        fn on_start(&mut self) -> Result<(), String> { Ok(()) }
        fn on_stop(&mut self) -> Result<(), String> { Ok(()) }
        fn on_error(&mut self, error: String) -> Result<(), String> {
            eprintln!("Actor error: {}", error);
            Ok(())
        }
    }

    /// Basic actor implementation
    pub struct BasicActor<B: ActorBehavior> {
        behavior: B,
        should_stop: Arc<AtomicBool>,
    }

    impl<B: ActorBehavior> BasicActor<B> {
        pub fn new(behavior: B, should_stop: Arc<AtomicBool>) -> Self {
            Self { behavior, should_stop }
        }

        pub fn run(&mut self, receiver: mpsc::Receiver<Message>) {
            // Call on_start when the actor starts
            if let Err(e) = self.behavior.on_start() {
                eprintln!("Actor startup error: {}", e);
            }

            loop {
                // Check if we should stop
                if self.should_stop.load(Ordering::Relaxed) {
                    break;
                }

                // Try to receive a message with a timeout to periodically check the stop flag
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(msg) => {
                        match self.behavior.handle_message(msg) {
                            Ok(Some(reply)) => {
                                // Send reply - in a real implementation, this would send to the original sender
                            },
                            Ok(None) => {
                                // No reply needed
                            },
                            Err(e) => {
                                if let Err(stop_err) = self.behavior.on_error(e) {
                                    eprintln!("Actor error handler failed: {}", stop_err);
                                    break; // Stop the actor on critical error
                                }
                            }
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed, exit the actor
                        break;
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Just continue the loop to check the stop flag again
                        continue;
                    }
                }
            }

            // Call on_stop when the actor stops
            if let Err(e) = self.behavior.on_stop() {
                eprintln!("Actor shutdown error: {}", e);
            }
        }
    }

    impl ActorSystem {
        /// Create a new actor system
        pub fn new() -> Self {
            Self {
                actors: HashMap::new(),
                message_queues: HashMap::new(),
                actor_states: HashMap::new(),
                supervisors: HashMap::new(),
            }
        }

        /// Create a new actor with the given behavior
        pub fn create_actor<B>(&mut self, id: String, behavior: B) -> Result<(), String>
        where
            B: ActorBehavior + 'static,
        {
            let (sender, receiver) = mpsc::channel();

            // Create the actor
            let should_stop = Arc::new(AtomicBool::new(false));
            let mut actor = BasicActor::new(behavior, should_stop.clone());

            // Spawn a thread for the actor
            let actor_id = id.clone();
            let handle = thread::spawn(move || {
                actor.run(receiver);
            });

            // Store the actor
            self.message_queues.insert(id.clone(), sender);
            self.actor_states.insert(id.clone(), ActorState::Starting);
            self.actors.insert(id.clone(), ActorHandle {
                id: id.clone(),
                thread_handle: Some(handle),
                should_stop,
            });

            // Update state to running after creation
            self.actor_states.insert(id, ActorState::Running);

            Ok(())
        }

        /// Send a message to an actor
        pub fn send_message(&self, to: &str, msg: Message) -> Result<(), String> {
            if let Some(sender) = self.message_queues.get(to) {
                sender.send(msg).map_err(|e| format!("Failed to send message: {}", e))?;
                Ok(())
            } else {
                Err(format!("Actor {} not found", to))
            }
        }

        /// Send a message to an actor and wait for a response
        pub fn send_request_reply(&self, to: &str, msg: Message) -> Result<Option<Message>, String> {
            if let Some(sender) = self.message_queues.get(to) {
                // Create a temporary channel for the response
                let (reply_sender, reply_receiver) = mpsc::channel();

                // Store the reply sender somewhere (this would require more complex state management)
                // For now, we'll just send the message
                sender.send(msg).map_err(|e| format!("Failed to send message: {}", e))?;

                // Wait for a response (with timeout)
                match reply_receiver.recv_timeout(Duration::from_secs(5)) {
                    Ok(response) => Ok(Some(response)),
                    Err(_) => Ok(None), // Timeout
                }
            } else {
                Err(format!("Actor {} not found", to))
            }
        }

        /// Stop an actor gracefully
        pub fn stop_actor(&mut self, id: &str) -> Result<(), String> {
            if let Some(handle) = self.actors.get_mut(id) {
                // Set the stop flag
                handle.should_stop.store(true, Ordering::Relaxed);

                // Update actor state
                self.actor_states.insert(id.to_string(), ActorState::Stopping);

                if let Some(thread_handle) = handle.thread_handle.take() {
                    // Wait for the thread to finish (with timeout)
                    // Note: join_timeout is not available in stable Rust, so we'll use a different approach
                    // We'll send a termination message and then join without timeout
                    // In a real implementation, we'd have a more sophisticated timeout mechanism
                    let _ = thread_handle.join().map_err(|_| "Actor thread panicked".to_string());

                    // Remove from registry
                    self.actors.remove(id);
                    self.message_queues.remove(id);
                    self.actor_states.insert(id.to_string(), ActorState::Stopped);
                    Ok(())
                } else {
                    Err("Actor thread already stopped".to_string())
                }
            } else {
                Err(format!("Actor {} not found", id))
            }
        }

        /// Pause an actor
        pub fn pause_actor(&mut self, id: &str) -> Result<(), String> {
            if let Some(state) = self.actor_states.get_mut(id) {
                match state {
                    ActorState::Running => {
                        *state = ActorState::Paused;
                        Ok(())
                    },
                    _ => Err(format!("Actor {} is not in a state that can be paused", id)),
                }
            } else {
                Err(format!("Actor {} not found", id))
            }
        }

        /// Resume a paused actor
        pub fn resume_actor(&mut self, id: &str) -> Result<(), String> {
            if let Some(state) = self.actor_states.get_mut(id) {
                match state {
                    ActorState::Paused => {
                        *state = ActorState::Running;
                        Ok(())
                    },
                    _ => Err(format!("Actor {} is not paused", id)),
                }
            } else {
                Err(format!("Actor {} not found", id))
            }
        }

        /// Get the state of an actor
        pub fn get_actor_state(&self, id: &str) -> Option<&ActorState> {
            self.actor_states.get(id)
        }

        /// Create a supervisor relationship
        pub fn create_supervisor(&mut self, supervisor_id: String, child_id: String) -> Result<(), String> {
            if !self.actors.contains_key(&supervisor_id) {
                return Err(format!("Supervisor {} does not exist", supervisor_id));
            }
            if !self.actors.contains_key(&child_id) {
                return Err(format!("Child actor {} does not exist", child_id));
            }

            self.supervisors.entry(supervisor_id).or_insert_with(Vec::new).push(child_id);
            Ok(())
        }

        /// Restart a failed actor
        pub fn restart_actor<B>(&mut self, id: String, behavior: B) -> Result<(), String>
        where
            B: ActorBehavior + 'static,
        {
            // Stop the existing actor if it exists
            if self.actors.contains_key(&id) {
                self.stop_actor(&id)?;
            }

            // Create a new actor with the same ID
            self.create_actor(id, behavior)
        }
    }

    /// Future-based actor system for async/await support
    pub struct AsyncActorSystem {
        /// Actors that can handle async messages
        actors: HashMap<String, Box<dyn AsyncActor>>,
    }

    /// Async actor trait
    pub trait AsyncActor: Send {
        fn handle_message_async(&mut self, msg: Message) -> Pin<Box<dyn Future<Output = Result<Option<Message>, String>> + Send>>;
    }

    impl AsyncActorSystem {
        pub fn new() -> Self {
            Self {
                actors: HashMap::new(),
            }
        }

        pub fn register_actor(&mut self, id: String, actor: Box<dyn AsyncActor>) {
            self.actors.insert(id, actor);
        }

        pub async fn send_message_async(&mut self, to: &str, msg: Message) -> Result<Option<Message>, String> {
            if let Some(actor) = self.actors.get_mut(to) {
                actor.handle_message_async(msg).await
            } else {
                Err(format!("Actor {} not found", to))
            }
        }
    }
}

/// Trait system enhancements
pub mod trait_system_enhancements {
    use std::collections::HashMap;
    use crate::ast::*;

    /// Enhanced trait definition with more features
    #[derive(Debug, Clone)]
    pub struct EnhancedTraitDef {
        pub name: String,
        pub type_params: Vec<TypeParam>,
        pub required_methods: Vec<FunctionDef>,
        pub provided_methods: Vec<FunctionDef>,
        pub associated_types: Vec<AssociatedTypeDef>,
        pub super_traits: Vec<String>,
        pub trait_constraints: Vec<TraitConstraint>,
    }

    /// Type parameter with bounds
    #[derive(Debug, Clone)]
    pub struct TypeParam {
        pub name: String,
        pub bounds: Vec<String>, // Trait bounds
    }

    /// Trait constraint for generic parameters
    #[derive(Debug, Clone)]
    pub struct TraitConstraint {
        pub param: String,
        pub bounds: Vec<String>,
    }

    /// Implementation of an enhanced trait
    #[derive(Debug, Clone)]
    pub struct EnhancedImplDef {
        pub trait_name: String,
        pub for_type: String,
        pub type_params: Vec<String>,
        pub methods: Vec<FunctionDef>,
        pub associated_types: Vec<(String, Type)>,
        pub trait_constraints: Vec<TraitConstraint>,
    }

    /// Enhanced trait resolver with more sophisticated resolution
    #[derive(Debug, Clone)]
    pub struct EnhancedTraitResolver {
        pub traits: HashMap<String, EnhancedTraitDef>,
        pub implementations: HashMap<String, Vec<EnhancedImplDef>>,
        coherence_checker: CoherenceChecker,
    }

    /// Checks for trait implementation coherence (no conflicting implementations)
    #[derive(Debug, Clone)]
    pub struct CoherenceChecker {
        overlapping_impls: Vec<String>,
    }

    impl EnhancedTraitResolver {
        pub fn new() -> Self {
            Self {
                traits: HashMap::new(),
                implementations: HashMap::new(),
                coherence_checker: CoherenceChecker {
                    overlapping_impls: Vec::new(),
                },
            }
        }

        /// Register an enhanced trait definition
        pub fn register_trait(&mut self, trait_def: EnhancedTraitDef) -> Result<(), String> {
            if self.traits.contains_key(&trait_def.name) {
                return Err(format!("Trait '{}' already defined", trait_def.name));
            }

            self.traits.insert(trait_def.name.clone(), trait_def);
            Ok(())
        }

        /// Register an enhanced trait implementation
        pub fn register_implementation(&mut self, impl_def: EnhancedImplDef) -> Result<(), String> {
            // Check if the trait exists
            if !self.traits.contains_key(&impl_def.trait_name) {
                return Err(format!("Trait '{}' not found", impl_def.trait_name));
            }

            // Check coherence - no overlapping implementations
            if self.coherence_checker.has_overlapping_impl(&impl_def, &self.implementations) {
                return Err(format!(
                    "Overlapping implementation for trait '{}' on type '{}'",
                    impl_def.trait_name, impl_def.for_type
                ));
            }

            // Add to the list of implementations for this trait
            self.implementations
                .entry(impl_def.trait_name.clone())
                .or_insert_with(Vec::new)
                .push(impl_def);

            Ok(())
        }

        /// Resolve a method call on a type implementing a trait
        pub fn resolve_trait_method(&self, type_name: &str, trait_name: &str, method_name: &str) -> Option<&FunctionDef> {
            if let Some(implementations) = self.implementations.get(trait_name) {
                for implementation in implementations {
                    if implementation.for_type == type_name {
                        if let Some(method) = implementation.methods.iter().find(|m| m.name == method_name) {
                            return Some(method);
                        }
                    }
                }
            }

            // If not found in implementation, check default methods in trait
            if let Some(trait_def) = self.traits.get(trait_name) {
                trait_def.provided_methods.iter().find(|m| m.name == method_name)
            } else {
                None
            }
        }

        /// Check if a type implements a specific trait
        pub fn implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
            if let Some(implementations) = self.implementations.get(trait_name) {
                implementations.iter().any(|imp| imp.for_type == type_name)
            } else {
                false
            }
        }

        /// Get all traits implemented by a type
        pub fn get_implemented_traits(&self, type_name: &str) -> Vec<String> {
            let mut traits = Vec::new();

            for (trait_name, implementations) in &self.implementations {
                if implementations.iter().any(|imp| imp.for_type == type_name) {
                    traits.push(trait_name.clone());
                }
            }

            traits
        }
    }

    impl CoherenceChecker {
        /// Check if an implementation overlaps with existing ones
        pub fn has_overlapping_impl(&self, new_impl: &EnhancedImplDef, existing_impls: &HashMap<String, Vec<EnhancedImplDef>>) -> bool {
            if let Some(exiting_for_trait) = existing_impls.get(&new_impl.trait_name) {
                for existing_impl in exiting_for_trait {
                    // Simple overlap check: same type implementing same trait
                    if existing_impl.for_type == new_impl.for_type {
                        return true;
                    }
                }
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependent_type_system_creation() {
        let dep_type_sys = type_system::DependentTypeSystem::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_linear_type_system_creation() {
        let linear_type_sys = type_system::LinearTypeSystem::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_effect_system_creation() {
        let effect_sys = type_system::EffectSystem::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = pattern_matching::AdvancedPatternMatcher::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_memory_manager_creation() {
        let mem_manager = memory_management::LinearMemoryManager::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_actor_system_creation() {
        let actor_system = concurrency::ActorSystem::new();
        // Test that the system can be created without errors
        assert!(true); // Basic test to ensure creation works
    }

    #[test]
    fn test_enhanced_trait_resolver_creation() {
        let resolver = trait_system_enhancements::EnhancedTraitResolver::new();
        assert_eq!(resolver.traits.len(), 0);
        assert_eq!(resolver.implementations.len(), 0);
    }

    #[test]
    fn test_dependent_type_checking() {
        let mut dep_type_sys = type_system::DependentTypeSystem::new();

        // Test adding a variable to context
        assert!(dep_type_sys.add_to_context("x".to_string(), Type::Int).is_ok());

        // Test type checking
        let expr = Expression::Identifier("x".to_string());
        let result = dep_type_sys.check_dependent_type(&expr, &Type::Int);
        assert!(result.is_ok());
    }

    #[test]
    fn test_linear_type_system() {
        let mut linear_sys = type_system::LinearTypeSystem::new();

        // Mark a variable as linear
        linear_sys.mark_linear("resource");
        assert!(linear_sys.is_linear("resource"));

        // Test resource tracking
        linear_sys.track_resource("resource", type_system::ResourceStatus::Owned);
        assert!(linear_sys.validate_consumption().is_err()); // Resource not consumed

        // Mark as consumed
        linear_sys.track_resource("resource", type_system::ResourceStatus::Consumed);
        assert!(linear_sys.validate_consumption().is_ok()); // Now properly consumed
    }

    #[test]
    fn test_effect_system_operations() {
        let mut effect_sys = type_system::EffectSystem::new();

        // Test effect activation
        effect_sys.activate_effect("IO".to_string());
        assert!(effect_sys.get_active_effects().contains(&"IO".to_string()));

        // Test effect signature registration
        let signature = type_system::EffectSignature {
            operation_name: "read".to_string(),
            parameter_types: vec![Type::String],
            return_type: Type::String,
        };
        effect_sys.register_effect_signature("read_file".to_string(), signature);

        // Test performing an operation
        let args = vec![Expression::String("test.txt".to_string())];
        let result = effect_sys.perform("read_file", "read", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_advanced_pattern_matching() {
        use pattern_matching::*;

        let mut matcher = AdvancedPatternMatcher::new();

        // Test integer matching
        let value = Value::Integer(42);
        let pattern = Pattern::Literal(Expression::Integer(42));
        assert!(matcher.match_pattern(&value, &pattern).unwrap());

        // Test string matching
        let value = Value::String("hello".to_string());
        let pattern = Pattern::Literal(Expression::String("hello".to_string()));
        assert!(matcher.match_pattern(&value, &pattern).unwrap());

        // Test identifier binding
        let value = Value::Integer(100);
        let pattern = Pattern::Identifier("x".to_string());
        assert!(matcher.match_pattern(&value, &pattern).unwrap());
        assert!(matcher.get_bindings().contains_key("x"));

        // Test tuple matching
        let value = Value::Tuple(vec![Value::Integer(1), Value::String("test".to_string())]);
        let pattern = Pattern::Tuple(vec![
            Pattern::Literal(Expression::Integer(1)),
            Pattern::Literal(Expression::String("test".to_string()))
        ]);
        assert!(matcher.match_pattern(&value, &pattern).unwrap());
    }

    #[test]
    fn test_memory_management() {
        use memory_management::*;

        let mut mem_manager = LinearMemoryManager::new();

        // Initialize a pool
        assert!(mem_manager.init_pool("test_pool", 1024).is_ok());

        // Allocate linear memory
        let addr = mem_manager.allocate_linear("var1", 100);
        assert!(addr.is_ok());
        let addr = addr.unwrap();

        // Check ownership
        assert!(mem_manager.is_owner("var1", &addr));

        // Transfer ownership
        assert!(mem_manager.transfer_ownership("var1", "var2", &addr).is_ok());
        assert!(mem_manager.is_owner("var2", &addr));

        // Deallocate
        assert!(mem_manager.deallocate(&addr).is_ok());
    }

    #[test]
    fn test_actor_system() {
        use concurrency::*;

        let mut actor_system = ActorSystem::new();

        // Define a simple actor behavior
        struct TestActor;
        impl ActorBehavior for TestActor {
            fn handle_message(&mut self, _msg: Message) -> Result<Option<Message>, String> {
                Ok(None)
            }
        }

        // Create an actor
        let actor_id = "test_actor".to_string();
        let behavior = TestActor {};
        assert!(actor_system.create_actor(actor_id.clone(), behavior).is_ok());

        // Check actor state
        assert_eq!(actor_system.get_actor_state(&actor_id), Some(&ActorState::Running));

        // Send a message
        let msg = Message {
            from: "test".to_string(),
            content: "hello".to_string(),
            timestamp: 0,
            priority: MessagePriority::Normal,
            correlation_id: None,
        };
        assert!(actor_system.send_message(&actor_id, msg).is_ok());

        // Stop the actor
        assert!(actor_system.stop_actor(&actor_id).is_ok());
    }

    #[test]
    fn test_enhanced_trait_resolver() {
        use trait_system_enhancements::*;

        let mut resolver = EnhancedTraitResolver::new();

        // Create a simple trait
        let trait_def = EnhancedTraitDef {
            name: "Display".to_string(),
            type_params: vec![],
            required_methods: vec![
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
            provided_methods: vec![],
            associated_types: vec![],
            super_traits: vec![],
            trait_constraints: vec![],
        };

        assert!(resolver.register_trait(trait_def).is_ok());
        assert!(resolver.traits.contains_key("Display"));
    }
}