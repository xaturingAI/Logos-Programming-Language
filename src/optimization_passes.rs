//! Optimization Passes for the Logos Programming Language
//! This module provides various optimization techniques to improve code performance

use crate::ast::*;
use std::collections::HashMap;

/// Optimization level settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptLevel {
    /// No optimizations
    None,
    /// Basic optimizations
    Basic,
    /// Standard optimizations
    Standard,
    /// Aggressive optimizations
    Aggressive,
}

/// Compiler optimization passes
pub struct Optimizer {
    /// Current optimization level
    pub opt_level: OptLevel,
    /// Enabled optimization passes
    pub passes: Vec<Pass>,
    /// Statistics about optimization effectiveness
    pub stats: OptimizationStats,
}

/// Individual optimization pass
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pass {
    /// Constant folding: evaluate constant expressions at compile time
    ConstantFolding,
    /// Dead code elimination: remove unused code
    DeadCodeElimination,
    /// Common subexpression elimination: avoid recomputing identical expressions
    CommonSubexpressionElimination,
    /// Loop invariant code motion: move invariant computations out of loops
    LoopInvariantCodeMotion,
    /// Function inlining: replace function calls with function bodies
    FunctionInlining,
    /// Tail call optimization: convert tail recursive calls to loops
    TailCallOptimization,
    /// Strength reduction: replace expensive operations with cheaper ones
    StrengthReduction,
    /// Copy propagation: propagate copies of variables
    CopyPropagation,
    /// Register allocation: assign variables to registers
    RegisterAllocation,
}

/// Statistics about optimization effectiveness
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of instructions before optimization
    pub original_instructions: usize,
    /// Number of instructions after optimization
    pub optimized_instructions: usize,
    /// Number of constants folded
    pub constants_folded: usize,
    /// Number of dead code eliminated
    pub dead_code_eliminated: usize,
    /// Number of functions inlined
    pub functions_inlined: usize,
    /// Number of tail calls optimized
    pub tail_calls_optimized: usize,
}

impl Optimizer {
    /// Creates a new optimizer with the specified optimization level
    pub fn new(opt_level: OptLevel) -> Self {
        let passes = match opt_level {
            OptLevel::None => Vec::new(),
            OptLevel::Basic => vec![
                Pass::ConstantFolding,
                Pass::DeadCodeElimination,
            ],
            OptLevel::Standard => vec![
                Pass::ConstantFolding,
                Pass::DeadCodeElimination,
                Pass::CommonSubexpressionElimination,
                Pass::CopyPropagation,
            ],
            OptLevel::Aggressive => vec![
                Pass::ConstantFolding,
                Pass::DeadCodeElimination,
                Pass::CommonSubexpressionElimination,
                Pass::LoopInvariantCodeMotion,
                Pass::FunctionInlining,
                Pass::TailCallOptimization,
                Pass::StrengthReduction,
                Pass::CopyPropagation,
                Pass::RegisterAllocation,
            ],
        };

        Self {
            opt_level,
            passes,
            stats: OptimizationStats {
                original_instructions: 0,
                optimized_instructions: 0,
                constants_folded: 0,
                dead_code_eliminated: 0,
                functions_inlined: 0,
                tail_calls_optimized: 0,
            },
        }
    }

    /// Applies optimizations to a program
    pub fn optimize_program(&mut self, program: Program) -> Program {
        let mut current_program = program;
        
        // Update statistics before optimization
        self.stats.original_instructions = self.count_instructions(&current_program);
        
        // Apply each optimization pass
        let passes_clone = self.passes.clone();
        for pass in passes_clone {
            current_program = self.apply_pass(current_program, pass);
        }
        
        // Update statistics after optimization
        self.stats.optimized_instructions = self.count_instructions(&current_program);
        
        current_program
    }

    /// Applies a single optimization pass to a program
    fn apply_pass(&mut self, program: Program, pass: Pass) -> Program {
        match pass {
            Pass::ConstantFolding => self.fold_constants(program),
            Pass::DeadCodeElimination => self.eliminate_dead_code(program),
            Pass::CommonSubexpressionElimination => self.eliminate_common_subexpressions(program),
            Pass::LoopInvariantCodeMotion => self.move_loop_invariants(program),
            Pass::FunctionInlining => self.inline_functions(program),
            Pass::TailCallOptimization => self.optimize_tail_calls(program),
            Pass::StrengthReduction => self.reduce_strength(program),
            Pass::CopyPropagation => self.propagate_copies(program),
            Pass::RegisterAllocation => self.allocate_registers(program),
        }
    }

