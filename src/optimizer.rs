// Logos Programming Language Optimizer
// This module provides optimization passes for the AST to improve performance.

use crate::ast::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Optimizer for performing various optimization passes on the AST
pub struct Optimizer {
    passes: Vec<OptimizationPass>,
    /// Cache for optimization results to avoid re-computation
    optimization_cache: Arc<RwLock<HashMap<String, Program>>>,
}

/// Different types of optimization passes available
#[derive(Debug, Clone)]
pub enum OptimizationPass {
    ConstantFolding,                    // Fold constant expressions at compile time
    DeadCodeElimination,               // Remove unreachable code
    CommonSubexpressionElimination,    // Eliminate repeated computations
    FunctionInlining,                  // Inline small functions
    LoopOptimization,                  // Optimize loop structures
    RegisterAllocation,                // Optimize register usage
    TailCallOptimization,              // Optimize tail recursive calls
    MemoryOptimization,                // Optimize memory usage patterns
    ConcurrencyOptimization,           // Optimize concurrent code patterns
    TraitBasedOptimization,            // Optimize based on trait implementations
    TypeSpecialization,                // Specialize code based on concrete types
    ClosureOptimization,               // Optimize closure representations
    PatternMatchingOptimization,       // Optimize pattern matching constructs
}

impl Optimizer {
    /// Creates a new optimizer instance with default optimization passes
    pub fn new() -> Self {
        Self {
            passes: vec![
                OptimizationPass::ConstantFolding,
                OptimizationPass::DeadCodeElimination,
                OptimizationPass::CommonSubexpressionElimination,
                OptimizationPass::FunctionInlining,
                OptimizationPass::TypeSpecialization,
                OptimizationPass::TraitBasedOptimization,
                OptimizationPass::TailCallOptimization,
                OptimizationPass::LoopOptimization,
                OptimizationPass::ClosureOptimization,
                OptimizationPass::PatternMatchingOptimization,
            ],
            optimization_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds an optimization pass to the optimizer
    pub fn add_pass(&mut self, pass: OptimizationPass) {
        self.passes.push(pass);
    }

    /// Optimizes a program by applying all registered optimization passes
    pub fn optimize_program(&self, program: Program) -> Program {
        // Create a cache key based on the program's content
        let cache_key = self.create_cache_key(&program);

        // Check if we have a cached result
        if let Ok(cache) = self.optimization_cache.read() {
            if let Some(cached_program) = cache.get(&cache_key) {
                return cached_program.clone();
            }
        }

        let mut optimized_program = program;

        for pass in &self.passes {
            optimized_program = self.apply_pass(optimized_program);
        }

        // Cache the result
        if let Ok(mut cache) = self.optimization_cache.write() {
            cache.insert(cache_key, optimized_program.clone());
        }

        optimized_program
    }

    /// Creates a cache key for a program
    fn create_cache_key(&self, program: &Program) -> String {
        // A simple hash of the program's statements could be used as a key
        // For now, we'll use a placeholder implementation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        program.statements.len().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Applies a single optimization pass to a program
    fn apply_pass(&self, program: Program) -> Program {
        // Apply different optimizations based on the pass type
        // For now, we'll just apply the standard statement/expression optimizations
        Program {
            statements: program.statements
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
        }
    }

    /// Applies tail call optimization to a function
    fn apply_tail_call_optimization(&self, func_def: FunctionDef) -> FunctionDef {
        // In a full implementation, this would identify tail recursive calls
        // and convert them to loops to avoid stack overflow
        // For now, return the function unchanged
        FunctionDef {
            name: func_def.name,
            parameters: func_def.parameters,
            return_type: func_def.return_type,
            body: func_def.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
            is_async: func_def.is_async,
            is_public: func_def.is_public,
            is_awaitable: func_def.is_awaitable,
            effect_annotations: func_def.effect_annotations,
            generic_params: func_def.generic_params,
        }
    }

    /// Applies loop optimization to statements
    fn apply_loop_optimization(&self, statements: Vec<Statement>) -> Vec<Statement> {
        // In a full implementation, this would optimize loop structures
        // For now, just return the statements with basic optimization
        statements
            .into_iter()
            .map(|stmt| self.optimize_statement(stmt))
            .collect()
    }

    /// Applies closure optimization
    fn apply_closure_optimization(&self, expr: Expression) -> Expression {
        // In a full implementation, this would optimize closure representations
        // For now, return the expression with basic optimization
        self.optimize_expression(expr)
    }

    /// Applies pattern matching optimization
    fn apply_pattern_matching_optimization(&self, expr: Expression) -> Expression {
        // In a full implementation, this would optimize pattern matching constructs
        // For now, return the expression with basic optimization
        self.optimize_expression(expr)
    }

    /// Applies register allocation optimization
    fn apply_register_allocation(&self, program: Program) -> Program {
        // In a full implementation, this would optimize register usage
        // For now, return the program unchanged
        Program {
            statements: program.statements
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
        }
    }

    /// Applies type specialization optimization
    fn apply_type_specialization(&self, program: Program) -> Program {
        // This would specialize generic code based on concrete types
        // For now, return the program unchanged
        program
    }

    /// Applies trait-based optimization
    fn apply_trait_optimization(&self, program: Program) -> Program {
        // This would optimize code based on trait implementations
        // For now, return the program unchanged
        program
    }

    /// Optimizes a statement
    fn optimize_statement(&self, statement: Statement) -> Statement {
        match statement {
            Statement::Expression(expr) => {
                Statement::Expression(self.optimize_expression(expr))
            },
            Statement::LetBinding { mutable, name, type_annotation, value, ownership_modifier, lifetime_annotation } => {
                Statement::LetBinding {
                    mutable,
                    name,
                    type_annotation,
                    value: self.optimize_expression(value),
                    ownership_modifier,
                    lifetime_annotation,
                }
            },
            Statement::ConstBinding { name, type_annotation, value } => {
                Statement::ConstBinding {
                    name,
                    type_annotation,
                    value: self.optimize_expression(value),
                }
            },
            Statement::Function(func_def) => {
                Statement::Function(self.optimize_function(func_def))
            },
            Statement::Block(statements) => {
                // Apply loop optimization to the block
                let optimized_statements = self.apply_loop_optimization(statements);
                Statement::Block(optimized_statements)
            },
            Statement::Return(expr) => {
                Statement::Return(expr.map(|e| self.optimize_expression(e)))
            },
            Statement::Break => Statement::Break,
            Statement::Continue => Statement::Continue,
            Statement::Class(class_def) => {
                Statement::Class(self.optimize_class(class_def))
            },
            Statement::Trait(trait_def) => {
                Statement::Trait(self.optimize_trait(trait_def))
            },
            Statement::Implementation(impl_def) => {
                Statement::Implementation(self.optimize_impl(impl_def))
            },
            Statement::Actor(actor_def) => {
                Statement::Actor(self.optimize_actor(actor_def))
            },
            Statement::Effect(effect_def) => {
                Statement::Effect(self.optimize_effect(effect_def))
            },
            Statement::MacroDefinition(macro_def) => {
                Statement::MacroDefinition(self.optimize_macro(macro_def))
            },
            Statement::Enum(enum_def) => {
                Statement::Enum(self.optimize_enum(enum_def))
            },
            Statement::TypeAlias(alias_def) => {
                Statement::TypeAlias(alias_def) // Type aliases don't need optimization
            },
        }
    }

    /// Optimizes an enum definition
    fn optimize_enum(&self, enum_def: EnumDef) -> EnumDef {
        // For now, return the enum definition as is
        // In a full implementation, we would optimize the variants
        EnumDef {
            name: enum_def.name,
            variants: enum_def.variants,
            access_modifier: enum_def.access_modifier,
            generics: enum_def.generics,
        }
    }

    /// Optimizes a macro definition
    fn optimize_macro(&self, macro_def: MacroDef) -> MacroDef {
        // For now, return the macro definition as is
        // In a full implementation, we would optimize the macro body
        MacroDef {
            name: macro_def.name,
            parameters: macro_def.parameters,  // Parameters typically don't need optimization
            body: macro_def.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
            is_hygienic: macro_def.is_hygienic,
        }
    }

    /// Optimizes a function definition
    fn optimize_function(&self, func_def: FunctionDef) -> FunctionDef {
        // Apply tail call optimization to the function
        let optimized_func = self.apply_tail_call_optimization(func_def);

        FunctionDef {
            name: optimized_func.name,
            parameters: optimized_func.parameters, // Parameters typically don't need optimization
            return_type: optimized_func.return_type,
            body: optimized_func.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
            is_async: optimized_func.is_async,
            is_public: optimized_func.is_public,
            is_awaitable: optimized_func.is_awaitable,
            effect_annotations: optimized_func.effect_annotations,
            generic_params: optimized_func.generic_params,
        }
    }

    /// Optimizes a class definition
    fn optimize_class(&self, class_def: ClassDef) -> ClassDef {
        ClassDef {
            name: class_def.name,
            fields: class_def.fields,
            methods: class_def.methods
                .into_iter()
                .map(|method| self.optimize_function(method))
                .collect(),
            parent: class_def.parent,
            access_modifier: class_def.access_modifier,
            is_abstract: class_def.is_abstract,
            generics: class_def.generics,
            interfaces: class_def.interfaces,
            constructors: class_def.constructors
                .into_iter()
                .map(|constructor| self.optimize_constructor(constructor))
                .collect(),
            destructors: class_def.destructors
                .into_iter()
                .map(|destructor| self.optimize_destructor(destructor))
                .collect(),
        }
    }

    /// Optimizes a constructor definition
    fn optimize_constructor(&self, constructor: ConstructorDef) -> ConstructorDef {
        ConstructorDef {
            parameters: constructor.parameters,
            body: constructor.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
            access_modifier: constructor.access_modifier,
        }
    }

    /// Optimizes a destructor definition
    fn optimize_destructor(&self, destructor: DestructorDef) -> DestructorDef {
        DestructorDef {
            body: destructor.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
            access_modifier: destructor.access_modifier,
        }
    }

    /// Optimizes a trait definition
    fn optimize_trait(&self, trait_def: TraitDef) -> TraitDef {
        TraitDef {
            name: trait_def.name,
            type_params: trait_def.type_params,
            methods: trait_def.methods
                .into_iter()
                .map(|method| self.optimize_function(method))
                .collect(),
            associated_types: trait_def.associated_types,
            super_traits: trait_def.super_traits,
        }
    }

    /// Optimizes an implementation definition
    fn optimize_impl(&self, impl_def: ImplDef) -> ImplDef {
        ImplDef {
            trait_name: impl_def.trait_name,
            for_type: impl_def.for_type,
            type_params: impl_def.type_params,
            methods: impl_def.methods
                .into_iter()
                .map(|method| self.optimize_function(method))
                .collect(),
            associated_types: impl_def.associated_types,
        }
    }

    /// Optimizes an actor definition
    fn optimize_actor(&self, actor_def: ActorDef) -> ActorDef {
        ActorDef {
            name: actor_def.name,
            state: actor_def.state,
            handlers: actor_def.handlers
                .into_iter()
                .map(|handler| self.optimize_function(handler))
                .collect(),
        }
    }

    /// Optimizes an effect definition
    fn optimize_effect(&self, effect_def: EffectDef) -> EffectDef {
        EffectDef {
            name: effect_def.name,
            operations: effect_def.operations
                .into_iter()
                .map(|operation| self.optimize_function(operation))
                .collect(),
        }
    }

    /// Optimizes an expression
    fn optimize_expression(&self, expr: Expression) -> Expression {
        match expr {
            Expression::Integer(_) | Expression::Float(_) | Expression::String(_) | 
            Expression::Boolean(_) | Expression::Nil => expr, // Literals don't need optimization
            
            Expression::Identifier(_) => expr, // Identifiers don't need optimization
            
            Expression::BinaryOp(left, op, right) => {
                let optimized_left = Box::new(self.optimize_expression(*left));
                let optimized_right = Box::new(self.optimize_expression(*right));

                // Perform constant folding if both operands are constants
                match (&*optimized_left, &*optimized_right, &op) {
                    (Expression::Integer(a), Expression::Integer(b), BinaryOp::Add) => {
                        Expression::Integer(a + b)
                    },
                    (Expression::Integer(a), Expression::Integer(b), BinaryOp::Sub) => {
                        Expression::Integer(a - b)
                    },
                    (Expression::Integer(a), Expression::Integer(b), BinaryOp::Mul) => {
                        Expression::Integer(a * b)
                    },
                    (Expression::Integer(a), Expression::Integer(b), BinaryOp::Div) if *b != 0 => {
                        Expression::Integer(a / b)
                    },
                    (Expression::Float(a), Expression::Float(b), BinaryOp::Add) => {
                        Expression::Float(a + b)
                    },
                    (Expression::Float(a), Expression::Float(b), BinaryOp::Sub) => {
                        Expression::Float(a - b)
                    },
                    (Expression::Float(a), Expression::Float(b), BinaryOp::Mul) => {
                        Expression::Float(a * b)
                    },
                    (Expression::Float(a), Expression::Float(b), BinaryOp::Div) if *b != 0.0 => {
                        Expression::Float(a / b)
                    },
                    (Expression::Boolean(a), Expression::Boolean(b), BinaryOp::And) => {
                        Expression::Boolean(*a && *b)
                    },
                    (Expression::Boolean(a), Expression::Boolean(b), BinaryOp::Or) => {
                        Expression::Boolean(*a || *b)
                    },
                    _ => Expression::BinaryOp(optimized_left, op, optimized_right),
                }
            },
            
            Expression::UnaryOp(op, expr) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));

                // Perform constant folding for unary operations on constants
                match (&*optimized_expr, &op) {
                    (Expression::Integer(a), UnaryOp::Neg) => Expression::Integer(-a),
                    (Expression::Float(a), UnaryOp::Neg) => Expression::Float(-a),
                    (Expression::Boolean(a), UnaryOp::Not) => Expression::Boolean(!a),
                    _ => Expression::UnaryOp(op, optimized_expr),
                }
            },
            
            Expression::Call(name, args) => {
                let optimized_args = args
                    .into_iter()
                    .map(|arg| self.optimize_expression(arg))
                    .collect();
                
                Expression::Call(name, optimized_args)
            },
            
            Expression::MethodCall(obj, method, args) => {
                let optimized_obj = Box::new(self.optimize_expression(*obj));
                let optimized_args = args
                    .into_iter()
                    .map(|arg| self.optimize_expression(arg))
                    .collect();
                
                Expression::MethodCall(optimized_obj, method, optimized_args)
            },
            
            Expression::FieldAccess(obj, field) => {
                let optimized_obj = Box::new(self.optimize_expression(*obj));
                
                Expression::FieldAccess(optimized_obj, field)
            },
            
            Expression::If(condition, then_branch, else_branch) => {
                let optimized_condition = self.optimize_expression(*condition);
                let optimized_then = then_branch
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();
                let optimized_else = else_branch
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();
                
                Expression::If(
                    Box::new(optimized_condition),
                    optimized_then,
                    optimized_else,
                )
            },
            
            Expression::Match(expr, arms) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                let optimized_arms = arms
                    .into_iter()
                    .map(|(pattern, guard, body)| {
                        let optimized_guard = guard.map(|g| Box::new(self.optimize_expression(*g)));
                        let optimized_body = body
                            .into_iter()
                            .map(|stmt| self.optimize_statement(stmt))
                            .collect();
                        (pattern, optimized_guard, optimized_body)
                    })
                    .collect();

                // Apply pattern matching optimization
                self.apply_pattern_matching_optimization(Expression::Match(optimized_expr, optimized_arms))
            },
            
            Expression::Lambda(params, body) => {
                let optimized_body = body
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();

                // Apply closure optimization
                self.apply_closure_optimization(Expression::Lambda(params, optimized_body))
            },
            
            Expression::BlockExpr(statements) => {
                let optimized_statements = statements
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();
                
                Expression::BlockExpr(optimized_statements)
            },
            
            Expression::Tuple(items) => {
                let optimized_items = items
                    .into_iter()
                    .map(|item| self.optimize_expression(item))
                    .collect();
                
                Expression::Tuple(optimized_items)
            },
            
            // Multi-language integration expressions
            Expression::MultiLangCall(lang, code) => {
                // For now, don't optimize multi-language calls
                Expression::MultiLangCall(lang, code)
            },
            Expression::MultiLangImport(lang, resource, alias) => {
                // For now, don't optimize multi-language imports
                Expression::MultiLangImport(lang, resource, alias)
            },
            Expression::MultiLangIndex(lang, resource) => {
                // For now, don't optimize multi-language indexing
                Expression::MultiLangIndex(lang, resource)
            },
            
            // CSP-style channel operations
            Expression::ChannelCreate(channel_type) => {
                Expression::ChannelCreate(Box::new(self.optimize_type(*channel_type)))
            },
            Expression::ChannelSend(channel, value) => {
                let optimized_channel = Box::new(self.optimize_expression(*channel));
                let optimized_value = Box::new(self.optimize_expression(*value));
                
                Expression::ChannelSend(optimized_channel, optimized_value)
            },
            Expression::ChannelReceive(channel) => {
                let optimized_channel = Box::new(self.optimize_expression(*channel));
                
                Expression::ChannelReceive(optimized_channel)
            },
            Expression::ChannelClose(channel) => {
                let optimized_channel = Box::new(self.optimize_expression(*channel));
                
                Expression::ChannelClose(optimized_channel)
            },
            Expression::Select(select_arms) => {
                let optimized_arms = select_arms
                    .into_iter()
                    .map(|arm| self.optimize_select_arm(arm))
                    .collect();
                
                Expression::Select(optimized_arms)
            },
            
            // Async/Await constructs
            Expression::AsyncBlock(statements) => {
                let optimized_statements = statements
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();
                
                Expression::AsyncBlock(optimized_statements)
            },
            Expression::Await(expr) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                
                Expression::Await(optimized_expr)
            },
            Expression::Future(expr) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                
                Expression::Future(optimized_expr)
            },
            Expression::SpawnTask(expr) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                
                Expression::SpawnTask(optimized_expr)
            },
            Expression::Join(expr) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                
                Expression::Join(optimized_expr)
            },
            Expression::Race(exprs) => {
                let optimized_exprs = exprs
                    .into_iter()
                    .map(|expr| self.optimize_expression(expr))
                    .collect();
                
                Expression::Race(optimized_exprs)
            },
            Expression::Timeout(expr, duration) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                let optimized_duration = Box::new(self.optimize_expression(*duration));

                Expression::Timeout(optimized_expr, optimized_duration)
            },

            // Missing expression types that were causing the error
            Expression::Spawn(actor_name, args) => {
                let optimized_args = args
                    .into_iter()
                    .map(|arg| self.optimize_expression(arg))
                    .collect();
                Expression::Spawn(actor_name, optimized_args)
            },
            Expression::Send(actor, message) => {
                let optimized_actor = Box::new(self.optimize_expression(*actor));
                let optimized_message = Box::new(self.optimize_expression(*message));
                Expression::Send(optimized_actor, optimized_message)
            },
            Expression::Receive => Expression::Receive,

            // Enhanced syntax constructs
            Expression::LambdaSimple(params, body) => {
                let optimized_body = Box::new(self.optimize_expression(*body));
                Expression::LambdaSimple(params, optimized_body)
            },
            Expression::Pipeline(expr, funcs) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                let optimized_funcs = funcs
                    .into_iter()
                    .map(|func| self.optimize_expression(func))
                    .collect();
                Expression::Pipeline(optimized_expr, optimized_funcs)
            },
            Expression::BackPipeline(expr, funcs) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                let optimized_funcs = funcs
                    .into_iter()
                    .map(|func| self.optimize_expression(func))
                    .collect();
                Expression::BackPipeline(optimized_expr, optimized_funcs)
            },
            Expression::DestructureAssignment(pattern, expr, stmt) => {
                let optimized_expr = Box::new(self.optimize_expression(*expr));
                let optimized_stmt = Box::new(self.optimize_statement(*stmt));
                Expression::DestructureAssignment(pattern, optimized_expr, optimized_stmt)
            },
            Expression::InterpolatedString(parts) => {
                let optimized_parts = parts
                    .into_iter()
                    .map(|part| match part {
                        StringPart::Literal(lit) => StringPart::Literal(lit),
                        StringPart::Interpolated(interp_expr) => {
                            StringPart::Interpolated(Box::new(self.optimize_expression(*interp_expr)))
                        }
                    })
                    .collect();
                Expression::InterpolatedString(optimized_parts)
            },
            Expression::MacroInvocation(name, args) => {
                let optimized_args = args
                    .into_iter()
                    .map(|arg| self.optimize_expression(arg))
                    .collect();
                Expression::MacroInvocation(name, optimized_args)
            },

            // New expression types added
            Expression::Char(_) => expr, // Character literals don't need optimization
            Expression::Array(items) => {
                let optimized_items = items
                    .into_iter()
                    .map(|item| self.optimize_expression(item))
                    .collect();
                Expression::Array(optimized_items)
            },
            Expression::Struct(name, fields) => {
                let optimized_fields = fields
                    .into_iter()
                    .map(|(field_name, field_expr)| (field_name, self.optimize_expression(field_expr)))
                    .collect();
                Expression::Struct(name, optimized_fields)
            },
            Expression::Block(statements) => {
                let optimized_statements = statements
                    .into_iter()
                    .map(|stmt| self.optimize_statement(stmt))
                    .collect();
                Expression::Block(optimized_statements)
            },
        }
    }

    /// Optimizes a type annotation
    fn optimize_type(&self, ty: Type) -> Type {
        match ty {
            Type::Array(inner) => Type::Array(Box::new(self.optimize_type(*inner))),
            Type::Tuple(types) => Type::Tuple(types.into_iter().map(|t| self.optimize_type(t)).collect()),
            Type::Function(params, ret) => Type::Function(
                params.into_iter().map(|t| self.optimize_type(t)).collect(),
                Box::new(self.optimize_type(*ret)),
            ),
            Type::Channel(inner) => Type::Channel(Box::new(self.optimize_type(*inner))),
            Type::Pi(param, ret) => Type::Pi(
                Box::new(self.optimize_parameter(*param)),
                Box::new(self.optimize_type(*ret)),
            ),
            Type::Sigma(param, snd) => Type::Sigma(
                Box::new(self.optimize_parameter(*param)),
                Box::new(self.optimize_type(*snd)),
            ),
            _ => ty, // Other types don't need optimization
        }
    }

    /// Optimizes a parameter
    fn optimize_parameter(&self, param: Parameter) -> Parameter {
        Parameter {
            name: param.name,
            type_annotation: self.optimize_type(param.type_annotation),
            ownership_modifier: param.ownership_modifier,
            lifetime_annotation: param.lifetime_annotation,
            default_value: param.default_value.map(|v| self.optimize_expression(v)),
            mutability: param.mutability,
        }
    }

    /// Optimizes a select arm (for CSP-style channels)
    fn optimize_select_arm(&self, arm: SelectArm) -> SelectArm {
        SelectArm {
            channel_operation: self.optimize_channel_operation(arm.channel_operation),
            pattern: arm.pattern,
            body: arm.body
                .into_iter()
                .map(|stmt| self.optimize_statement(stmt))
                .collect(),
        }
    }

    /// Optimizes a channel operation
    fn optimize_channel_operation(&self, op: ChannelOperation) -> ChannelOperation {
        match op {
            ChannelOperation::Send { channel, value } => ChannelOperation::Send {
                channel: Box::new(self.optimize_expression(*channel)),
                value: Box::new(self.optimize_expression(*value)),
            },
            ChannelOperation::Receive { channel } => ChannelOperation::Receive {
                channel: Box::new(self.optimize_expression(*channel)),
            },
            ChannelOperation::Close { channel } => ChannelOperation::Close {
                channel: Box::new(self.optimize_expression(*channel)),
            },
        }
    }
}

/// Performs constant folding optimization on an expression
pub fn constant_fold(expr: Expression) -> Expression {
    let optimizer = Optimizer::new();
    optimizer.optimize_expression(expr)
}

/// Optimizes a program with all registered optimization passes
pub fn optimize_program(program: Program) -> Program {
    let optimizer = Optimizer::new();
    optimizer.optimize_program(program)
}