use crate::ast::*;
use crate::effects::{Effect, EffectSet};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, VariableInfo>,
    functions: HashMap<String, FunctionSignature>,
    // Map of type names to their definitions
    types: HashMap<String, TypeDefinition>,
    // Track which linear variables have been used
    used_linear_vars: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    NamedType(Type),
    GenericType { params: Vec<String> },  // Generic type with parameters
    // Dependent type definitions
    PiType { param: Parameter, return_type: Box<Type> },
    SigmaType { fst_param: Parameter, snd_type: Box<Type> },
    UniverseType(u32),
    EqualityType { ty: Box<Type>, lhs: Box<Expression>, rhs: Box<Expression> },
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub type_: Type,
    pub ownership: OwnershipStatus,
    pub lifetime: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OwnershipStatus {
    Owned,
    Borrowed,
    MutablyBorrowed,
    Shared,
    Linear,  // For linear types - used exactly once
    Moved,   // For tracking moved values
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub type_: Type,
    pub ownership: OwnershipStatus,
    pub lifetime: Option<String>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        let mut env = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            used_linear_vars: HashSet::new(),
        };

        // Pre-register basic types
        env.types.insert("Int".to_string(), TypeDefinition::NamedType(Type::Int));
        env.types.insert("Float".to_string(), TypeDefinition::NamedType(Type::Float));
        env.types.insert("Bool".to_string(), TypeDefinition::NamedType(Type::Bool));
        env.types.insert("String".to_string(), TypeDefinition::NamedType(Type::String));
        env.types.insert("Unit".to_string(), TypeDefinition::NamedType(Type::Unit));

        env
    }

    pub fn insert_variable(&mut self, name: String, type_: Type, ownership: OwnershipStatus, lifetime: Option<String>) {
        let var_info = VariableInfo {
            type_,
            ownership: ownership.clone(),  // Clone ownership to avoid move
            lifetime,
        };
        self.variables.insert(name.clone(), var_info);  // Clone name to avoid move

        // If it's a linear variable, mark it as unused initially
        if matches!(ownership, OwnershipStatus::Linear) {
            self.used_linear_vars.remove(&name);
        }
    }

    pub fn insert_function(&mut self, name: String, signature: FunctionSignature) {
        self.functions.insert(name, signature);
    }

    pub fn insert_type(&mut self, name: String, type_def: TypeDefinition) {
        self.types.insert(name, type_def);
    }

    pub fn mark_linear_var_used(&mut self, name: &str) -> Result<(), String> {
        if let Some(var_info) = self.variables.get(name) {
            if matches!(var_info.ownership, OwnershipStatus::Linear) {
                if self.used_linear_vars.contains(name) {
                    return Err(format!("Linear variable {} used more than once", name));
                } else {
                    self.used_linear_vars.insert(name.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn check_linear_vars_used(&self) -> Result<(), String> {
        for (name, var_info) in &self.variables {
            if matches!(var_info.ownership, OwnershipStatus::Linear) &&
               !self.used_linear_vars.contains(name) {
                return Err(format!("Linear variable {} not used", name));
            }
        }
        Ok(())
    }

    pub fn get_variable_info(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }

    pub fn get_variable_type(&self, name: &str) -> Option<&Type> {
        self.variables.get(name).map(|info| &info.type_)
    }

    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn get_type_definition(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }

    pub fn update_variable_ownership(&mut self, name: &str, ownership: OwnershipStatus) -> Result<(), String> {
        if let Some(var_info) = self.variables.get_mut(name) {
            var_info.ownership = ownership;
            Ok(())
        } else {
            Err(format!("Variable {} not found", name))
        }
    }

}

pub fn check_program(program: &Program) -> Result<(), String> {
    let mut env = TypeEnvironment::new();

    // Pre-populate with built-in functions
    env.insert_function(
        "print".to_string(),
        FunctionSignature {
            parameters: vec![ParameterInfo {
                type_: Type::String,
                ownership: OwnershipStatus::Borrowed,
                lifetime: None,
            }],
            return_type: Type::Unit,
        }
    );

    env.insert_function(
        "len".to_string(),
        FunctionSignature {
            parameters: vec![ParameterInfo {
                type_: Type::array(Type::Named("T".to_string())), // Generic
                ownership: OwnershipStatus::Borrowed,
                lifetime: None,
            }],
            return_type: Type::Int,
        }
    );

    // Process statements with effect tracking
    let mut program_effects = EffectSet::new();

    for statement in &program.statements {
        match statement {
            Statement::Expression(expr) => {
                check_expression(expr, &mut env, &mut program_effects)?;
            },
            Statement::LetBinding { name, type_annotation, value, mutable } => {
                let value_type = check_expression(value, &mut env, &mut program_effects)?;

                if let Some(expected_type) = type_annotation {
                    if !types_match(expected_type, &value_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?}",
                            expected_type, value_type
                        ));
                    }
                }

                // Determine ownership based on context
                let ownership = if *mutable {
                    OwnershipStatus::MutablyBorrowed
                } else {
                    OwnershipStatus::Owned
                };

                env.insert_variable(name.clone(), value_type, ownership, None);
            },
            Statement::ConstBinding { name, type_annotation, value } => {
                let value_type = check_expression(value, &mut env, &mut program_effects)?;

                if let Some(expected_type) = type_annotation {
                    if !types_match(expected_type, &value_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?}",
                            expected_type, value_type
                        ));
                    }
                }

                env.insert_variable(name.clone(), value_type, OwnershipStatus::Owned, None);
            },
            Statement::Function(func) => {
                check_function(func, &mut env)?;
            },
            Statement::Return(expr) => {
                if let Some(return_expr) = expr {
                    check_expression(return_expr, &mut env, &mut program_effects)?;
                }
            },
            Statement::Block(statements) => {
                // Create a new scope for the block
                let mut block_env = env.clone();
                check_statements(statements, &mut block_env, &mut program_effects)?;
            },
            Statement::Class(class_def) => {
                check_class(class_def, &mut env)?;
            },
            Statement::Trait(trait_def) => {
                check_trait(trait_def, &mut env)?;
            },
            Statement::Implementation(impl_def) => {
                check_impl(impl_def, &mut env)?;
            },
            Statement::Actor(actor_def) => {
                check_actor(actor_def, &mut env)?;
            },
            Statement::Effect(effect_def) => {
                check_effect(effect_def, &mut env)?;
            },
            _ => {
                // Other statement types not fully implemented in this example
            }
        }
    }

    // Print out the effects detected in the program (for debugging purposes)
    if !program_effects.is_empty() {
        println!("Program effects: {:?}", program_effects.effects);
    }

    // Check that all linear variables have been used exactly once
    env.check_linear_vars_used()?;

    Ok(())
}

fn check_statement(statement: &Statement, env: &mut TypeEnvironment, effects: &mut EffectSet) -> Result<(), String> {
    match statement {
        Statement::Expression(expr) => {
            check_expression(expr, env, effects)?;
            Ok(())
        },
        Statement::LetBinding { name, type_annotation, value, mutable } => {
            let value_type = check_expression(value, env, effects)?;

            if let Some(expected_type) = type_annotation {
                if !types_match(expected_type, &value_type) {
                    return Err(format!(
                        "Type mismatch: expected {:?}, found {:?}",
                        expected_type, value_type
                    ));
                }
            }

            // Determine ownership based on context
            let ownership = if *mutable {
                OwnershipStatus::MutablyBorrowed
            } else {
                OwnershipStatus::Owned
            };

            env.insert_variable(name.clone(), value_type, ownership, None);
            Ok(())
        },
        Statement::ConstBinding { name, type_annotation, value } => {
            let value_type = check_expression(value, env, effects)?;

            if let Some(expected_type) = type_annotation {
                if !types_match(expected_type, &value_type) {
                    return Err(format!(
                        "Type mismatch: expected {:?}, found {:?}",
                        expected_type, value_type
                    ));
                }
            }

            env.insert_variable(name.clone(), value_type, OwnershipStatus::Owned, None);
            Ok(())
        },
        Statement::Function(func) => {
            check_function(func, env)?;
            Ok(())
        },
        Statement::Return(expr) => {
            if let Some(return_expr) = expr {
                check_expression(return_expr, env, effects)?;
            }
            Ok(())
        },
        Statement::Block(statements) => {
            // Create a new scope for the block
            let mut block_env = env.clone();
            for stmt in statements {
                check_statement(stmt, &mut block_env, effects)?;
            }
            // Update the original environment with any new variables
            // (in a real implementation, block-scoped variables would be handled differently)
            Ok(())
        },
        Statement::Class(class_def) => {
            check_class(class_def, env)?;
            Ok(())
        },
        Statement::Trait(trait_def) => {
            check_trait(trait_def, env)?;
            Ok(())
        },
        Statement::Implementation(impl_def) => {
            check_impl(impl_def, env)?;
            Ok(())
        },
        Statement::Actor(actor_def) => {
            check_actor(actor_def, env)?;
            Ok(())
        },
        Statement::Effect(effect_def) => {
            check_effect(effect_def, env)?;
            Ok(())
        },
        _ => {
            // Other statement types not fully implemented in this example
            Ok(())
        }
    }
}

