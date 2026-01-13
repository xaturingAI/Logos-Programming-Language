//! Code formatter for the Logos programming language
//! Provides basic formatting capabilities for Logos source code

use crate::ast::*;

/// Format a program according to Logos style guidelines
pub fn format_program(program: &Program) -> String {
    let mut formatter = LogosFormatter::new();
    formatter.format_program(program)
}

/// Logos code formatter
pub struct LogosFormatter {
    indent_level: usize,
    output: String,
}

impl LogosFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    /// Format a program
    pub fn format_program(&mut self, program: &Program) -> String {
        for statement in &program.statements {
            self.format_statement(statement);
            self.output.push('\n'); // Add newline between statements
        }
        self.output.clone()
    }

    /// Format a statement
    fn format_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expr) => {
                self.format_expression(expr);
                self.output.push(';');
            }
            Statement::LetBinding { mutable, name, type_annotation, value, ownership_modifier: _, lifetime_annotation: _ } => {
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.output.push_str("let ");
                self.output.push_str(name);
                if let Some(ty) = type_annotation {
                    self.output.push_str(": ");
                    self.format_type(ty);
                }
                self.output.push_str(" = ");
                self.format_expression(value);
                self.output.push(';');
            }
            Statement::Function(func_def) => {
                self.format_function(func_def);
            }
            Statement::Trait(trait_def) => {
                self.format_trait(trait_def);
            }
            Statement::Implementation(impl_def) => {
                self.format_impl(impl_def);
            }
            Statement::Actor(actor_def) => {
                self.format_actor(actor_def);
            }
            Statement::Effect(effect_def) => {
                self.format_effect(effect_def);
            }
            Statement::Class(class_def) => {
                self.format_class(class_def);
            }
            Statement::MacroDefinition(macro_def) => {
                self.format_macro(macro_def);
            }
            Statement::Return(expr) => {
                self.output.push_str("return ");
                self.format_expression(expr);
                self.output.push(';');
            }
            Statement::Break => {
                self.output.push_str("break;");
            }
            Statement::Continue => {
                self.output.push_str("continue;");
            }
            Statement::Block(statements) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.indent();
                    self.format_statement(stmt);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}");
            }
        }
    }

    /// Format an expression
    fn format_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.output.push_str(name);
            }
            Expression::Integer(value) => {
                self.output.push_str(&value.to_string());
            }
            Expression::Float(value) => {
                self.output.push_str(&value.to_string());
            }
            Expression::String(value) => {
                self.output.push('"');
                self.output.push_str(value);
                self.output.push('"');
            }
            Expression::Char(value) => {
                self.output.push('\'');
                self.output.push(*value);
                self.output.push('\'');
            }
            Expression::Boolean(value) => {
                self.output.push_str(if *value { "true" } else { "false" });
            }
            Expression::Nil => {
                self.output.push_str("nil");
            }
            Expression::BinaryOp(left, op, right) => {
                self.format_expression(left);
                self.output.push(' ');
                self.format_binary_op(op);
                self.output.push(' ');
                self.format_expression(right);
            }
            Expression::UnaryOp(op, operand) => {
                self.format_unary_op(op);
                self.format_expression(operand);
            }
            Expression::Call(func_name, args) => {
                self.output.push_str(func_name);
                self.output.push('(');
                let args_str: Vec<String> = args.iter()
                    .map(|arg| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_expression(arg);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&args_str.join(", "));
                self.output.push(')');
            }
            Expression::Lambda(params, body) => {
                self.output.push('|');
                let param_strs: Vec<String> = params.iter()
                    .map(|param| format!("{}: {}", param.name, param.type_annotation))
                    .collect();
                self.output.push_str(&param_strs.join(", "));
                self.output.push('|');
                self.format_expression(body);
            }
            Expression::If(condition, then_expr, else_expr) => {
                self.output.push_str("if ");
                self.format_expression(condition);
                self.output.push_str(" { ");
                self.format_expression(then_expr);
                if let Some(else_expr) = else_expr {
                    self.output.push_str(" } else { ");
                    self.format_expression(else_expr);
                }
                self.output.push_str(" }");
            }
            Expression::Match(expression, arms) => {
                self.output.push_str("match ");
                self.format_expression(expression);
                self.output.push_str(" { ");
                for (pattern, body) in arms {
                    self.format_pattern(pattern);
                    self.output.push_str(" => ");
                    self.format_expression(body);
                    self.output.push_str(", ");
                }
                self.output.push_str(" }");
            }
            Expression::Tuple(elements) => {
                self.output.push('(');
                let element_strs: Vec<String> = elements.iter()
                    .map(|element| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_expression(element);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&element_strs.join(", "));
                self.output.push(')');
            }
            Expression::Array(elements) => {
                self.output.push('[');
                let element_strs: Vec<String> = elements.iter()
                    .map(|element| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_expression(element);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&element_strs.join(", "));
                self.output.push(']');
            }
            Expression::FieldAccess(object, field) => {
                self.format_expression(object);
                self.output.push('.');
                self.output.push_str(field);
            }
            Expression::MethodCall(object, method, args) => {
                self.format_expression(object);
                self.output.push('.');
                self.output.push_str(method);
                self.output.push('(');
                let args_str: Vec<String> = args.iter()
                    .map(|arg| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_expression(arg);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&args_str.join(", "));
                self.output.push(')');
            }
            Expression::Block(statements) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in statements {
                    self.indent();
                    self.format_statement(stmt);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}");
            }
        }
    }

    /// Format a type
    fn format_type(&mut self, ty: &Type) {
        match ty {
            Type::Int => self.output.push_str("Int"),
            Type::Float => self.output.push_str("Float"),
            Type::String => self.output.push_str("String"),
            Type::Bool => self.output.push_str("Bool"),
            Type::Unit => self.output.push_str("Unit"),
            Type::Array(inner_type) => {
                self.output.push_str("[");
                self.format_type(inner_type);
                self.output.push_str("]");
            }
            Type::Tuple(inner_types) => {
                self.output.push('(');
                let type_strs: Vec<String> = inner_types.iter()
                    .map(|ty| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_type(ty);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&type_strs.join(", "));
                self.output.push(')');
            }
            Type::Function(param_types, return_type) => {
                self.output.push('(');
                let param_strs: Vec<String> = param_types.iter()
                    .map(|ty| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_type(ty);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&param_strs.join(", "));
                self.output.push_str(") -> ");
                self.format_type(return_type);
            }
            Type::Generic(name) => {
                self.output.push_str(name);
            }
            Type::Channel(inner_type) => {
                self.output.push_str("chan ");
                self.format_type(inner_type);
            }
            Type::Pi(param, return_type) => {
                // Dependent function type: (x: A) -> B(x)
                self.output.push('(');
                if let Some(param_name) = &param.name {
                    self.output.push_str(param_name);
                    self.output.push_str(": ");
                }
                self.format_type(&param.type_annotation);
                self.output.push_str(") -> ");
                self.format_type(return_type);
            }
            Type::Sigma(param, second_type) => {
                // Dependent pair type: (x: A, B(x))
                self.output.push('(');
                if let Some(param_name) = &param.name {
                    self.output.push_str(param_name);
                    self.output.push_str(": ");
                }
                self.format_type(&param.type_annotation);
                self.output.push_str(", ");
                self.format_type(second_type);
                self.output.push(')');
            }
            Type::Universe(level) => {
                self.output.push_str(&format!("Type{}", level));
            }
            Type::Equality(ty, left, right) => {
                // Equality type: x =_A y
                self.format_expression(left);
                self.output.push_str(" =_");
                self.format_type(ty);
                self.format_expression(right);
            }
            Type::Linear(name) => {
                self.output.push_str("linear ");
                self.output.push_str(name);
            }
        }
    }

    /// Format a binary operator
    fn format_binary_op(&mut self, op: &BinaryOp) {
        let op_str = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::PipeForward => "|>",
            BinaryOp::PipeBackward => "<|",
            BinaryOp::Power => "^",
            BinaryOp::Range => "..",
            BinaryOp::Spaceship => "<=>",
        };
        self.output.push_str(op_str);
    }

    /// Format a unary operator
    fn format_unary_op(&mut self, op: &UnaryOp) {
        let op_str = match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::Ref => "&",
            UnaryOp::Deref => "*",
        };
        self.output.push_str(op_str);
    }

    /// Format a function definition
    fn format_function(&mut self, func_def: &FunctionDef) {
        self.output.push_str("fn ");
        self.output.push_str(&func_def.name);
        self.output.push('(');
        
        let param_strs: Vec<String> = func_def.parameters.iter()
            .map(|param| {
                let mut param_str = param.name.clone();
                if !param.type_annotation.is_empty() {
                    param_str.push_str(&format!(": {}", param.type_annotation));
                }
                param_str
            })
            .collect();
        
        self.output.push_str(&param_strs.join(", "));
        self.output.push(')');
        
        if let Some(ref ret_type) = func_def.return_type {
            self.output.push_str(" -> ");
            self.output.push_str(ret_type);
        }
        
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for stmt in &func_def.body {
            self.indent();
            self.format_statement(stmt);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format a trait definition
    fn format_trait(&mut self, trait_def: &TraitDef) {
        self.output.push_str("trait ");
        self.output.push_str(&trait_def.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for method in &trait_def.methods {
            self.indent();
            self.format_function(method);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format an implementation definition
    fn format_impl(&mut self, impl_def: &ImplDef) {
        self.output.push_str("impl ");
        self.output.push_str(&impl_def.trait_name);
        self.output.push_str(" for ");
        self.output.push_str(&impl_def.for_type);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for method in &impl_def.methods {
            self.indent();
            self.format_function(method);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format an actor definition
    fn format_actor(&mut self, actor_def: &ActorDef) {
        self.output.push_str("actor ");
        self.output.push_str(&actor_def.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for (name, ty) in &actor_def.state {
            self.indent();
            self.output.push_str(&format!("state {}: {};\n", name, ty));
        }
        for handler in &actor_def.handlers {
            self.indent();
            self.format_function(handler);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format an effect definition
    fn format_effect(&mut self, effect_def: &EffectDef) {
        self.output.push_str("effect ");
        self.output.push_str(&effect_def.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for op in &effect_def.operations {
            self.indent();
            self.format_function(op);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format a class definition
    fn format_class(&mut self, class_def: &ClassDef) {
        self.output.push_str("class ");
        self.output.push_str(&class_def.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for (name, ty) in &class_def.fields {
            self.indent();
            self.output.push_str(&format!("{}: {},\n", name, ty));
        }
        for method in &class_def.methods {
            self.indent();
            self.format_function(method);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format a macro definition
    fn format_macro(&mut self, macro_def: &MacroDef) {
        self.output.push_str("macro ");
        self.output.push_str(&macro_def.name);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        for stmt in &macro_def.body {
            self.indent();
            self.format_statement(stmt);
            self.output.push('\n');
        }
        self.indent_level -= 1;
        self.indent();
        self.output.push_str("}");
    }

    /// Format a pattern
    fn format_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => self.output.push_str(name),
            Pattern::Literal(literal) => self.format_expression(literal),
            Pattern::Wildcard => self.output.push('_'),
            Pattern::Tuple(patterns) => {
                self.output.push('(');
                let pattern_strs: Vec<String> = patterns.iter()
                    .map(|pat| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_pattern(pat);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&pattern_strs.join(", "));
                self.output.push(')');
            }
            Pattern::Struct(name, fields) => {
                self.output.push_str(name);
                self.output.push_str(" { ");
                let field_strs: Vec<String> = fields.iter()
                    .map(|(field_name, field_pattern)| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_pattern(field_pattern);
                        format!("{}: {}", field_name, formatter.output)
                    })
                    .collect();
                self.output.push_str(&field_strs.join(", "));
                self.output.push_str(" }");
            }
            Pattern::Array(patterns) => {
                self.output.push('[');
                let pattern_strs: Vec<String> = patterns.iter()
                    .map(|pat| {
                        let mut formatter = LogosFormatter::new();
                        formatter.format_pattern(pat);
                        formatter.output
                    })
                    .collect();
                self.output.push_str(&pattern_strs.join(", "));
                self.output.push(']');
            }
            Pattern::Or(left, right) => {
                self.format_pattern(left);
                self.output.push_str(" | ");
                self.format_pattern(right);
            }
        }
    }

    /// Add indentation to the output
    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    "); // 4 spaces per indent level
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let formatter = LogosFormatter::new();
        assert_eq!(formatter.indent_level, 0);
    }

    #[test]
    fn test_format_integer() {
        let mut formatter = LogosFormatter::new();
        let expr = Expression::Integer(42);
        formatter.format_expression(&expr);
        assert_eq!(formatter.output, "42");
    }

    #[test]
    fn test_format_string() {
        let mut formatter = LogosFormatter::new();
        let expr = Expression::String("hello".to_string());
        formatter.format_expression(&expr);
        assert_eq!(formatter.output, "\"hello\"");
    }
}