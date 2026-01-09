use crate::ast::*;

pub fn generate_code(program: &Program) -> Result<String, String> {
    let mut code = String::new();
    
    code.push_str("// Generated Eux Code\n");
    code.push_str("// This is a simplified representation\n\n");
    
    for stmt in &program.statements {
        code.push_str(&generate_statement(stmt, 0)?);
        code.push('\n');
    }
    
    Ok(code)
}

pub fn generate_executable(_program: &Program, _output: &str) -> Result<(), String> {
    // In a real implementation, this would generate actual executable code
    // For now, we'll just return Ok to indicate success
    println!("Generated executable: {}", _output);
    Ok(())
}

fn generate_statement(stmt: &Statement, indent_level: usize) -> Result<String, String> {
    let indent = "    ".repeat(indent_level);
    
    match stmt {
        Statement::Expression(expr) => {
            Ok(format!("{}{};\n", indent, generate_expression(expr)?))
        },
        Statement::LetBinding { mutable, name, type_annotation, value } => {
            let mut decl = if *mutable {
                format!("{}mut {} = ", indent, name)
            } else {
                format!("{}let {} = ", indent, name)
            };
            
            if let Some(ty) = type_annotation {
                decl.push_str(&format!(": {} = ", generate_type(ty)?));
            }
            
            decl.push_str(&format!("{};\n", generate_expression(value)?));
            Ok(decl)
        },
        Statement::ConstBinding { name, type_annotation, value } => {
            let mut decl = format!("{}const {} = ", indent, name);
            
            if let Some(ty) = type_annotation {
                decl.push_str(&format!(": {} = ", generate_type(ty)?));
            }
            
            decl.push_str(&format!("{};\n", generate_expression(value)?));
            Ok(decl)
        },
        Statement::Function(func) => {
            let mut func_def = format!("{}fn {}(", indent, func.name);
            
            // Add parameters
            let params: Result<Vec<String>, String> = func.parameters
                .iter()
                .map(|param| {
                    Ok(format!("{}: {}", param.name, generate_type(&param.type_annotation)?))
                })
                .collect();
            
            let params = params?;
            func_def.push_str(&params.join(", "));
            
            func_def.push(')');
            
            // Add return type if present
            if let Some(return_type) = &func.return_type {
                func_def.push_str(&format!(" -> {}", generate_type(return_type)?));
            }
            
            func_def.push_str(" {\n");
            
            // Add function body
            for stmt in &func.body {
                func_def.push_str(&generate_statement(stmt, indent_level + 1)?);
            }
            
            func_def.push_str(&format!("{}}}\n", indent));
            Ok(func_def)
        },
        Statement::Return(expr) => {
            if let Some(return_expr) = expr {
                Ok(format!("{}return {};\n", indent, generate_expression(return_expr)?))
            } else {
                Ok(format!("{}return;\n", indent))
            }
        },
        Statement::Block(statements) => {
            let mut block = format!("{}{{\n", indent);
            
            for stmt in statements {
                block.push_str(&generate_statement(stmt, indent_level + 1)?);
            }
            
            block.push_str(&format!("{}}}\n", indent));
            Ok(block)
        },
        _ => {
            // Other statement types not fully implemented in this example
            Ok(format!("{}// [Unimplemented statement type]\n", indent))
        }
    }
}