// Function to check a vector of statements and return the type of the last expression
fn check_statements(statements: &[Statement], env: &mut TypeEnvironment, effects: &mut EffectSet) -> Result<Type, String> {
    if statements.is_empty() {
        return Ok(Type::Unit);
    }

    let mut last_type = Type::Unit;
    for (i, statement) in statements.iter().enumerate() {
        match statement {
            Statement::Expression(expr) => {
                last_type = check_expression(expr, env, effects)?;
            },
            Statement::LetBinding { name, type_annotation, value, mutable } => {
                let value_type = check_expression(value, env, effects)?;

                if let Some(expected_type) = type_annotation {
                    if !types_match(expected_type, &value_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?}",
                            expected_type, value_type
                        ));
                    }
                }

                // Determine ownership based on context
                let ownership = if *mutable {
                    OwnershipStatus::MutablyBorrowed
                } else {
                    OwnershipStatus::Owned
                };

                env.insert_variable(name.clone(), value_type, ownership, None);
                last_type = Type::Unit;
            },
            Statement::ConstBinding { name, type_annotation, value } => {
                let value_type = check_expression(value, env, effects)?;

                if let Some(expected_type) = type_annotation {
                    if !types_match(expected_type, &value_type) {
                        return Err(format!(
                            "Type mismatch: expected {:?}, found {:?}",
                            expected_type, value_type
                        ));
                    }
                }

                env.insert_variable(name.clone(), value_type, OwnershipStatus::Owned, None);
                last_type = Type::Unit;
            },
            Statement::Function(func) => {
                check_function(func, env)?;
                last_type = Type::Unit;
            },
            Statement::Return(expr) => {
                if let Some(return_expr) = expr {
                    check_expression(return_expr, env, effects)?;
                }
                // Return statements don't have a type in the context of a statement sequence
                last_type = Type::Unit;
            },
            Statement::Block(block_statements) => {
                // Create a new scope for the block
                let mut block_env = env.clone();
                last_type = check_statements(block_statements, &mut block_env, effects)?;
            },
            Statement::Class(class_def) => {
                check_class(class_def, env)?;
                last_type = Type::Unit;
            },
            Statement::Trait(trait_def) => {
                check_trait(trait_def, env)?;
                last_type = Type::Unit;
            },
            Statement::Implementation(impl_def) => {
                check_impl(impl_def, env)?;
                last_type = Type::Unit;
            },
            Statement::Actor(actor_def) => {
                check_actor(actor_def, env)?;
                last_type = Type::Unit;
            },
            Statement::Effect(effect_def) => {
                check_effect(effect_def, env)?;
                last_type = Type::Unit;
            },
            Statement::Break | Statement::Continue => {
                last_type = Type::Unit;
            },
        }
    }

    Ok(last_type)
}

fn check_function(func: &FunctionDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Create a new environment for the function body
    let mut func_env = env.clone();

    // Add parameters to the function environment
    for param in &func.parameters {
        let ownership = match &param.ownership_modifier {
            Some(OwnershipModifier::Owned) => OwnershipStatus::Owned,
            Some(OwnershipModifier::Borrowed) => OwnershipStatus::Borrowed,
            Some(OwnershipModifier::MutablyBorrowed) => OwnershipStatus::MutablyBorrowed,
            Some(OwnershipModifier::Shared) => OwnershipStatus::Shared,
            Some(OwnershipModifier::Linear) => OwnershipStatus::Linear,
            Some(OwnershipModifier::Moved) => OwnershipStatus::Moved,
            None => OwnershipStatus::Borrowed, // Default to borrowed for safety
        };

        func_env.insert_variable(
            param.name.clone(),
            param.type_annotation.clone(),
            ownership,
            param.lifetime_annotation.clone()
        );
    }

    // Check the function body with a new effects set (functions have their own effect context)
    let mut function_effects = EffectSet::new();
    for stmt in &func.body {
        check_statement(stmt, &mut func_env, &mut function_effects)?;
    }

    // Check that all linear parameters have been used exactly once in the function body
    func_env.check_linear_vars_used()?;

    // In a real implementation, we would also check that all paths return the correct type
    Ok(())
}

