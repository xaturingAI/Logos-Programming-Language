// Logos Programming Language Code Generator
// This module transforms the AST into executable code or intermediate representation.

use crate::ast::*;

/// Code generator for converting AST to target code
pub struct CodeGen {
    output: String,
    indent_level: usize,
}

impl CodeGen {
    /// Creates a new code generator instance
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Generates code from an AST program
    pub fn generate_program(&mut self, program: &Program) -> String {
        for statement in &program.statements {
            self.generate_statement(statement);
        }
        self.output.clone()
    }

    /// Generates code for a statement
    fn generate_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expr) => {
                let expr_code = self.generate_expression(expr);
                self.emit(&format!("{};", expr_code));
            },
            Statement::LetBinding { mutable, name, type_annotation, value, ownership_modifier: _, lifetime_annotation: _ } => {
                let value_code = self.generate_expression(value);
                let mut_type = if *mutable { "mut " } else { "" };
                
                match type_annotation {
                    Some(ty) => {
                        let type_str = self.type_to_string(ty);
                        self.emit(&format!("let {}{}: {} = {};", mut_type, name, type_str, value_code));
                    },
                    None => {
                        self.emit(&format!("let {}{} = {};", mut_type, name, value_code));
                    }
                }
            },
            Statement::ConstBinding { name, type_annotation, value } => {
                let value_code = self.generate_expression(value);
                
                match type_annotation {
                    Some(ty) => {
                        let type_str = self.type_to_string(ty);
                        self.emit(&format!("const {}: {} = {};", name, type_str, value_code));
                    },
                    None => {
                        self.emit(&format!("const {} = {};", name, value_code));
                    }
                }
            },
            Statement::Function(func_def) => {
                self.generate_function(func_def);
            },
            Statement::Return(expr) => {
                match expr {
                    Some(return_expr) => {
                        let expr_code = self.generate_expression(return_expr);
                        self.emit(&format!("return {};", expr_code));
                    },
                    None => {
                        self.emit("return;");
                    }
                }
            },
            Statement::Block(statements) => {
                self.emit("{");
                self.indent_level += 1;
                for stmt in statements {
                    self.generate_statement(stmt);
                }
                self.indent_level -= 1;
                self.emit("}");
            },
            // Handle other statement types as needed
            _ => {
                // For now, emit a comment for unhandled statements
                self.emit("// [Unhandled statement]");
            }
        }
    }

    /// Generates code for a function definition
    fn generate_function(&mut self, func_def: &FunctionDef) {
        let params_str = self.generate_parameters(&func_def.parameters);
        let return_type_str = match &func_def.return_type {
            Some(ty) => format!(" -> {}", self.type_to_string(ty)),
            None => String::new(),
        };

        self.emit(&format!("fn {}({}){} {{", func_def.name, params_str, return_type_str));
        self.indent_level += 1;
        
        for stmt in &func_def.body {
            self.generate_statement(stmt);
        }
        
        self.indent_level -= 1;
        self.emit("}");
    }

    /// Generates code for function parameters
    fn generate_parameters(&mut self, parameters: &[Parameter]) -> String {
        let param_strings: Vec<String> = parameters
            .iter()
            .map(|param| {
                let type_str = self.type_to_string(&param.type_annotation);
                format!("{}: {}", param.name, type_str)
            })
            .collect();
        
        param_strings.join(", ")
    }

    /// Generates code for an expression
    fn generate_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Integer(val) => val.to_string(),
            Expression::Float(val) => val.to_string(),
            Expression::String(val) => format!("\"{}\"", val.escape_debug()),
            Expression::Boolean(val) => val.to_string(),
            Expression::Nil => "nil".to_string(),
            Expression::Identifier(name) => name.clone(),
            Expression::BinaryOp(left, op, right) => {
                let left_str = self.generate_expression(left);
                let right_str = self.generate_expression(right);
                let op_str = self.binary_op_to_string(op);
                format!("({} {} {})", left_str, op_str, right_str)
            },
            Expression::UnaryOp(op, expr) => {
                let expr_str = self.generate_expression(expr);
                let op_str = self.unary_op_to_string(op);
                format!("{}{}", op_str, expr_str)
            },
            Expression::Call(name, args) => {
                let arg_strings: Vec<String> = args
                    .iter()
                    .map(|arg| self.generate_expression(arg))
                    .collect();
                format!("{}({})", name, arg_strings.join(", "))
            },
            Expression::If(condition, then_stmts, else_stmts) => {
                let cond_str = self.generate_expression(condition);
                let then_str = self.generate_block(then_stmts);
                
                if !else_stmts.is_empty() {
                    let else_str = self.generate_block(else_stmts);
                    format!("if {} {} else {}", cond_str, then_str, else_str)
                } else {
                    format!("if {} {}", cond_str, then_str)
                }
            },
            Expression::Tuple(items) => {
                let item_strings: Vec<String> = items
                    .iter()
                    .map(|item| self.generate_expression(item))
                    .collect();
                format!("({})", item_strings.join(", "))
            },
            // Handle other expression types as needed
            _ => {
                // For now, return a placeholder for unhandled expressions
                String::from("[unhandled expression]")
            }
        }
    }

    /// Generates code for a block of statements
    fn generate_block(&mut self, statements: &[Statement]) -> String {
        if statements.is_empty() {
            return "{}".to_string();
        }

        let mut block = String::from("{\n");
        let old_indent = self.indent_level;
        self.indent_level += 1;

        for stmt in statements {
            let mut stmt_gen = CodeGen::new();
            stmt_gen.indent_level = self.indent_level;
            stmt_gen.generate_statement(stmt);
            
            // Extract the generated code and add proper indentation
            let stmt_lines: Vec<&str> = stmt_gen.output.trim_end().split('\n').collect();
            for line in stmt_lines {
                if !line.trim().is_empty() {
                    block.push_str(&self.indent());
                    block.push_str(line);
                    block.push('\n');
                }
            }
        }

        self.indent_level = old_indent;
        block.push_str(&self.indent());
        block.push('}');
        block
    }

    /// Converts a binary operation to its string representation
    fn binary_op_to_string(&self, op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => "+".to_string(),
            BinaryOp::Sub => "-".to_string(),
            BinaryOp::Mul => "*".to_string(),
            BinaryOp::Div => "/".to_string(),
            BinaryOp::Mod => "%".to_string(),
            BinaryOp::Eq => "==".to_string(),
            BinaryOp::Ne => "!=".to_string(),
            BinaryOp::Lt => "<".to_string(),
            BinaryOp::Gt => ">".to_string(),
            BinaryOp::Le => "<=".to_string(),
            BinaryOp::Ge => ">=".to_string(),
            BinaryOp::And => "&&".to_string(),
            BinaryOp::Or => "||".to_string(),
            BinaryOp::PipeForward => "|>".to_string(),
            BinaryOp::PipeBackward => "<|".to_string(),
            BinaryOp::Power => "^".to_string(),
            BinaryOp::Range => "..".to_string(),
            BinaryOp::Spaceship => "<=>".to_string(),
        }
    }

    /// Converts a unary operation to its string representation
    fn unary_op_to_string(&self, op: &UnaryOp) -> String {
        match op {
            UnaryOp::Neg => "-".to_string(),
            UnaryOp::Not => "!".to_string(),
            UnaryOp::Ref => "&".to_string(),
            UnaryOp::Deref => "*".to_string(),
        }
    }

    /// Converts a type to its string representation
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::Array(inner) => format!("[{}]", self.type_to_string(inner)),
            Type::Tuple(types) => {
                let type_strings: Vec<String> = types
                    .iter()
                    .map(|ty| self.type_to_string(ty))
                    .collect();
                format!("({})", type_strings.join(", "))
            },
            Type::Function(params, ret) => {
                let param_strings: Vec<String> = params
                    .iter()
                    .map(|ty| self.type_to_string(ty))
                    .collect();
                format!("({}) -> {}", param_strings.join(", "), self.type_to_string(ret))
            },
            Type::Named(name) => name.clone(),
            Type::Generic(name) => name.clone(),
            Type::Option(inner) => format!("Option<{}>", self.type_to_string(inner)),
            Type::Result(ok, err) => format!("Result<{}, {}>", self.type_to_string(ok), self.type_to_string(err)),
            Type::Infer => "auto".to_string(),
            // Handle other type variants as needed
            _ => "unknown".to_string(),
        }
    }

    /// Emits a line of code with proper indentation
    fn emit(&mut self, code: &str) {
        if !code.is_empty() {
            self.output.push_str(&self.indent());
            self.output.push_str(code);
        }
        self.output.push('\n');
    }

    /// Generates the current indentation string
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)  // 4 spaces per indent level
    }
}

/// Generates code from an AST program
pub fn generate_code(program: &Program) -> String {
    let mut codegen = CodeGen::new();
    codegen.generate_program(program)
}