    /// Counts the number of instructions/statements in a program
    fn count_instructions(&self, program: &Program) -> usize {
        let mut count = 0;
        for stmt in &program.statements {
            count += self.count_statement_instructions(stmt);
        }
        count
    }

    /// Counts instructions in a statement
    fn count_statement_instructions(&self, stmt: &Statement) -> usize {
        match stmt {
            Statement::Expression(expr) => self.count_expression_instructions(expr),
            Statement::LetBinding { value, .. } => self.count_expression_instructions(value),
            Statement::Function(func_def) => {
                let mut count = 0;
                for stmt in &func_def.body {
                    count += self.count_statement_instructions(stmt);
                }
                count
            },
            Statement::Block(statements) => {
                let mut count = 0;
                for stmt in statements {
                    count += self.count_statement_instructions(stmt);
                }
                count
            },
            // Count other statement types as needed
            _ => 1, // Default to 1 for unhandled statement types
        }
    }

    /// Counts instructions in an expression
    fn count_expression_instructions(&self, expr: &Expression) -> usize {
        match expr {
            Expression::BinaryOp(left, _, right) => {
                self.count_expression_instructions(left) +
                self.count_expression_instructions(right) +
                1 // The operation itself
            },
            Expression::Call(_, args) => {
                let mut count = 1; // The call itself
                for arg in args {
                    count += self.count_expression_instructions(arg);
                }
                count
            },
            // Count other expression types as needed
            _ => 1, // Default to 1 for unhandled expression types
        }
    }

    /// Applies constant folding optimization
    fn fold_constants(&mut self, program: Program) -> Program {
        let mut folded_constants = 0;

        let statements: Vec<Statement> = program.statements
            .into_iter()
            .map(|stmt| self.fold_constants_in_statement(stmt, &mut folded_constants))
            .collect();

        // Update statistics
        self.stats.constants_folded += folded_constants;

        Program { statements }
    }

    /// Applies constant folding to a statement
    fn fold_constants_in_statement(&mut self, stmt: Statement, count: &mut usize) -> Statement {
        match stmt {
            Statement::Expression(expr) => {
                let folded_expr = self.fold_constants_in_expression(expr, count);
                Statement::Expression(folded_expr)
            },
            Statement::LetBinding { mutable, name, type_annotation, value, ownership_modifier, lifetime_annotation } => {
                let folded_value = self.fold_constants_in_expression(value, count);
                Statement::LetBinding {
                    mutable,
                    name,
                    type_annotation,
                    value: folded_value,
                    ownership_modifier,
                    lifetime_annotation,
                }
            },
            Statement::Function(func_def) => {
                let folded_body: Vec<Statement> = func_def.body
                    .into_iter()
                    .map(|stmt| self.fold_constants_in_statement(stmt, count))
                    .collect();
                
                Statement::Function(FunctionDef {
                    name: func_def.name,
                    parameters: func_def.parameters,
                    return_type: func_def.return_type,
                    body: folded_body,
                    is_async: func_def.is_async,
                    is_public: func_def.is_public,
                    is_awaitable: func_def.is_awaitable,
                    effect_annotations: func_def.effect_annotations,
                    generic_params: func_def.generic_params,
                })
            },
            // Handle other statement types as needed
            _ => stmt, // For unhandled statements, return as-is
        }
    }

    /// Applies constant folding to an expression
    fn fold_constants_in_expression(&self, expr: Expression, count: &mut usize) -> Expression {
        match expr {
            Expression::BinaryOp(left, op, right) => {
                let left_folded = Box::new(self.fold_constants_in_expression(*left, count));
                let right_folded = Box::new(self.fold_constants_in_expression(*right, count));

                // Try to fold the operation if both operands are constants
                if let (Expression::Integer(l_val), Expression::Integer(r_val)) = (left_folded.as_ref(), right_folded.as_ref()) {
                    if let Some(result) = self.perform_integer_operation(*l_val, &op, *r_val) {
                        *count += 1; // Increment count since we folded a constant
                        return Expression::Integer(result);
                    }
                } else if let (Expression::Float(l_val), Expression::Float(r_val)) = (left_folded.as_ref(), right_folded.as_ref()) {
                    if let Some(result) = self.perform_float_operation(*l_val, &op, *r_val) {
                        *count += 1; // Increment count since we folded a constant
                        return Expression::Float(result);
                    }
                }

                Expression::BinaryOp(left_folded, op, right_folded)
            },
            // Handle other expression types as needed
            _ => expr, // For unhandled expressions, return as-is
        }
    }