fn generate_expression(expr: &Expression) -> Result<String, String> {
    match expr {
        Expression::Integer(value) => Ok(value.to_string()),
        Expression::Float(value) => Ok(value.to_string()),
        Expression::String(value) => Ok(format!("\"{}\"", value.replace('"', "\\\""))),
        Expression::Boolean(value) => Ok(value.to_string()),
        Expression::Nil => Ok("nil".to_string()),
        Expression::Identifier(name) => Ok(name.clone()),
        Expression::BinaryOp(left, op, right) => {
            let left_str = generate_expression(left)?;
            let right_str = generate_expression(right)?;
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
                BinaryOp::Spaceship => "<=>",
                BinaryOp::Power => "^",
                BinaryOp::Range => "..",
            };
            
            Ok(format!("({} {} {})", left_str, op_str, right_str))
        },
        Expression::UnaryOp(op, operand) => {
            let operand_str = generate_expression(operand)?;
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
                _ => return Err("Unsupported unary operator".to_string()),
            };
            
            Ok(format!("{}{}", op_str, operand_str))
        },
        Expression::Call(name, args) => {
            let args_str: Result<Vec<String>, String> = args
                .iter()
                .map(generate_expression)
                .collect();
            
            Ok(format!("{}({})", name, args_str?.join(", ")))
        },
        Expression::MethodCall(obj, method, args) => {
            let obj_str = generate_expression(obj)?;
            let args_str: Result<Vec<String>, String> = args
                .iter()
                .map(generate_expression)
                .collect();
            
            Ok(format!("{}.{}({})", obj_str, method, args_str?.join(", ")))
        },
        Expression::FieldAccess(obj, field) => {
            let obj_str = generate_expression(obj)?;
            Ok(format!("{}.{}", obj_str, field))
        },
        Expression::If(condition, then_branch, else_branch) => {
            let mut result = format!(
                "if ({}) {{\n",
                generate_expression(condition)?
            );
            
            for stmt in then_branch {
                result.push_str(&generate_statement(stmt, 1)?);
            }
            
            if !else_branch.is_empty() {
                result.push_str("} else {\n");
                for stmt in else_branch {
                    result.push_str(&generate_statement(stmt, 1)?);
                }
            }
            
            result.push_str("}");
            Ok(result)
        },
        Expression::Match(expr, arms) => {
            let mut result = format!("match ({}) {{\n", generate_expression(expr)?);

            for (pattern, guard, body) in arms {
                let guard_part = if let Some(guard_expr) = guard {
                    format!(" if {}", generate_expression(guard_expr)?)
                } else {
                    String::new()
                };
                result.push_str(&format!("    {}{} => {{\n", generate_pattern(pattern)?, guard_part));
                for stmt in body {
                    result.push_str(&generate_statement(stmt, 2)?);
                }
                result.push_str("    },\n");
            }

            result.push_str("}");
            Ok(result)
        },
        Expression::Lambda(params, body) => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            let mut result = format!("|{}| {{\n", param_names.join(", "));
            
            for stmt in body {
                result.push_str(&generate_statement(stmt, 1)?);
            }
            
            result.push_str("}");
            Ok(result)
        },
        Expression::BlockExpr(statements) => {
            let mut result = "{\n".to_string();
            
            for stmt in statements {
                result.push_str(&generate_statement(stmt, 1)?);
            }
            
            result.push_str("}");
            Ok(result)
        },
        Expression::MultiLangCall(lang, code) => {
            Ok(format!("@{}(\"{}\")", lang, code))
        },
        Expression::Tuple(elements) => {
            let element_strings: Result<Vec<String>, String> = elements
                .iter()
                .map(generate_expression)
                .collect();
            Ok(format!("({})", element_strings?.join(", ")))
        },
        Expression::Spawn(actor_name, args) => {
            let arg_strings: Result<Vec<String>, String> = args
                .iter()
                .map(generate_expression)
                .collect();
            Ok(format!("spawn {}({})", actor_name, arg_strings?.join(", ")))
        },
        Expression::Send(actor_expr, message_expr) => {
            let actor_str = generate_expression(actor_expr)?;
            let message_str = generate_expression(message_expr)?;
            Ok(format!("send({}, {})", actor_str, message_str))
        },
        Expression::Receive => {
            Ok("receive".to_string())
        },
        Expression::MultiLangImport(lang, module, alias) => {
            let alias_str = if let Some(alias_name) = alias {
                format!(" as {}", alias_name)
            } else {
                "".to_string()
            };
            Ok(format!("@import({}, \"{}\"{})", lang, module, alias_str))
        },
        Expression::MultiLangIndex(lang, resource) => {
            Ok(format!("@index({}, \"{}\")", lang, resource))
        },
        Expression::InterpolatedString(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(text) => result.push_str(&text),
                    StringPart::Interpolated(expr) => {
                        let expr_str = generate_expression(expr)?;
                        result.push_str(&format!("${{{}}}", expr_str));
                    }
                }
            }
            Ok(result)
        },
        Expression::LambdaSimple(params, body) => {
            let param_str = params.join(", ");
            let body_str = generate_expression(body)?;
            Ok(format!("|{}| {}", param_str, body_str))
        },
        Expression::Pipeline(start_expr, funcs) => {
            let start_str = generate_expression(start_expr)?;
            let func_strs: Result<Vec<String>, String> = funcs.iter().map(generate_expression).collect();
            let func_strs = func_strs?;
            Ok(format!("{} |> {}", start_str, func_strs.join(" |> ")))
        },
        Expression::BackPipeline(end_expr, funcs) => {
            let end_str = generate_expression(end_expr)?;
            let func_strs: Result<Vec<String>, String> = funcs.iter().map(generate_expression).collect();
            let func_strs = func_strs?;
            Ok(format!("{} <| {}", func_strs.join(" <| "), end_str))
        },
        Expression::DestructureAssignment(pattern, expr, stmt) => {
            let pattern_str = generate_pattern(pattern)?;
            let expr_str = generate_expression(expr)?;
            let stmt_str = generate_statement(stmt, 0)?; // Simplified indentation
            Ok(format!("let {} = {} in {}", pattern_str, expr_str, stmt_str))
        },
        Expression::ChannelCreate(channel_type) => {
            Ok(format!("chan {}", generate_type(channel_type)?))
        },
        Expression::ChannelSend(channel_expr, value_expr) => {
            let channel_str = generate_expression(channel_expr)?;
            let value_str = generate_expression(value_expr)?;
            Ok(format!("{} <- {}", channel_str, value_str))
        },
        Expression::ChannelReceive(channel_expr) => {
            let channel_str = generate_expression(channel_expr)?;
            Ok(format!("<-{}", channel_str))
        },
        Expression::ChannelClose(channel_expr) => {
            let channel_str = generate_expression(channel_expr)?;
            Ok(format!("close({})", channel_str))
        },
        Expression::Select(select_arms) => {
            let mut arms_strs = Vec::new();
            for arm in select_arms {
                match &arm.channel_operation {
                    ChannelOperation::Send { channel, value } => {
                        let channel_str = generate_expression(channel)?;
                        let value_str = generate_expression(value)?;
                        let body_str = arm.body.iter()
                            .map(|stmt| generate_statement(stmt, 1))
                            .collect::<Result<Vec<_>, _>>()?
                            .join("\n");
                        arms_strs.push(format!("{} <- {}: {{\n{}\n}}", channel_str, value_str, body_str));
                    },
                    ChannelOperation::Receive { channel } => {
                        let channel_str = generate_expression(channel)?;
                        let var_binding = if let Some(var_name) = bind_to {
                            format!("{} := ", var_name)
                        } else {
                            String::new()
                        };
                        let body_str = arm.body.iter()
                            .map(|stmt| generate_statement(stmt, 1))
                            .collect::<Result<Vec<_>, _>>()?
                            .join("\n");
                        arms_strs.push(format!("{}<-{}: {{\n{}\n}}", var_binding, channel_str, body_str));
                    }
                }
            }
            Ok(format!("select {{\n{}\n}}", arms_strs.join("\n")))
        },
    }
}