// Updated function that includes effect checking
fn check_expression(expr: &Expression, env: &mut TypeEnvironment, effects: &mut EffectSet) -> Result<Type, String> {
    match expr {
        Expression::Integer(_) => Ok(Type::Int),
        Expression::Float(_) => Ok(Type::Float),
        Expression::String(_) => Ok(Type::String),
        Expression::Boolean(_) => Ok(Type::Bool),
        Expression::Nil => Ok(Type::Named("Nil".to_string())),
        Expression::Identifier(name) => {
            let var_info_opt = env.get_variable_info(name).cloned();
            if let Some(var_info) = var_info_opt {
                // Check ownership rules
                match var_info.ownership {
                    OwnershipStatus::Moved => {
                        return Err(format!("Use after move: variable {} has been moved", name));
                    },
                    OwnershipStatus::Linear => {
                        // For linear types, we need to mark the variable as used
                        // This will check if it's already been used and mark it as used
                        env.mark_linear_var_used(name)?;
                    },
                    OwnershipStatus::MutablyBorrowed => {
                        // Check if we're trying to access while mutably borrowed
                        // This is allowed for reading, but not for mutation
                    },
                    _ => {}
                }
                Ok(var_info.type_.clone())
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        },
        Expression::BinaryOp(left, op, right) => {
            let left_type = check_expression(left, env, effects)?;
            let right_type = check_expression(right, env, effects)?;

            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    // For arithmetic operations, both operands must be numeric
                    if matches!(left_type, Type::Int | Type::Float) &&
                       matches!(right_type, Type::Int | Type::Float) {
                        // Result type: if both are int, result is int; otherwise float
                        if matches!(left_type, Type::Int) && matches!(right_type, Type::Int) {
                            Ok(Type::Int)
                        } else {
                            Ok(Type::Float)
                        }
                    } else {
                        Err(format!("Arithmetic operation requires numeric operands"))
                    }
                },
                BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    // For comparison operations, both operands must be comparable
                    if types_match(&left_type, &right_type) {
                        Ok(Type::Bool)
                    } else {
                        Err(format!("Comparison operation requires operands of the same type"))
                    }
                },
                BinaryOp::And | BinaryOp::Or => {
                    // Logical operations require boolean operands
                    if matches!(left_type, Type::Bool) && matches!(right_type, Type::Bool) {
                        Ok(Type::Bool)
                    } else {
                        Err(format!("Logical operation requires boolean operands"))
                    }
                },
                BinaryOp::PipeForward | BinaryOp::PipeBackward => {
                    // Pipeline operations - the result of the left becomes input to the right
                    // For simplicity, we'll just return the type of the right expression
                    Ok(right_type)
                },
                BinaryOp::Spaceship => {
                    // Spaceship operator returns -1, 0, or 1
                    if types_match(&left_type, &right_type) {
                        Ok(Type::Int)
                    } else {
                        Err(format!("Spaceship operator requires operands of the same type"))
                    }
                },
                BinaryOp::Power => {
                    // Power operation requires numeric operands
                    if matches!(left_type, Type::Int | Type::Float) &&
                       matches!(right_type, Type::Int | Type::Float) {
                        // Result type: if both are int, result is int; otherwise float
                        if matches!(left_type, Type::Int) && matches!(right_type, Type::Int) {
                            Ok(Type::Int)
                        } else {
                            Ok(Type::Float)
                        }
                    } else {
                        Err(format!("Power operation requires numeric operands"))
                    }
                },
                BinaryOp::Range => {
                    // Range operator creates a range from left to right
                    // Both operands should be of numeric types
                    if matches!(left_type, Type::Int | Type::Float) &&
                       matches!(right_type, Type::Int | Type::Float) {
                        // Return a range type (could be a specific range type in a full implementation)
                        Ok(Type::Named("Range".to_string()))
                    } else {
                        Err(format!("Range operation requires numeric operands"))
                    }
                },
            }
        },
        Expression::UnaryOp(op, operand) => {
            let operand_type = check_expression(operand, env, effects)?;

            match op {
                UnaryOp::Neg => {
                    if matches!(operand_type, Type::Int | Type::Float) {
                        Ok(operand_type)
                    } else {
                        Err(format!("Negation requires numeric operand"))
                    }
                },
                UnaryOp::Not => {
                    if matches!(operand_type, Type::Bool) {
                        Ok(Type::Bool)
                    } else {
                        Err(format!("Logical NOT requires boolean operand"))
                    }
                },
                _ => Err(format!("Unary operation not implemented")),
            }
        },
        Expression::Call(name, args) => {
            // Check if this is a call to an effect operation
            if is_effect_operation(name, env) {
                // Add the effect to the effects set
                let effect = get_effect_from_operation(name);
                effects.insert(effect);
            }

            let signature_opt = env.get_function_signature(name).cloned();
            if let Some(signature) = signature_opt {
                if args.len() != signature.parameters.len() {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        name, signature.parameters.len(), args.len()
                    ));
                }

                for (i, (arg, expected_param)) in args.iter().zip(&signature.parameters).enumerate() {
                    let arg_type = check_expression(arg, env, effects)?;
                    if !types_match(&expected_param.type_, &arg_type) {
                        return Err(format!(
                            "Argument {} to function {} has type {:?}, expected {:?}",
                            i, name, arg_type, expected_param.type_
                        ));
                    }

                    // Check ownership compatibility
                    if !ownership_compatible(&expected_param.ownership, arg) {
                        return Err(format!(
                            "Ownership mismatch for argument {}: expected {:?}, got incompatible ownership",
                            i, expected_param.ownership
                        ));
                    }
                }

                Ok(signature.return_type)
            } else {
                // Check if this might be a Pi-type application
                let func_type = check_expression(&Expression::Identifier(name.clone()), env, effects)?;

                // Handle dependent function application
                if let Type::Pi(param, return_type) = &func_type {
                    if args.len() != 1 {
                        return Err("Dependent function application expects exactly one argument".to_string());
                    }

                    let arg_type = check_expression(&args[0], env, effects)?;
                    if !types_match(&param.type_annotation, &arg_type) {
                        return Err(format!(
                            "Dependent function argument has type {:?}, expected {:?}",
                            arg_type, param.type_annotation
                        ));
                    }

                    // Substitute the argument for the parameter in the return type
                    // This is a simplified substitution - in a full implementation, we'd need proper substitution
                    Ok(*return_type.clone())
                } else {
                    Err(format!("Undefined function: {}", name))
                }
            }
        },
        Expression::MethodCall(obj, method, args) => {
            // For now, just check the object and arguments
            check_expression(obj, env, effects)?;
            for arg in args {
                check_expression(arg, env, effects)?;
            }
            // Return type would depend on the method - simplified for this example
            Ok(Type::Named("MethodCallResult".to_string()))
        },
        Expression::FieldAccess(obj, field) => {
            // For now, just check the object
            check_expression(obj, env, effects)?;
            // Return type would depend on the field - simplified for this example
            Ok(Type::Named("FieldAccessType".to_string()))
        },
        Expression::If(condition, then_branch, else_branch) => {
            let cond_type = check_expression(condition, env, effects)?;
            if !matches!(cond_type, Type::Bool) {
                return Err(format!("If condition must be boolean, got {:?}", cond_type));
            }

            // Check both branches and ensure they return the same type
            // For simplicity, we'll just check that both branches are valid
            let mut then_env = env.clone();
            for stmt in then_branch {
                check_statement(stmt, &mut then_env, effects)?;
            }

            let mut else_env = env.clone();
            for stmt in else_branch {
                check_statement(stmt, &mut else_env, effects)?;
            }

            // In a real implementation, we'd check that both branches return the same type
            Ok(Type::Unit) // Simplified
        },
        Expression::Match(expr, arms) => {
            // Check the expression being matched
            let match_expr_type = check_expression(expr, env, effects)?;

            // Check each arm and ensure pattern matches the expression type
            let mut arm_types = Vec::new();

            for (pattern, guard, body) in arms {
                // Create a new environment for this arm with pattern bindings
                let mut arm_env = env.clone();

                // Check that the pattern is compatible with the matched expression type
                check_pattern(pattern, &match_expr_type, &mut arm_env, effects)?;

                // Check the guard expression if present
                if let Some(guard_expr) = guard {
                    let guard_type = check_expression(guard_expr, &mut arm_env, effects)?;
                    // Guard expressions should evaluate to boolean
                    if !matches!(guard_type, Type::Bool) {
                        return Err("Guard expression must evaluate to boolean".to_string());
                    }
                }

                // Check the body of the arm
                let mut body_types = Vec::new();
                for stmt in body {
                    // For expression statements, we can get the resulting type
                    if let Statement::Expression(expr) = stmt {
                        let stmt_type = check_expression(expr, &mut arm_env, effects)?;
                        body_types.push(stmt_type);
                    } else {
                        check_statement(stmt, &mut arm_env, effects)?;
                    }
                }

                // The type of the arm is the type of its last expression, or Unit
                let arm_result_type = if !body_types.is_empty() {
                    body_types.last().unwrap().clone()
                } else {
                    Type::Unit
                };

                arm_types.push(arm_result_type);
            }

            // In a real implementation, we'd check that all arms return the same type
            // For now, we'll just return the type of the first arm if available
            if !arm_types.is_empty() {
                Ok(arm_types[0].clone())
            } else {
                Ok(Type::Unit)
            }
        },
        Expression::Lambda(params, body) => {
            // Create a new environment for the lambda
            let mut lambda_env = env.clone();

            // Add parameters to the lambda environment
            for param in params {
                let ownership = match &param.ownership_modifier {
                    Some(OwnershipModifier::Owned) => OwnershipStatus::Owned,
                    Some(OwnershipModifier::Borrowed) => OwnershipStatus::Borrowed,
                    Some(OwnershipModifier::MutablyBorrowed) => OwnershipStatus::MutablyBorrowed,
                    Some(OwnershipModifier::Shared) => OwnershipStatus::Shared,
                    Some(OwnershipModifier::Linear) => OwnershipStatus::Linear,
                    Some(OwnershipModifier::Moved) => OwnershipStatus::Moved,
                    None => OwnershipStatus::Borrowed, // Default to borrowed for safety
                };

                lambda_env.insert_variable(
                    param.name.clone(),
                    param.type_annotation.clone(),
                    ownership,
                    param.lifetime_annotation.clone()
                );
            }

            // Check the lambda body
            for stmt in body {
                check_statement(stmt, &mut lambda_env, effects)?;
            }

            // If there's exactly one parameter, we could have a Pi type (dependent function)
            if params.len() == 1 {
                let param = &params[0];
                // For now, return a Pi type if we have a single parameter
                Ok(Type::Pi(
                    Box::new(param.clone()),
                    Box::new(Type::Unit) // Simplified - in reality, this would be the return type of the lambda
                ))
            } else {
                // Return a function type for multiple parameters
                Ok(Type::Function(
                    params.iter().map(|p| p.type_annotation.clone()).collect(),
                    Box::new(Type::Unit) // Simplified
                ))
            }
        },
        Expression::BlockExpr(statements) => {
            // Create a new environment for the block
            let mut block_env = env.clone();

            for stmt in statements {
                check_statement(stmt, &mut block_env, effects)?;
            }

            // In a real implementation, we'd return the type of the last expression
            Ok(Type::Unit) // Simplified
        },
        Expression::Tuple(elements) => {
            let mut elem_types = Vec::new();
            for elem in elements {
                elem_types.push(check_expression(elem, env, effects)?);
            }
            Ok(Type::Tuple(elem_types))
        },
        Expression::Spawn(actor_name, args) => {
            // For now, spawning an actor returns an Actor handle type
            // In a full implementation, we'd look up the actor definition
            // and validate the arguments match the actor's constructor
            for arg in args {
                check_expression(arg, env, effects)?;
            }
            // Return an Actor type (we'll define this as a generic Actor type for now)
            Ok(Type::Named(format!("Actor_{}", actor_name)))
        },
        Expression::Send(actor_expr, message_expr) => {
            // Check that both the actor and message expressions are valid
            let actor_type = check_expression(actor_expr, env, effects)?;
            let message_type = check_expression(message_expr, env, effects)?;

            // In a real implementation, we'd validate that the message type
            // is compatible with the actor's accepted message types
            // For now, we just check that both expressions are valid

            // Sending a message typically returns a Unit type
            Ok(Type::Unit)
        },
        Expression::Receive => {
            // Receiving a message would return the type of the received message
            // For now, we'll return a generic type
            // In a real implementation, this would be handled in the actor context
            Ok(Type::Named("ReceivedMessage".to_string()))
        },
        Expression::MultiLangCall(lang, code) => {
            // For multi-language calls, we'll return a generic type
            // In a real implementation, this would depend on the language and code
            Ok(Type::Named(format!("{}Result", lang)))
        },
        Expression::MultiLangImport(lang, module, alias) => {
            // For multi-language imports, we return a module/type representing the imported functionality
            // The exact type would depend on what's available in the external module
            Ok(Type::Named(format!("{}Module_{}", lang, module.replace(".", "_"))))
        },
        Expression::MultiLangIndex(lang, resource) => {
            // For indexing resources from other languages, return an indexed type
            let clean_resource = resource.replace('.', "_").replace('/', "_").replace('\\', "_").replace('-', "_");
            Ok(Type::Named(format!("Indexed{}_{}", lang, clean_resource)))
        },
        Expression::InterpolatedString(parts) => {
            // For interpolated strings, the result is always a string
            // Evaluate the types of interpolated expressions for type checking purposes
            for part in parts {
                match part {
                    StringPart::Literal(_) => {}, // Nothing to type check for literals
                    StringPart::Interpolated(expr) => {
                        // Type check the interpolated expression
                        check_expression(expr, env, effects)?;
                    }
                }
            }
            Ok(Type::String) // Interpolated strings always result in string type
        },
        Expression::LambdaSimple(params, body) => {
            // For simple lambda expressions |args| expr
            // Create a new environment for the lambda
            let mut lambda_env = env.clone();

            // Add parameters to the lambda environment (with inferred types for now)
            for param in params {
                lambda_env.insert_variable(
                    param.clone(),
                    Type::Infer, // For now, infer the type
                    OwnershipStatus::Borrowed, // Default to borrowed for safety
                    None
                );
            }

            // Check the lambda body in the extended environment
            let body_type = check_expression(body, &mut lambda_env, effects)?;

            // Return a function type
            Ok(Type::Function(
                params.iter().map(|_| Type::Infer).collect(), // Parameter types (inferred)
                Box::new(body_type) // Return type
            ))
        },
        Expression::Pipeline(start_expr, funcs) => {
            // For pipeline expressions: start |> func1 |> func2
            // Type check the start expression
            let mut current_type = check_expression(start_expr, env, effects)?;

            // Type check each function in the pipeline
            for func in funcs {
                // Get the function type
                let func_type = check_expression(func, env, effects)?;

                // In a real implementation, we'd verify that the function accepts the current type
                // For now, we'll just return the function's return type
                // This is a simplification - in a full implementation, we'd check function signatures
                current_type = func_type; // Simplified - in reality, this would be the return type of applying the function
            }

            Ok(current_type)
        },
        Expression::BackPipeline(end_expr, funcs) => {
            // For backward pipeline expressions: funcN <| ... <| func1 <| start
            // This is equivalent to funcN(...(func1(start))...)
            // Type check the starting expression
            let mut current_type = check_expression(end_expr, env, effects)?;

            // Type check each function in reverse order
            for func in funcs {
                let func_type = check_expression(func, env, effects)?;
                // In a real implementation, we'd verify proper composition
                current_type = func_type; // Simplified
            }

            Ok(current_type)
        },
        Expression::DestructureAssignment(pattern, expr, stmt) => {
            // For destructure assignment: let pat = expr in stmt
            let expr_type = check_expression(expr, env, effects)?;

            // Check that the pattern matches the expression type
            check_pattern(pattern, &expr_type, env, effects)?;

            // Create a new environment with pattern bindings for checking the statement
            let mut stmt_env = env.clone();
            bind_pattern_variables(pattern, &expr_type, &mut stmt_env)?;

            // Check the statement in the context where pattern variables are bound
            check_statement(stmt, &mut stmt_env, effects)?;

            Ok(Type::Unit) // Destructure assignment returns Unit
        },
        // CSP-style channel operations
        Expression::ChannelCreate(channel_type) => {
            // Creating a channel of the specified type
            // Validate the channel element type
            validate_type(channel_type, env)?;
            Ok(Type::Channel(channel_type.clone()))
        }
        Expression::ChannelSend(channel_expr, value_expr) => {
            // Type check the channel expression
            let channel_type = check_expression(channel_expr, env, effects)?;

            // Type check the value to be sent
            let value_type = check_expression(value_expr, env, effects)?;

            // Verify that the channel expression has a channel type
            if let Type::Channel(elem_type) = &channel_type {
                // Verify that the value type matches the channel element type
                if !type_equal(&value_type, elem_type, env) {
                    return Err(format!(
                        "Channel send type mismatch: expected {:?}, got {:?}",
                        elem_type, value_type
                    ));
                }
            } else {
                return Err(format!(
                    "Channel send requires channel type, got {:?}",
                    channel_type
                ));
            }

            // Channel send returns Unit
            Ok(Type::Unit)
        }
        Expression::ChannelReceive(channel_expr) => {
            // Type check the channel expression
            let channel_type = check_expression(channel_expr, env, effects)?;

            // Verify that the channel expression has a channel type
            if let Type::Channel(elem_type) = channel_type {
                // Channel receive returns the element type of the channel
                Ok(*elem_type)
            } else {
                Err(format!(
                    "Channel receive requires channel type, got {:?}",
                    channel_type
                ))
            }
        }
        Expression::ChannelClose(channel_expr) => {
            // Type check the channel expression
            let channel_type = check_expression(channel_expr, env, effects)?;

            // Verify that the channel expression has a channel type
            if !matches!(channel_type, Type::Channel(_)) {
                return Err(format!(
                    "Channel close requires channel type, got {:?}",
                    channel_type
                ));
            }

            // Channel close returns Unit
            Ok(Type::Unit)
        }
        Expression::Select(select_arms) => {
            // Type check each select arm
            for arm in select_arms {
                // Check the channel operation in the arm
                match &arm.channel_operation {
                    ChannelOperation::Send { channel, value } => {
                        // Type check channel and value
                        let channel_type = check_expression(channel, env, effects)?;
                        let value_type = check_expression(value, env, effects)?;

                        // Verify channel type and value compatibility
                        if let Type::Channel(elem_type) = &channel_type {
                            if !type_equal(&value_type, elem_type, env) {
                                return Err(format!(
                                    "Select send type mismatch: expected {:?}, got {:?}",
                                    elem_type, value_type
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Select send requires channel type, got {:?}",
                                channel_type
                            ));
                        }
                    }
                    ChannelOperation::Receive { channel } => {
                        // Type check the channel
                        let channel_type = check_expression(channel, env, effects)?;

                        // Verify it's a channel type
                        if !matches!(channel_type, Type::Channel(_)) {
                            return Err(format!(
                                "Select receive requires channel type, got {:?}",
                                channel_type
                            ));
                        }
                    }
                }

                // Type check the body of the arm
                let mut arm_env = env.clone();
                // If there's a pattern, bind it to the environment
                if let Some(pattern) = &arm.pattern {
                    // For receive operations, we need to determine the type to bind
                    // This is a simplified approach - in a full implementation, we'd
                    // determine the type based on the channel operation
                    match &arm.channel_operation {
                        ChannelOperation::Receive { channel } => {
                            let channel_type = check_expression(channel, env, effects)?;
                            if let Type::Channel(elem_type) = channel_type {
                                bind_pattern_types(pattern, &*elem_type, &mut arm_env)?;
                            }
                        }
                        ChannelOperation::Send { .. } => {
                            // For send operations, there's typically no value to bind
                            // But we still need to validate the pattern
                            bind_pattern_types(pattern, &Type::Unit, &mut arm_env)?;
                        }
                    }
                }

                // Type check the body statements
                for stmt in &arm.body {
                    check_statement(stmt, &mut arm_env, effects)?;
                }
            }

            // Select expression returns Unit (in Go-like semantics)
            Ok(Type::Unit)
        },
    }
}