    /// Performs an integer operation and returns the result if possible
    fn perform_integer_operation(&self, left: i64, op: &BinaryOp, right: i64) -> Option<i64> {
        match op {
            BinaryOp::Add => Some(left + right),
            BinaryOp::Sub => Some(left - right),
            BinaryOp::Mul => Some(left * right),
            BinaryOp::Div => {
                if right != 0 {
                    Some(left / right)
                } else {
                    None // Division by zero - don't fold
                }
            },
            BinaryOp::Mod => {
                if right != 0 {
                    Some(left % right)
                } else {
                    None // Division by zero - don't fold
                }
            },
            BinaryOp::Eq => Some(if left == right { 1 } else { 0 }), // Using 1 for true, 0 for false
            BinaryOp::Ne => Some(if left != right { 1 } else { 0 }),
            BinaryOp::Lt => Some(if left < right { 1 } else { 0 }),
            BinaryOp::Gt => Some(if left > right { 1 } else { 0 }),
            BinaryOp::Le => Some(if left <= right { 1 } else { 0 }),
            BinaryOp::Ge => Some(if left >= right { 1 } else { 0 }),
            _ => None, // Other operations not handled
        }
    }

    /// Performs a float operation and returns the result if possible
    fn perform_float_operation(&self, left: f64, op: &BinaryOp, right: f64) -> Option<f64> {
        match op {
            BinaryOp::Add => Some(left + right),
            BinaryOp::Sub => Some(left - right),
            BinaryOp::Mul => Some(left * right),
            BinaryOp::Div => {
                if right != 0.0 {
                    Some(left / right)
                } else {
                    None // Division by zero - don't fold
                }
            },
            BinaryOp::Eq => Some(if (left - right).abs() < f64::EPSILON { 1.0 } else { 0.0 }), // Using 1.0 for true, 0.0 for false
            BinaryOp::Ne => Some(if (left - right).abs() >= f64::EPSILON { 1.0 } else { 0.0 }),
            BinaryOp::Lt => Some(if left < right { 1.0 } else { 0.0 }),
            BinaryOp::Gt => Some(if left > right { 1.0 } else { 0.0 }),
            BinaryOp::Le => Some(if left <= right { 1.0 } else { 0.0 }),
            BinaryOp::Ge => Some(if left >= right { 1.0 } else { 0.0 }),
            _ => None, // Other operations not handled
        }
    }

    /// Applies dead code elimination
    fn eliminate_dead_code(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd identify and remove unreachable code
        program
    }

    /// Applies common subexpression elimination
    fn eliminate_common_subexpressions(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd identify repeated expressions and optimize them
        program
    }

    /// Applies loop invariant code motion
    fn move_loop_invariants(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd identify loop invariant computations and move them out of loops
        program
    }

    /// Applies function inlining
    fn inline_functions(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd replace function calls with function bodies
        program
    }

    /// Applies tail call optimization
    fn optimize_tail_calls(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd convert tail recursive calls to loops
        program
    }

    /// Applies strength reduction
    fn reduce_strength(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd replace expensive operations with cheaper ones
        program
    }

    /// Applies copy propagation
    fn propagate_copies(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd propagate copies of variables
        program
    }

    /// Applies register allocation
    fn allocate_registers(&self, program: Program) -> Program {
        // For now, we'll just return the program as-is
        // In a full implementation, we'd assign variables to registers
        program
    }
}

impl Default for OptimizationStats {
    fn default() -> Self {
        Self {
            original_instructions: 0,
            optimized_instructions: 0,
            constants_folded: 0,
            dead_code_eliminated: 0,
            functions_inlined: 0,
            tail_calls_optimized: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::new(OptLevel::Standard);
        assert_eq!(optimizer.opt_level, OptLevel::Standard);
        assert!(optimizer.passes.contains(&Pass::ConstantFolding));
        assert!(optimizer.passes.contains(&Pass::DeadCodeElimination));
    }

    #[test]
    fn test_constant_folding() {
        use crate::ast::*;
        
        // Create a simple expression: 2 + 3
        let expr = Expression::BinaryOp(
            Box::new(Expression::Integer(2)),
            BinaryOp::Add,
            Box::new(Expression::Integer(3)),
        );
        
        let stmt = Statement::Expression(expr);
        let program = Program {
            statements: vec![stmt],
        };
        
        let mut optimizer = Optimizer::new(OptLevel::Basic);
        let optimized_program = optimizer.optimize_program(program);
        
        // The expression should be folded to 5
        if let Statement::Expression(Expression::Integer(5)) = &optimized_program.statements[0] {
            assert!(true); // Success
        } else {
            panic!("Constant folding failed");
        }
    }
}