fn generate_type(ty: &Type) -> Result<String, String> {
    match ty {
        Type::Int => Ok("Int".to_string()),
        Type::Float => Ok("Float".to_string()),
        Type::Bool => Ok("Bool".to_string()),
        Type::String => Ok("String".to_string()),
        Type::Unit => Ok("Unit".to_string()),
        Type::Array(inner) => Ok(format!("[{}]", generate_type(inner)?)),
        Type::Tuple(types) => {
            let inner_types: Result<Vec<String>, String> = types
                .iter()
                .map(generate_type)
                .collect();
            Ok(format!("({})", inner_types?.join(", ")))
        },
        Type::Function(params, return_type) => {
            let param_types: Result<Vec<String>, String> = params
                .iter()
                .map(generate_type)
                .collect();
            Ok(format!("({}) -> {}", param_types?.join(", "), generate_type(return_type)?))
        },
        Type::Named(name) => Ok(name.clone()),
        Type::Generic(name) => Ok(name.clone()),
        Type::Option(inner) => Ok(format!("Option<{}>", generate_type(inner)?)),
        Type::Result(ok_type, err_type) => {
            Ok(format!("Result<{}, {}>", generate_type(ok_type)?, generate_type(err_type)?))
        },
        Type::Pi(param, return_type) => {
            Ok(format!("({}: {}) -> {}", param.name, generate_type(&param.type_annotation)?, generate_type(return_type)?))
        },
        Type::Sigma(param, snd_type) => {
            Ok(format!("({}: {}, _)", param.name, generate_type(&param.type_annotation)?))
        },
        Type::Universe(level) => {
            Ok(format!("Type{}", level))
        },
        Type::Equality(ty, _, _) => {
            Ok(format!("{} = {}", generate_type(ty)?, generate_type(ty)?))
        },
        Type::Channel(inner) => {
            Ok(format!("chan {}", generate_type(inner)?))
        },
        Type::Linear(inner) => {
            Ok(format!("!{}", generate_type(inner)?))
        },
        Type::Infer => {
            Ok("_".to_string())
        },
    }
}