fn check_class(class_def: &ClassDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Create a new environment for the class scope
    let mut class_env = env.clone();

    // Check field types
    for field in &class_def.fields {
        validate_type(        // Validate that the field type existsfield.type_annotation, env)?;
        validate_type(validate_type(validate_type(field.type_annotation, env)field.type_annotation, env)?field.type_annotation, env)?;
    }

    // Check method types
    for method in &class_def.methods {
        check_function(method, &mut class_env)?;
    }

    // Register the class type in the environment
    // In a real implementation, we'd register the class type globally
    Ok(())
}

fn check_trait(trait_def: &TraitDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Check that all methods in the trait have valid signatures
    for method in &trait_def.methods {
        // For trait methods, we only check the signature, not the implementation
        // (since trait methods might not have bodies)
        for param in &method.parameters {
            validate_type(&param.type_annotation, env)?;
        }

        if let Some(return_type) = &method.return_type {
            validate_type(return_type, env)?;
        }
    }

    Ok(())
}

fn check_impl(impl_def: &ImplDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Find the trait being implemented
    // In a real implementation, we'd look up the trait definition

    // Check that the implementation type exists
    // For now, we'll just validate that it's a valid type
    // This would require more sophisticated type lookup in practice

    // Check that all required methods are implemented
    // In a real implementation, we'd compare against the trait definition

    // Check each method in the implementation
    for method in &impl_def.methods {
        check_function(method, env)?;
    }

    Ok(())
}

fn check_actor(actor_def: &ActorDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Create a new environment for the actor scope
    let mut actor_env = env.clone();

    // Check state field types
    for (field_name, field.type_annotation) in &actor_def.state {
    for field in         validate_type(validate_type(validate_type(field.type_annotation, env)field.type_annotation, env)?field.type_annotation, env)?;actor_def.state {
        validate_type(        // Add state fields to the actor environmentfield.type_annotation, env)?;
        actor_env.insert_variable(
            field_name.clone(),
            field.name.clone(),
            field.type_annotation.clone(),
            None
        );
    }

    // Check handler function types
    for handler in &actor_def.handlers {
        check_function(handler, &mut actor_env)?;
    }

    Ok(())
}

fn check_effect(effect_def: &EffectDef, env: &mut TypeEnvironment) -> Result<(), String> {
    // Check operation types
    for operation in &effect_def.operations {
        // For effect operations, we check the signature and register the effect operation
        for param in &operation.parameters {
            validate_type(&param.type_annotation, env)?;
        }

        if let Some(return_type) = &operation.return_type {
            validate_type(return_type, env)?;
        }

        // Add the operation to the environment as a callable
        let operation_type = Type::Function(
            operation.parameters.iter().map(|p| p.type_annotation.clone()).collect(),
            Box::new(operation.return_type.clone().unwrap_or(Type::Unit))
        );
        env.insert_function(operation.name.clone(), FunctionSignature {
            parameters: operation.parameters.iter().map(|p| ParameterInfo {
                type_: p.type_annotation.clone(),
                ownership: OwnershipStatus::Borrowed, // Default ownership
                lifetime: None,
            }).collect(),
            return_type: operation.return_type.clone().unwrap_or(Type::Unit),
        });
    }

    // Register the effect in the environment as a named type
    env.insert_type(effect_def.name.clone(), TypeDefinition::NamedType(Type::Named(effect_def.name.clone())));

    Ok(())
}

// Function to check effect polymorphism and typing
fn check_effect_polymorphism(expr: &Expression, env: &mut TypeEnvironment, expected_effects: &mut EffectSet) -> Result<Type, String> {
    match expr {
        Expression::Call(name, args) => {
            // Check if this is a call to an effect operation
            if is_effect_operation(name, env) {
                // Add the effect to the expected effects set
                let effect = get_effect_from_operation(name);
                expected_effects.insert(effect);
            }

            // Type check the call normally
            let mut arg_types = Vec::new();
            for arg in args {
                arg_types.push(check_expression(arg, env, expected_effects)?);
            }

            // Look up the function signature in the environment
            if let Some(func_sig) = env.get_function_signature(name) {
                if arg_types.len() != func_sig.parameters.len() {
                    return Err(format!("Function {} expects {} arguments, got {}", name, func_sig.parameters.len(), arg_types.len()));
                }

                for (i, (arg_type, param_info)) in arg_types.iter().zip(func_sig.parameters.iter()).enumerate() {
                    if !type_equal(arg_type, &param_info.type_, env) {
                        return Err(format!("Argument {} of function {} has type {:?}, expected {:?}", i + 1, name, arg_type, param_info.type_));
                    }
                }

                Ok(func_sig.return_type.clone())
            } else {
                Err(format!("Function {} not found", name))
            }
        }
        _ => check_expression(expr, env, expected_effects)
    }
}

// Helper function to check if an identifier is an effect operation
fn is_effect_operation(name: &str, env: &TypeEnvironment) -> bool {
    // Check if the name exists in the environment and corresponds to an effect operation
    // In a real implementation, this would check the effect registry
    // For now, we'll use a simple heuristic based on naming conventions
    name.contains("effect") || name.starts_with("perform_") || name.starts_with("handle_") ||
    env.get_variable_info(name).is_some() ||
    env.get_function_signature(name).is_some() ||
    env.get_type_definition(name).is_some()
}

// Helper function to get effect from operation name
fn get_effect_from_operation(name: &str) -> Effect {
    // In a real implementation, this would map operation names to effects
    // For now, we'll categorize based on naming convention
    if name.contains("io") || name.contains("file") || name.contains("network") {
        Effect::IO
    } else if name.contains("exception") || name.contains("error") {
        Effect::Exception
    } else if name.contains("state") {
        Effect::State("Generic".to_string())
    } else if name.contains("reader") {
        Effect::Reader("Generic".to_string())
    } else if name.contains("writer") {
        Effect::Writer("Generic".to_string())
    } else {
        Effect::Custom(name.to_string())
    }
}

// Enhanced type checking function that tracks effects
fn check_expression_with_effects(expr: &Expression, env: &mut TypeEnvironment) -> Result<(Type, EffectSet), String> {
    let mut effects = EffectSet::new();
    let ty = check_expression(expr, env, &mut effects)?;
    Ok((ty, effects))
}

// Function to get the universe level of a type
fn get_universe_level(type_: &Type, env: &TypeEnvironment) -> Result<u32, String> {
    match type_ {
        Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Infer => Ok(0), // All base types in Type_0
        Type::Array(inner) => get_universe_level(inner, env),
        Type::Tuple(types) => {
            // The universe level of a tuple is the maximum of its elements
            let mut max_level = 0;
            for t in types {
                let level = get_universe_level(t, env)?;
                if level > max_level {
                    max_level = level;
                }
            }
            Ok(max_level)
        },
        Type::Function(params, return_type) => {
            // Function types live in the universe of their return type
            get_universe_level(return_type, env)
        },
        Type::Pi(param, return_type) => {
            // Pi types: if param.type_annotation : Type_i and return_type : Type_j,
            // then Pi type : max(i, j)
            let param_level = get_universe_level(&param.type_annotation, env)?;
            // Create a new environment with the parameter bound to check return type
            let mut pi_env = env.clone();
            pi_env.insert_variable(
                param.name.clone(),
                param.type_annotation.clone(),
                OwnershipStatus::Borrowed,
                param.lifetime_annotation.clone()
            );
            let return_level = get_universe_level(return_type, &pi_env)?;
            Ok(std::cmp::max(param_level, return_level))
        },
        Type::Sigma(param, snd_type) => {
            // Sigma types: if param.type_annotation : Type_i and snd_type : Type_j,
            // then Sigma type : max(i, j)
            let fst_level = get_universe_level(&param.type_annotation, env)?;
            // Create a new environment with the parameter bound to check snd_type
            let mut sigma_env = env.clone();
            sigma_env.insert_variable(
                param.name.clone(),
                param.type_annotation.clone(),
                OwnershipStatus::Borrowed,
                param.lifetime_annotation.clone()
            );
            let snd_level = get_universe_level(snd_type, &sigma_env)?;
            Ok(std::cmp::max(fst_level, snd_level))
        },
        Type::Linear(inner) => get_universe_level(inner, env),
        Type::Universe(level) => Ok(*level),
        Type::Equality(ty, _, _) => {
            // Equality types live in the same universe as the type they're comparing
            get_universe_level(ty, env)
        },
        Type::Channel(inner) => get_universe_level(inner, env), // Channel types have the same universe as their inner type
        Type::Named(name) => {
            // For named types, we need to look up their definition
            match env.get_type_definition(name) {
                Some(TypeDefinition::NamedType(ty)) => get_universe_level(ty, env),
                Some(TypeDefinition::UniverseType(level)) => Ok(*level),
                Some(_) => Ok(0), // Other type definitions default to Type_0
                None => Err(format!("Unknown type: {}", name)),
            }
        },
        Type::Generic(_) => Ok(0), // Generic types are in Type_0
        Type::Option(inner) => get_universe_level(inner, env),
        Type::Result(ok, _) => get_universe_level(ok, env),
    }
}

// Helper function to validate that a type is well-formed with universe checking
fn validate_type(type_: &Type, env: &TypeEnvironment) -> Result<(), String> {
    match type_ {
        Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Infer => Ok(()),
        Type::Array(inner) => validate_type(inner, env),
        Type::Tuple(types) => {
            for t in types {
                validate_type(t, env)?;
            }
            Ok(())
        },
        Type::Function(params, return_type) => {
            for param in params {
                validate_type(param, env)?;
            }
            validate_type(return_type, env)
        },
        Type::Pi(param, return_type) => {
            // Validate the parameter type
            validate_type(&param.type_annotation, env)?;

            // Check universe consistency: parameter type must be in a lower or equal universe
            // than the Pi type itself will inhabit
            let param_level = get_universe_level(&param.type_annotation, env)?;

            // Create a new environment with the parameter bound
            let mut pi_env = env.clone();
            pi_env.insert_variable(
                param.name.clone(),
                param.type_annotation.clone(),
                OwnershipStatus::Borrowed, // In Pi types, the parameter is borrowed
                param.lifetime_annotation.clone()
            );

            // Validate the return type in the extended environment
            validate_type(return_type, &pi_env)?;

            // The Pi type itself inhabits a universe level that's consistent
            // with its parameter and return types
            Ok(())
        },
        Type::Sigma(param, snd_type) => {
            // Validate the first parameter type
            validate_type(&param.type_annotation, env)?;

            // Create a new environment with the parameter bound
            let mut sigma_env = env.clone();
            sigma_env.insert_variable(
                param.name.clone(),
                param.type_annotation.clone(),
                OwnershipStatus::Borrowed,
                param.lifetime_annotation.clone()
            );

            // Validate the second type in the extended environment
            validate_type(snd_type, &sigma_env)
        },
        Type::Linear(inner) => {
            // Linear types wrap another valid type
            validate_type(inner, env)
        },
        Type::Universe(level) => {
            // Universe types are always valid
            Ok(())
        },
        Type::Equality(ty, lhs, rhs) => {
            // Validate the type being compared
            validate_type(ty, env)?;

            // Check that both expressions have the expected type
            let mut temp_effects = EffectSet::new();
            let lhs_type = check_expression(lhs, &mut env.clone(), &mut temp_effects)?;
            let rhs_type = check_expression(rhs, &mut env.clone(), &mut temp_effects)?;

            if !types_match(ty, &lhs_type) || !types_match(ty, &rhs_type) {
                return Err("Equality constraint expressions must match the given type".to_string());
            }

            Ok(())
        },
        Type::Channel(inner) => {
            // Channel types wrap another valid type
            validate_type(inner, env)
        },
        Type::Named(name) => {
            // Check if this named type exists in the environment
            match env.get_type_definition(name) {
                Some(_) => Ok(()),
                None => Err(format!("Unknown type: {}", name)),
            }
        },
        Type::Generic(name) => {
            // Generic types are valid in type contexts
            Ok(())
        },
        Type::Option(inner) => validate_type(inner, env),
        Type::Result(ok, err) => {
            validate_type(ok, env)?;
            validate_type(err, env)
        },
    }
}

// Function to check pattern matching types
fn bind_pattern_variables(pattern: &Pattern, pattern_type: &Type, env: &mut TypeEnvironment) -> Result<(), String> {
    match pattern {
        Pattern::Identifier(name) => {
            // Bind the variable to the pattern type
            let ownership = if matches!(pattern_type, Type::Linear(_)) {
                OwnershipStatus::Linear
            } else {
                OwnershipStatus::Owned
            };

            env.insert_variable(name.clone(), pattern_type.clone(), ownership, None);
            Ok(())
        },
        Pattern::Tuple(patterns) => {
            if let Type::Tuple(type_elements) = pattern_type {
                if patterns.len() != type_elements.len() {
                    return Err(format!(
                        "Tuple pattern has {} elements, but expected {}",
                        patterns.len(),
                        type_elements.len()
                    ));
                }

                for (pattern, type_elem) in patterns.iter().zip(type_elements.iter()) {
                    bind_pattern_variables(pattern, type_elem, env)?;
                }

                Ok(())
            } else {
                Err(format!("Tuple pattern applied to non-tuple type {:?}", pattern_type))
            }
        },
        Pattern::Wildcard => Ok(()), // Wildcard binds nothing
        Pattern::Literal(_) => Ok(()), // Literal patterns don't bind variables
        Pattern::Struct(struct_name, fields) => {
            // In a real implementation, we'd look up the struct definition
            // For now, we'll just accept struct patterns
            Ok(())
        },
        Pattern::Array(patterns) => {
            if let Type::Array(element_type) = pattern_type {
                // For array patterns, bind each element pattern in the context of the element type
                for pattern in patterns {
                    bind_pattern_variables(pattern, element_type, env)?;
                }
                Ok(())
            } else {
                Err(format!("Array pattern applied to non-array type {:?}", pattern_type))
            }
        },
        Pattern::Or(left, right) => {
            // Both alternatives should bind the same variables
            // For simplicity, we'll just check both sides
            bind_pattern_variables(left, pattern_type, env)?;
            bind_pattern_variables(right, pattern_type, env)?;
            Ok(())
        },
    }
}

fn check_pattern(pattern: &Pattern, expected_type: &Type, env: &mut TypeEnvironment, effects: &mut EffectSet) -> Result<(), String> {
    match pattern {
        Pattern::Identifier(name) => {
            // Determine the appropriate ownership based on the expected type
            // If the expected type is linear, the bound variable should also be linear
            let ownership = if matches!(expected_type, Type::Linear(_)) {
                OwnershipStatus::Linear
            } else {
                OwnershipStatus::Owned
            };

            // Bind the variable to the expected type
            env.insert_variable(name.clone(), expected_type.clone(), ownership, None);
            Ok(())
        },
        Pattern::Literal(literal_expr) => {
            let literal_type = check_expression(literal_expr, env, effects)?;
            if !types_match(expected_type, &literal_type) {
                return Err(format!(
                    "Pattern literal type {:?} doesn't match expected type {:?}",
                    literal_type, expected_type
                ));
            }
            Ok(())
        },
        Pattern::Wildcard => Ok(()), // Wildcard matches anything
        Pattern::Tuple(patterns) => {
            if let Type::Tuple(expected_types) = expected_type {
                if patterns.len() != expected_types.len() {
                    return Err(format!(
                        "Tuple pattern has {} elements, but expected {}",
                        patterns.len(),
                        expected_types.len()
                    ));
                }

                for (pattern, expected_elem_type) in patterns.iter().zip(expected_types.iter()) {
                    check_pattern(pattern, expected_elem_type, env, effects)?;
                }

                Ok(())
            } else {
                Err(format!("Tuple pattern applied to non-tuple type {:?}", expected_type))
            }
        },
        Pattern::Array(patterns) => {
            if let Type::Array(expected_elem_type) = expected_type {
                // For array patterns, each element pattern should match the array element type
                for pattern in patterns {
                    check_pattern(pattern, expected_elem_type, env, effects)?;
                }
                Ok(())
            } else {
                Err(format!("Array pattern applied to non-array type {:?}", expected_type))
            }
        },
        Pattern::Struct(struct_name, fields) => {
            // In a real implementation, we'd look up the struct definition
            // For now, we'll just check that the fields match expected types where possible
            // This would require more complex type lookup

            // For this simplified implementation, we'll just accept struct patterns
            // but we should handle linear types in the fields appropriately
            Ok(())
        },
        Pattern::Or(left, right) => {
            // Both alternatives should match the expected type
            check_pattern(left, expected_type, env, effects)?;
            check_pattern(right, expected_type, env, effects)?;
            Ok(())
        },
        Pattern::Array(_) => {
            // For array patterns, we would check that the expected type is an array
            // and that the element patterns match the array element type
            // For now, we'll just accept array patterns
            Ok(())
        },
    }
}

// Function to normalize a type by evaluating any embedded expressions
fn normalize_type(type_: &Type, env: &TypeEnvironment) -> Result<Type, String> {
    match type_ {
        Type::Int | Type::Float | Type::Bool | Type::String | Type::Unit | Type::Infer => Ok(type_.clone()),
        Type::Array(inner) => {
            let normalized_inner = normalize_type(inner, env)?;
            Ok(Type::Array(Box::new(normalized_inner)))
        },
        Type::Tuple(types) => {
            let mut normalized_types = Vec::new();
            for t in types {
                normalized_types.push(normalize_type(t, env)?);
            }
            Ok(Type::Tuple(normalized_types))
        },
        Type::Function(params, return_type) => {
            let mut normalized_params = Vec::new();
            for param in params {
                normalized_params.push(normalize_type(param, env)?);
            }
            let normalized_return = normalize_type(return_type, env)?;
            Ok(Type::Function(normalized_params, Box::new(normalized_return)))
        },
        Type::Pi(param, return_type) => {
            // Normalize the parameter type
            let normalized_param_type = normalize_type(&param.type_annotation, env)?;

            // Create a new environment with the parameter bound to normalize the return type
            let mut pi_env = env.clone();
            pi_env.insert_variable(
                param.name.clone(),
                normalized_param_type.clone(),
                OwnershipStatus::Borrowed,
                param.lifetime_annotation.clone()
            );

            let normalized_return = normalize_type(return_type, &pi_env)?;

            // Create a new parameter with the normalized type
            let mut new_param = param.clone();
            new_param.type_annotation = normalized_param_type;

            Ok(Type::Pi(new_param, Box::new(normalized_return)))
        },
        Type::Sigma(param, snd_type) => {
            // Normalize the first parameter type
            let normalized_fst_type = normalize_type(&param.type_annotation, env)?;

            // Create a new environment with the parameter bound to normalize the second type
            let mut sigma_env = env.clone();
            sigma_env.insert_variable(
                param.name.clone(),
                normalized_fst_type.clone(),
                OwnershipStatus::Borrowed,
                param.lifetime_annotation.clone()
            );

            let normalized_snd = normalize_type(snd_type, &sigma_env)?;

            // Create a new parameter with the normalized type
            let mut new_param = param.clone();
            new_param.type_annotation = normalized_fst_type;

            Ok(Type::Sigma(new_param, Box::new(normalized_snd)))
        },
        Type::Linear(inner) => {
            let normalized_inner = normalize_type(inner, env)?;
            Ok(Type::Linear(Box::new(normalized_inner)))
        },
        Type::Channel(inner) => {
            let normalized_inner = normalize_type(inner, env)?;
            Ok(Type::Channel(Box::new(normalized_inner)))
        },
        Type::Universe(level) => Ok(Type::Universe(*level)),
        Type::Equality(ty, lhs, rhs) => {
            let normalized_ty = normalize_type(ty, env)?;
            // In a full implementation, we'd normalize the expressions too
            Ok(Type::Equality(
                Box::new(normalized_ty),
                lhs.clone(),
                rhs.clone()
            ))
        },
        Type::Named(name) => Ok(Type::Named(name.clone())),
        Type::Generic(name) => Ok(Type::Generic(name.clone())),
        Type::Option(inner) => {
            let normalized_inner = normalize_type(inner, env)?;
            Ok(Type::Option(Box::new(normalized_inner)))
        },
        Type::Result(ok, err) => {
            let normalized_ok = normalize_type(ok, env)?;
            let normalized_err = normalize_type(err, env)?;
            Ok(Type::Result(Box::new(normalized_ok), Box::new(normalized_err)))
        },
    }
}

// Helper function to check if ownership is compatible for function calls
fn ownership_compatible(expected_ownership: &OwnershipStatus, arg_expr: &Expression) -> bool {
    match arg_expr {
        Expression::Identifier(name) => {
            // In a real implementation, we'd check the ownership of the variable
            // For now, we'll just return true
            true
        },
        _ => true, // For literals and other expressions, ownership is not an issue
    }
}

// Function to check if two types are equal in the context of an environment
fn type_equal(t1: &Type, t2: &Type, env: &TypeEnvironment) -> bool {
    // For now, we'll use structural type matching
    // In a more sophisticated implementation, we'd handle type aliases and context-dependent types
    structural_type_match(t1, t2)
}

// Function to bind pattern types to an environment
fn bind_pattern_types(pattern: &Pattern, pattern_type: &Type, env: &mut TypeEnvironment) -> Result<(), String> {
    match pattern {
        Pattern::Identifier(name) => {
            // Bind the variable to the pattern type
            let ownership = if matches!(pattern_type, Type::Linear(_)) {
                OwnershipStatus::Linear
            } else {
                OwnershipStatus::Owned
            };

            // Use the insert_variable method of TypeEnvironment
            env.insert_variable(name.clone(), pattern_type.clone(), ownership, None);
            Ok(())
        },
        Pattern::Tuple(patterns) => {
            if let Type::Tuple(type_elements) = pattern_type {
                if patterns.len() != type_elements.len() {
                    return Err(format!(
                        "Tuple pattern has {} elements, but expected {}",
                        patterns.len(),
                        type_elements.len()
                    ));
                }

                for (pattern, type_elem) in patterns.iter().zip(type_elements.iter()) {
                    bind_pattern_types(pattern, type_elem, env)?;
                }

                Ok(())
            } else {
                Err(format!("Tuple pattern applied to non-tuple type {:?}", pattern_type))
            }
        },
        Pattern::Wildcard => Ok(()), // Wildcard binds nothing
        Pattern::Literal(_) => Ok(()), // Literal patterns don't bind variables
        Pattern::Struct(struct_name, fields) => {
            // In a real implementation, we'd look up the struct definition
            // For now, we'll just accept struct patterns
            Ok(())
        },
        Pattern::Array(patterns) => {
            if let Type::Array(element_type) = pattern_type {
                // For array patterns, bind each element pattern in the context of the element type
                for pattern in patterns {
                    bind_pattern_types(pattern, element_type, env)?;
                }
                Ok(())
            } else {
                Err(format!("Array pattern applied to non-array type {:?}", pattern_type))
            }
        },
        Pattern::Or(left, right) => {
            // Both alternatives should bind the same variables with the same types
            // For simplicity, we'll just check both sides
            bind_pattern_types(left, pattern_type, env)?;
            bind_pattern_types(right, pattern_type, env)?;
            Ok(())
        },
    }
}

// Function to check conversion (definitional equality) between types
fn conversion_check(t1: &Type, t2: &Type, env: &TypeEnvironment) -> Result<bool, String> {
    // First, normalize both types
    let norm_t1 = normalize_type(t1, env)?;
    let norm_t2 = normalize_type(t2, env)?;

    // Then check if they match structurally
    Ok(structural_type_match(&norm_t1, &norm_t2))
}

// Helper function for structural type matching
fn structural_type_match(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::Int, Type::Int) => true,
        (Type::Float, Type::Float) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::String, Type::String) => true,
        (Type::Unit, Type::Unit) => true,
        (Type::Array(a), Type::Array(b)) => structural_type_match(a, b),
        (Type::Tuple(types1), Type::Tuple(types2)) => {
            if types1.len() != types2.len() {
                false
            } else {
                types1.iter().zip(types2).all(|(t1, t2)| structural_type_match(t1, t2))
            }
        },
        (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
            if params1.len() != params2.len() {
                false
            } else {
                let params_match = params1.iter().zip(params2).all(|(t1, t2)| structural_type_match(t1, t2));
                params_match && structural_type_match(ret1, ret2)
            }
        },
        (Type::Pi(param1, ret1), Type::Pi(param2, ret2)) => {
            // For Pi types to match, the parameter types must match
            // and the return types must match in the context where the parameter is bound
            structural_type_match(&param1.type_annotation, &param2.type_annotation) &&
            structural_type_match(ret1, ret2)
        },
        (Type::Sigma(param1, snd1), Type::Sigma(param2, snd2)) => {
            // For Sigma types to match, the first parameter types must match
            // and the second types must match in the context where the first is bound
            structural_type_match(&param1.type_annotation, &param2.type_annotation) &&
            structural_type_match(snd1, snd2)
        },
        (Type::Linear(a), Type::Linear(b)) => structural_type_match(a, b),
        (Type::Universe(l1), Type::Universe(l2)) => l1 == l2,
        (Type::Equality(ty1, lhs1, rhs1), Type::Equality(ty2, lhs2, rhs2)) => {
            structural_type_match(ty1, ty2) &&
            // We would need to check expression equality here, simplified for now
            true
        },
        (Type::Named(n1), Type::Named(n2)) => n1 == n2,
        (Type::Generic(_), _) | (_, Type::Generic(_)) => true, // Simplified for generics
        (Type::Option(opt1), Type::Option(opt2)) => structural_type_match(opt1, opt2),
        (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
            structural_type_match(ok1, ok2) && structural_type_match(err1, err2)
        },
        (Type::Channel(a), Type::Channel(b)) => structural_type_match(a, b), // Added for channel types
        _ => false,
    }
}