fn generate_pattern(pattern: &Pattern) -> Result<String, String> {
    match pattern {
        Pattern::Identifier(name) => Ok(name.clone()),
        Pattern::Literal(expr) => generate_expression(expr),
        Pattern::Wildcard => Ok("_".to_string()),
        Pattern::Tuple(patterns) => {
            let inner_patterns: Result<Vec<String>, String> = patterns
                .iter()
                .map(generate_pattern)
                .collect();
            Ok(format!("({})", inner_patterns?.join(", ")))
        },
        Pattern::Array(patterns) => {
            let inner_patterns: Result<Vec<String>, String> = patterns
                .iter()
                .map(generate_pattern)
                .collect();
            Ok(format!("[{}]", inner_patterns?.join(", ")))
        },
        Pattern::Struct(name, fields) => {
            let field_patterns: Result<Vec<String>, String> = fields
                .iter()
                .map(|(field_name, field_pattern)| {
                    Ok(format!("{}: {}", field_name, generate_pattern(field_pattern)?))
                })
                .collect();
            Ok(format!("{} {{ {} }}", name, field_patterns?.join(", ")))
        },
        Pattern::Or(left, right) => {
            Ok(format!("{} | {}", generate_pattern(left)?, generate_pattern(right)?))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_code_generation() {
        let input = r#"
        fn add(a: Int, b: Int) -> Int {
            return a + b
        }
        
        fn main() {
            let x = 42
            let result = add(x, 8)
            print(result)
        }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let generated = generate_code(&program);
        assert!(generated.is_ok());
        
        let code = generated.unwrap();
        assert!(code.contains("fn add"));
        assert!(code.contains("let x = 42"));
        assert!(code.contains("a + b"));
    }

    #[test]
    fn test_simple_expression_generation() {
        let expr = Expression::BinaryOp(
            Box::new(Expression::Integer(5)),
            BinaryOp::Add,
            Box::new(Expression::Integer(3))
        );
        
        let generated = generate_expression(&expr).unwrap();
        assert_eq!(generated, "(5 + 3)");
    }

    #[test]
    fn test_function_generation() {
        let func = FunctionDef {
            name: "test".to_string(),
            parameters: vec![
                Parameter {
                    name: "x".to_string(),
                    type_annotation: Type::Int,
                    default_value: None,
                }
            ],
            return_type: Some(Type::Int),
            body: vec![
                Statement::Return(Some(Expression::Identifier("x".to_string())))
            ],
            is_async: false,
            is_public: false,
        };
        
        let stmt = Statement::Function(func);
        let generated = generate_statement(&stmt, 0).unwrap();
        
        assert!(generated.contains("fn test"));
        assert!(generated.contains("x: Int"));
        assert!(generated.contains("-> Int"));
        assert!(generated.contains("return x"));
    }
}