fn types_match(t1: &Type, t2: &Type) -> bool {
    // For backward compatibility, use the structural match
    structural_type_match(t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_type_check_simple_program() {
        let input = r#"
        fn main() {
            let x: Int = 42
            let y: Int = x + 1
            print("Result: " + y.to_string())
        }
        "#;

        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();

        // This should not panic
        let result = check_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_error() {
        let input = r#"
        fn main() {
            let x: Int = "hello"  // Type error: string assigned to int
        }
        "#;

        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();

        let result = check_program(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_check_function_call() {
        let input = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b
        }

        fn main() {
            let result = add(5, 10)
            print(result.to_string())
        }
        "#;

        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();

        let result = check_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_function_call_wrong_args() {
        let input = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b
        }

        fn main() {
            let result = add(5)  // Wrong number of arguments
        }
        "#;

        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();

        let result = check_program(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_dependent_types_universe_levels() {
        let env = TypeEnvironment::new();

        // Test that basic types are in universe 0
        let int_level = get_universe_level(&Type::Int, &env).unwrap();
        assert_eq!(int_level, 0);

        // Test that universe types have their specified level
        let universe_level = get_universe_level(&Type::Universe(2), &env).unwrap();
        assert_eq!(universe_level, 2);
    }

    #[test]
    fn test_type_normalization() {
        let env = TypeEnvironment::new();

        // Test normalization of basic types
        let normalized_int = normalize_type(&Type::Int, &env).unwrap();
        assert_eq!(normalized_int, Type::Int);

        // Test normalization of complex types
        let array_type = Type::Array(Box::new(Type::Int));
        let normalized_array = normalize_type(&array_type, &env).unwrap();
        assert_eq!(normalized_array, array_type);
    }

    #[test]
    fn test_structural_type_matching() {
        // Test that identical types match
        assert!(structural_type_match(&Type::Int, &Type::Int));
        assert!(!structural_type_match(&Type::Int, &Type::Bool));

        // Test that complex types match appropriately
        let array_int = Type::Array(Box::new(Type::Int));
        let array_bool = Type::Array(Box::new(Type::Bool));
        assert!(structural_type_match(&array_int, &array_int));
        assert!(!structural_type_match(&array_int, &array_bool));
    }

    #[test]
    fn test_linear_type_creation() {
        let env = TypeEnvironment::new();

        // Test creating a linear type
        let linear_int = Type::Linear(Box::new(Type::Int));
        let normalized = normalize_type(&linear_int, &env).unwrap();
        assert!(matches!(normalized, Type::Linear(_)));

        // Test that linear types have the same universe level as their inner type
        let level = get_universe_level(&linear_int, &env).unwrap();
        assert_eq!(level, 0); // Should be same as Int
    }

    #[test]
    fn test_linear_variable_tracking() {
        let mut env = TypeEnvironment::new();

        // Insert a linear variable
        env.insert_variable(
            "x".to_string(),
            Type::Int,
            OwnershipStatus::Linear,
            None
        );

        // Initially, the variable should not be marked as used
        assert!(!env.used_linear_vars.contains("x"));

        // Mark it as used
        env.mark_linear_var_used("x").unwrap();

        // Now it should be marked as used
        assert!(env.used_linear_vars.contains("x"));

        // Trying to use it again should fail
        let result = env.mark_linear_var_used("x");
        assert!(result.is_err());
    }

    #[test]
    fn test_guard_pattern_type_checking() {
        // This test would require a full parser and type checker setup
        // For now, we verify that the AST structure supports guards
        let expr = Expression::Integer(42);
        let pattern = Pattern::Identifier("x".to_string());
        let guard = Some(Box::new(Expression::BinaryOp(
            Box::new(Expression::Identifier("x".to_string())),
            BinaryOp::Gt,
            Box::new(Expression::Integer(10))
        )));
        let body = vec![Statement::Expression(Expression::Identifier("x".to_string()))];

        // Create a match expression with a guard
        let match_expr = Expression::Match(
            Box::new(expr),
            vec![(pattern, guard, body)]
        );

        // The structure should be valid
        match match_expr {
            Expression::Match(_, arms) => {
                assert_eq!(arms.len(), 1);
                let (pat, guard_opt, _) = &arms[0];
                assert!(matches!(pat, Pattern::Identifier(_)));
                assert!(guard_opt.is_some());
            },
            _ => panic!("Expected Match expression"),
        }
    }
}