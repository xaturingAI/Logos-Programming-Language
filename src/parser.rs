// Logos Programming Language Parser
// This module provides parsing functionality for the Logos programming language.
// It converts a sequence of tokens from the lexer into an Abstract Syntax Tree (AST)
// that represents the structure of the program.

use crate::lexer::{Lexer, Token};
use crate::ast::*;

/// The Parser struct processes tokens and converts them into AST nodes
pub struct Parser<'a> {
    lexer: Lexer<'a>,        // The lexer that provides tokens
    current_token: Token,    // The current token being processed
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance for the given input string
    /// 
    /// # Arguments
    /// * `input` - The source code string to parse
    /// 
    /// # Returns
    /// A new Parser instance ready to parse the input
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
        }
    }

    /// Parses the entire input program into an AST
    /// 
    /// # Returns
    /// A Result containing the Program AST node if successful, or an error string
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !matches!(self.current_token, Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    /// Advances the parser to the next token from the lexer
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    /// Gets a reference to the current token
    /// 
    /// # Returns
    /// A reference to the current token being processed
    fn current_token(&self) -> &Token {
        &self.current_token
    }

    /// Parses a single statement from the input
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Token::Let | Token::Mut => self.parse_variable_declaration(),
            Token::Const => self.parse_const_declaration(),
            Token::Fn => self.parse_function(),
            Token::If => self.parse_if_statement(),
            Token::Match => self.parse_match_statement(),
            Token::Actor => self.parse_actor(),
            Token::Effect => self.parse_effect(),
            Token::Class => self.parse_class(),
            Token::Trait => self.parse_trait(),
            Token::Impl => self.parse_impl(),
            Token::Macro => self.parse_macro(),
            Token::Break => {
                self.advance(); // consume break
                Ok(Statement::Break)
            },
            Token::Continue => {
                self.advance(); // consume continue
                Ok(Statement::Continue)
            },
            Token::Return => {
                self.advance(); // consume return
                let expr = if !matches!(self.current_token(), Token::Semicolon | Token::Eof) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                Ok(Statement::Return(expr))
            }
            Token::LeftBrace => {
                // Block statement
                self.parse_block_statement()
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    /// Parses an expression (the right-hand side of assignments, function arguments, etc.)
    ///
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_assignment()
    }

    /// Parses a struct literal expression
    ///
    /// # Returns
    /// A Result containing the Struct Expression AST node if successful, or an error string
    fn parse_struct_literal(&mut self, struct_name: String) -> Result<Expression, String> {
        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{' after struct name".to_string());
        }
        self.advance(); // consume {

        let mut fields = Vec::new();

        if !matches!(self.current_token(), Token::RightBrace) {
            // Parse first field
            if let Token::Identifier(field_name) = self.current_token().clone() {
                self.advance(); // consume field name
                if !matches!(self.current_token(), Token::Colon) {
                    return Err("Expected ':' after field name".to_string());
                }
                self.advance(); // consume :

                let value = self.parse_expression()?;
                fields.push((field_name, value));

                // Parse additional fields
                while matches!(self.current_token(), Token::Comma) {
                    self.advance(); // consume ,

                    if let Token::Identifier(field_name) = self.current_token().clone() {
                        self.advance(); // consume field name
                        if !matches!(self.current_token(), Token::Colon) {
                            return Err("Expected ':' after field name".to_string());
                        }
                        self.advance(); // consume :

                        let value = self.parse_expression()?;
                        fields.push((field_name, value));
                    } else {
                        return Err("Expected field name after comma".to_string());
                    }
                }
            } else {
                return Err("Expected field name".to_string());
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Expression::Struct(struct_name, fields))
    }

    /// Parses an assignment expression (left = right)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_pipeline()?;

        if matches!(self.current_token(), Token::Assign) {
            self.advance(); // consume =
            let right = self.parse_assignment()?;
            // For now, treat as a function call to an assignment function
            if let Expression::Identifier(name) = left {
                Ok(Expression::Call("assign".to_string(), vec![
                    Expression::String(name),
                    right
                ]))
            } else {
                Err("Invalid assignment target".to_string())
            }
        } else {
            Ok(left)
        }
    }

    /// Parses pipeline expressions (left |> right or left <| right)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_pipeline(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_range()?;

        loop {
            match self.current_token() {
                Token::PipeForward => {
                    self.advance(); // consume |>
                    let right = self.parse_range()?;
                    left = Expression::BinaryOp(
                        Box::new(left),
                        BinaryOp::PipeForward,
                        Box::new(right)
                    );
                }
                Token::PipeBackward => {
                    self.advance(); // consume <|
                    let right = self.parse_range()?;
                    left = Expression::BinaryOp(
                        Box::new(left),
                        BinaryOp::PipeBackward,
                        Box::new(right)
                    );
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parses range expressions (left .. right)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_range(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;

        if matches!(self.current_token(), Token::Range) {
            self.advance(); // consume ..
            let right = self.parse_range()?; // right associative
            Ok(Expression::BinaryOp(Box::new(left), BinaryOp::Range, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Parses equality expressions (==, !=)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.current_token() {
                Token::Equal => BinaryOp::Eq,
                Token::NotEqual => BinaryOp::Ne,
                _ => break,
            };

            self.advance(); // consume operator
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses comparison expressions (<, <=, >, >=, <=>)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_term()?;

        loop {
            let op = match self.current_token() {
                Token::Less => BinaryOp::Lt,
                Token::LessEqual => BinaryOp::Le,
                Token::Greater => BinaryOp::Gt,
                Token::GreaterEqual => BinaryOp::Ge,
                Token::Spaceship => BinaryOp::Spaceship,
                _ => break,
            };

            self.advance(); // consume operator
            let right = self.parse_term()?;
            left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses term expressions (+, -)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_factor()?;

        loop {
            let op = match self.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                Token::LeftArrow => {
                    // Handle channel send: left <- right
                    self.advance(); // consume <-
                    let right = self.parse_factor()?;
                    return Ok(Expression::ChannelSend(Box::new(left), Box::new(right)));
                },
                _ => break,
            };

            self.advance(); // consume operator
            let right = self.parse_factor()?;
            left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses factor expressions (*, /, %)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_exponentiation()?;

        loop {
            let op = match self.current_token() {
                Token::Multiply => BinaryOp::Mul,
                Token::Divide => BinaryOp::Div,
                Token::Modulo => BinaryOp::Mod,
                _ => break,
            };

            self.advance(); // consume operator
            let right = self.parse_exponentiation()?;
            left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// Parses exponentiation expressions (^)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_exponentiation(&mut self) -> Result<Expression, String> {
        let left = self.parse_unary()?;

        if matches!(self.current_token(), Token::Power) {
            self.advance(); // consume ^
            let right = self.parse_exponentiation()?; // right associative
            Ok(Expression::BinaryOp(Box::new(left), BinaryOp::Power, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Parses unary expressions (-, !, <- for channel receive)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.current_token() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::UnaryOp(UnaryOp::Neg, Box::new(expr)))
            }
            Token::Not => {
                self.advance();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            Token::LeftArrow => {
                // Handle channel receive: <-expression
                self.advance();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::ChannelReceive(Box::new(expr)))
            }
            _ => self.parse_call(),
        }
    }

    /// Parses function calls and method calls
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current_token() {
                Token::LeftParen => {
                    self.advance(); // consume (
                    let args = self.parse_arguments()?;
                    expr = match expr {
                        Expression::Identifier(name) => {
                            Expression::Call(name, args)
                        },
                        _ => return Err("Expected function name".to_string()),
                    };
                }
                Token::Dot => {
                    self.advance(); // consume .
                    if let Token::Identifier(method_name) = self.current_token().clone() {
                        self.advance(); // consume method name
                        if matches!(self.current_token(), Token::LeftParen) {
                            self.advance(); // consume (
                            let args = self.parse_arguments()?;
                            expr = Expression::MethodCall(
                                Box::new(expr),
                                method_name,
                                args
                            );
                        } else {
                            expr = Expression::FieldAccess(
                                Box::new(expr),
                                method_name
                            );
                        }
                    } else {
                        return Err("Expected method or field name".to_string());
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parses function call arguments
    /// 
    /// # Returns
    /// A Result containing a vector of Expression AST nodes if successful, or an error string
    fn parse_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut args = Vec::new();

        if matches!(self.current_token(), Token::RightParen) {
            self.advance(); // consume )
            return Ok(args);
        }

        args.push(self.parse_expression()?);

        while matches!(self.current_token(), Token::Comma) {
            self.advance(); // consume ,
            args.push(self.parse_expression()?);
        }

        if !matches!(self.current_token(), Token::RightParen) {
            return Err("Expected ')'".to_string());
        }
        self.advance(); // consume )

        Ok(args)
    }

    /// Parses primary expressions (literals, identifiers, grouped expressions)
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.current_token().clone() {
            Token::Integer(value) => {
                self.advance();
                Ok(Expression::Integer(value))
            }
            Token::Float(value) => {
                self.advance();
                Ok(Expression::Float(value))
            }
            Token::String(value) => {
                self.advance();
                Ok(Expression::String(value))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            Token::Nil => {
                self.advance();
                Ok(Expression::Nil)
            }
            Token::Char(value) => {
                self.advance();
                Ok(Expression::Char(value))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            Token::LeftParen => {
                self.advance(); // consume (

                // Check if this is an empty tuple
                if matches!(self.current_token(), Token::RightParen) {
                    self.advance(); // consume )
                    return Ok(Expression::Tuple(vec![]));
                }

                let first_expr = self.parse_expression()?;

                // Check if this is a tuple (has comma after first element)
                if matches!(self.current_token(), Token::Comma) {
                    // This is a tuple - parse the rest of the elements
                    let mut elements = vec![first_expr];

                    while matches!(self.current_token(), Token::Comma) {
                        self.advance(); // consume ,
                        elements.push(self.parse_expression()?);
                    }

                    if !matches!(self.current_token(), Token::RightParen) {
                        return Err("Expected ')'".to_string());
                    }
                    self.advance(); // consume )

                    Ok(Expression::Tuple(elements))
                } else {
                    // Single expression in parentheses
                    if !matches!(self.current_token(), Token::RightParen) {
                        return Err("Expected ')'".to_string());
                    }
                    self.advance(); // consume )
                    Ok(first_expr)
                }
            }
            Token::LeftBracket => {
                // Parse array literal
                self.advance(); // consume [
                let mut elements = Vec::new();

                if !matches!(self.current_token(), Token::RightBracket) {
                    elements.push(self.parse_expression()?);

                    while matches!(self.current_token(), Token::Comma) {
                        self.advance(); // consume ,
                        elements.push(self.parse_expression()?);
                    }
                }

                if !matches!(self.current_token(), Token::RightBracket) {
                    return Err("Expected ']'".to_string());
                }
                self.advance(); // consume ]

                Ok(Expression::Array(elements))
            }
            Token::LeftBrace => {
                // Parse block or struct literal
                // Check if this is preceded by an identifier to determine if it's a struct literal
                // For now, we'll assume it's a block if not preceded by an identifier that looks like a struct name

                // We need to peek ahead to see if there's a struct name before the brace
                // Since we can't peek easily, we'll assume it's a block for now
                // In a more sophisticated parser, we'd track if the previous token was a struct name
                self.advance(); // consume {
                let mut statements = Vec::new();

                while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
                    statements.push(self.parse_statement()?);
                }

                if !matches!(self.current_token(), Token::RightBrace) {
                    return Err("Expected '}'".to_string());
                }
                self.advance(); // consume }

                Ok(Expression::Block(statements))
            }
            Token::Async => {
                // Parse async block: async { ... }
                self.advance(); // consume async

                if !matches!(self.current_token(), Token::LeftBrace) {
                    return Err("Expected '{'".to_string());
                }

                let block_statements = self.parse_block()?;
                Ok(Expression::AsyncBlock(block_statements))
            },
            Token::Await => {
                // Parse await expression: await expr
                self.advance(); // consume await
                let expr = self.parse_expression()?;
                Ok(Expression::Await(Box::new(expr)))
            },
            Token::At => {
                // Parse multi-language call with intelligent indexing and reverse DNS lookup
                self.advance(); // consume @

                // Check for special keywords like import or index
                match self.current_token().clone() {
                    Token::Identifier(first_ident) => {
                        if first_ident == "import" {
                            self.advance(); // consume "import"

                            // Parse @import("url" or "github_repo")
                            if !matches!(self.current_token(), Token::LeftParen) {
                                return Err("Expected '(' after @import".to_string());
                            }
                            self.advance(); // consume (

                            // Get the resource to import (URL or GitHub repo)
                            let resource = if let Token::String(resource_name) = self.current_token().clone() {
                                self.advance(); // consume resource name
                                resource_name
                            } else {
                                return Err("Expected resource string in @import".to_string());
                            };

                            if !matches!(self.current_token(), Token::RightParen) {
                                return Err("Expected ')' after @import arguments".to_string());
                            }
                            self.advance(); // consume )

                            // Determine if it's a URL or GitHub repo and process accordingly
                            let (lang, resource_type) = if resource.starts_with("https://github.com/") || resource.starts_with("http://github.com/") {
                                ("github", "repository")
                            } else if resource.starts_with("http://") || resource.starts_with("https://") {
                                // Attempt reverse DNS lookup to determine language
                                ("auto", "url") // Placeholder - in real implementation, determine from domain
                            } else {
                                ("local", "file") // Local file
                            };

                            Ok(Expression::MultiLangImport(lang.to_string(), resource, Some(resource_type.to_string())))
                        } else if first_ident == "index" {
                            self.advance(); // consume "index"

                            // Parse @index("resource")
                            if !matches!(self.current_token(), Token::LeftParen) {
                                return Err("Expected '(' after @index".to_string());
                            }
                            self.advance(); // consume (

                            // Get the resource to index
                            let resource = if let Token::String(resource_name) = self.current_token().clone() {
                                self.advance(); // consume resource name
                                resource_name
                            } else {
                                return Err("Expected resource string in @index".to_string());
                            };

                            if !matches!(self.current_token(), Token::RightParen) {
                                return Err("Expected ')' after @index arguments".to_string());
                            }
                            self.advance(); // consume )

                            Ok(Expression::MultiLangIndex("codex".to_string(), resource))
                        } else {
                            // Handle regular @lang{code} syntax with intelligent language detection
                            let lang = first_ident;
                            self.advance(); // consume language identifier

                            // Expect opening brace for code block
                            if !matches!(self.current_token(), Token::LeftBrace) {
                                return Err("Expected '{' after @lang".to_string());
                            }
                            self.advance(); // consume '{'

                            let mut brace_count = 1;
                            let mut embedded_code = String::new();

                            // Collect tokens until we find the matching closing brace
                            while brace_count > 0 && !matches!(self.current_token(), Token::Eof) {
                                match self.current_token() {
                                    Token::LeftBrace => {
                                        embedded_code.push('{');
                                        brace_count += 1;
                                        self.advance();
                                    },
                                    Token::RightBrace => {
                                        brace_count -= 1;
                                        if brace_count > 0 {
                                            embedded_code.push('}');
                                        }
                                        if brace_count == 0 {
                                            self.advance(); // consume the final closing brace
                                            break;
                                        } else {
                                            self.advance();
                                        }
                                    },
                                    Token::LeftParen => {
                                        embedded_code.push('(');
                                        self.advance();
                                    },
                                    Token::RightParen => {
                                        embedded_code.push(')');
                                        self.advance();
                                    },
                                    Token::LeftBracket => {
                                        embedded_code.push('[');
                                        self.advance();
                                    },
                                    Token::RightBracket => {
                                        embedded_code.push(']');
                                        self.advance();
                                    },
                                    Token::Identifier(s) => {
                                        embedded_code.push_str(s);
                                        self.advance();
                                    },
                                    Token::String(s) => {
                                        embedded_code.push('"');
                                        embedded_code.push_str(s);
                                        embedded_code.push('"');
                                        self.advance();
                                    },
                                    Token::Integer(i) => {
                                        embedded_code.push_str(&i.to_string());
                                        self.advance();
                                    },
                                    Token::Float(f) => {
                                        embedded_code.push_str(&f.to_string());
                                        self.advance();
                                    },
                                    Token::Boolean(b) => {
                                        embedded_code.push_str(if *b { "true" } else { "false" });
                                        self.advance();
                                    },
                                    Token::Plus => {
                                        embedded_code.push('+');
                                        self.advance();
                                    },
                                    Token::Minus => {
                                        embedded_code.push('-');
                                        self.advance();
                                    },
                                    Token::Multiply => {
                                        embedded_code.push('*');
                                        self.advance();
                                    },
                                    Token::Divide => {
                                        embedded_code.push('/');
                                        self.advance();
                                    },
                                    Token::Assign => {
                                        embedded_code.push('=');
                                        self.advance();
                                    },
                                    Token::Equal => {
                                        embedded_code.push_str("==");
                                        self.advance();
                                    },
                                    Token::NotEqual => {
                                        embedded_code.push_str("!=");
                                        self.advance();
                                    },
                                    Token::Less => {
                                        embedded_code.push('<');
                                        self.advance();
                                    },
                                    Token::Greater => {
                                        embedded_code.push('>');
                                        self.advance();
                                    },
                                    Token::And => {
                                        embedded_code.push_str("&&");
                                        self.advance();
                                    },
                                    Token::Or => {
                                        embedded_code.push_str("||");
                                        self.advance();
                                    },
                                    Token::Not => {
                                        embedded_code.push('!');
                                        self.advance();
                                    },
                                    Token::Comma => {
                                        embedded_code.push(',');
                                        self.advance();
                                    },
                                    Token::Semicolon => {
                                        embedded_code.push(';');
                                        self.advance();
                                    },
                                    Token::Colon => {
                                        embedded_code.push(':');
                                        self.advance();
                                    },
                                    Token::Dot => {
                                        embedded_code.push('.');
                                        self.advance();
                                    },
                                    Token::Arrow => {
                                        embedded_code.push_str("->");
                                        self.advance();
                                    },
                                    Token::FatArrow => {
                                        embedded_code.push_str("=>");
                                        self.advance();
                                    },
                                    Token::Range => {
                                        embedded_code.push_str("..");
                                        self.advance();
                                    },
                                    Token::PipeForward => {
                                        embedded_code.push_str("|>");
                                        self.advance();
                                    },
                                    Token::PipeBackward => {
                                        embedded_code.push_str("<|");
                                        self.advance();
                                    },
                                    Token::Spaceship => {
                                        embedded_code.push_str("<=>");
                                        self.advance();
                                    },
                                    Token::LeftArrow => {
                                        embedded_code.push_str("<-");
                                        self.advance();
                                    },
                                    Token::Ampersand => {
                                        embedded_code.push('&');
                                        self.advance();
                                    },
                                    Token::Pipe => {
                                        embedded_code.push('|');
                                        self.advance();
                                    },
                                    Token::Asterisk => {
                                        embedded_code.push('*');
                                        self.advance();
                                    },
                                    Token::Apostrophe => {
                                        embedded_code.push('\'');
                                        self.advance();
                                    },
                                    Token::At => {
                                        embedded_code.push('@');
                                        self.advance();
                                    },
                                    Token::Break => {
                                        embedded_code.push_str("break");
                                        self.advance();
                                    },
                                    Token::Continue => {
                                        embedded_code.push_str("continue");
                                        self.advance();
                                    },
                                    Token::Fn => {
                                        embedded_code.push_str("fn");
                                        self.advance();
                                    },
                                    Token::Let => {
                                        embedded_code.push_str("let");
                                        self.advance();
                                    },
                                    Token::Mut => {
                                        embedded_code.push_str("mut");
                                        self.advance();
                                    },
                                    Token::Const => {
                                        embedded_code.push_str("const");
                                        self.advance();
                                    },
                                    Token::If => {
                                        embedded_code.push_str("if");
                                        self.advance();
                                    },
                                    Token::Elif => {
                                        embedded_code.push_str("elif");
                                        self.advance();
                                    },
                                    Token::Else => {
                                        embedded_code.push_str("else");
                                        self.advance();
                                    },
                                    Token::While => {
                                        embedded_code.push_str("while");
                                        self.advance();
                                    },
                                    Token::For => {
                                        embedded_code.push_str("for");
                                        self.advance();
                                    },
                                    Token::In => {
                                        embedded_code.push_str("in");
                                        self.advance();
                                    },
                                    Token::Return => {
                                        embedded_code.push_str("return");
                                        self.advance();
                                    },
                                    Token::Match => {
                                        embedded_code.push_str("match");
                                        self.advance();
                                    },
                                    Token::Enum => {
                                        embedded_code.push_str("enum");
                                        self.advance();
                                    },
                                    Token::Struct => {
                                        embedded_code.push_str("struct");
                                        self.advance();
                                    },
                                    Token::Class => {
                                        embedded_code.push_str("class");
                                        self.advance();
                                    },
                                    Token::Trait => {
                                        embedded_code.push_str("trait");
                                        self.advance();
                                    },
                                    Token::Impl => {
                                        embedded_code.push_str("impl");
                                        self.advance();
                                    },
                                    Token::Pub => {
                                        embedded_code.push_str("pub");
                                        self.advance();
                                    },
                                    Token::True => {
                                        embedded_code.push_str("true");
                                        self.advance();
                                    },
                                    Token::False => {
                                        embedded_code.push_str("false");
                                        self.advance();
                                    },
                                    Token::Nil => {
                                        embedded_code.push_str("nil");
                                        self.advance();
                                    },
                                    Token::Async => {
                                        embedded_code.push_str("async");
                                        self.advance();
                                    },
                                    Token::Await => {
                                        embedded_code.push_str("await");
                                        self.advance();
                                    },
                                    Token::Try => {
                                        embedded_code.push_str("try");
                                        self.advance();
                                    },
                                    Token::Catch => {
                                        embedded_code.push_str("catch");
                                        self.advance();
                                    },
                                    Token::Finally => {
                                        embedded_code.push_str("finally");
                                        self.advance();
                                    },
                                    Token::Actor => {
                                        embedded_code.push_str("actor");
                                        self.advance();
                                    },
                                    Token::Spawn => {
                                        embedded_code.push_str("spawn");
                                        self.advance();
                                    },
                                    Token::Send => {
                                        embedded_code.push_str("send");
                                        self.advance();
                                    },
                                    Token::Receive => {
                                        embedded_code.push_str("receive");
                                        self.advance();
                                    },
                                    Token::Effect => {
                                        embedded_code.push_str("effect");
                                        self.advance();
                                    },
                                    Token::Perform => {
                                        embedded_code.push_str("perform");
                                        self.advance();
                                    },
                                    Token::With => {
                                        embedded_code.push_str("with");
                                        self.advance();
                                    },
                                    Token::Chan => {
                                        embedded_code.push_str("chan");
                                        self.advance();
                                    },
                                    Token::Close => {
                                        embedded_code.push_str("close");
                                        self.advance();
                                    },
                                    _ => {
                                        // For any other token, just advance to avoid infinite loop
                                        self.advance();
                                    }
                                }
                            }

                            Ok(Expression::MultiLangCall(lang, embedded_code.trim().to_string()))
                        }
                    },
                    Token::Import => {
                        self.advance(); // consume "import"

                        // Parse @import("url" or "github_repo")
                        if !matches!(self.current_token(), Token::LeftParen) {
                            return Err("Expected '(' after @import".to_string());
                        }
                        self.advance(); // consume (

                        // Get the resource to import (URL or GitHub repo)
                        let resource = if let Token::String(resource_name) = self.current_token().clone() {
                            self.advance(); // consume resource name
                            resource_name
                        } else {
                            return Err("Expected resource string in @import".to_string());
                        };

                        if !matches!(self.current_token(), Token::RightParen) {
                            return Err("Expected ')' after @import arguments".to_string());
                        }
                        self.advance(); // consume )

                        // Determine if it's a URL or GitHub repo and process accordingly
                        let (lang, resource_type) = if resource.starts_with("https://github.com/") || resource.starts_with("http://github.com/") {
                            ("github", "repository")
                        } else if resource.starts_with("http://") || resource.starts_with("https://") {
                            // Attempt reverse DNS lookup to determine language
                            ("auto", "url") // Placeholder - in real implementation, determine from domain
                        } else {
                            ("local", "file") // Local file
                        };

                        Ok(Expression::MultiLangImport(lang.to_string(), resource, Some(resource_type.to_string())))
                    },
                    Token::Index => {
                        self.advance(); // consume "index"

                        // Parse @index("resource")
                        if !matches!(self.current_token(), Token::LeftParen) {
                            return Err("Expected '(' after @index".to_string());
                        }
                        self.advance(); // consume (

                        // Get the resource to index
                        let resource = if let Token::String(resource_name) = self.current_token().clone() {
                            self.advance(); // consume resource name
                            resource_name
                        } else {
                            return Err("Expected resource string in @index".to_string());
                        };

                        if !matches!(self.current_token(), Token::RightParen) {
                            return Err("Expected ')' after @index arguments".to_string());
                        }
                        self.advance(); // consume )

                        Ok(Expression::MultiLangIndex("codex".to_string(), resource))
                    },
                    _ => {
                        Err("Expected identifier, import, or index after @".to_string())
                    }
                }
            }
            Token::Spawn => {
                // Parse spawn expression: spawn ActorName(args)
                self.advance(); // consume spawn

                if let Token::Identifier(actor_name) = self.current_token().clone() {
                    self.advance(); // consume actor name

                    if matches!(self.current_token(), Token::LeftParen) {
                        self.advance(); // consume (
                        let args = self.parse_arguments()?;
                        Ok(Expression::Spawn(actor_name, args))
                    } else {
                        Ok(Expression::Spawn(actor_name, vec![]))
                    }
                } else {
                    Err("Expected actor name after spawn".to_string())
                }
            },
            Token::Send => {
                // Parse send expression: send(actor, message)
                self.advance(); // consume send

                if !matches!(self.current_token(), Token::LeftParen) {
                    return Err("Expected '(' after send".to_string());
                }
                self.advance(); // consume (

                let actor = self.parse_expression()?;

                if !matches!(self.current_token(), Token::Comma) {
                    return Err("Expected ',' after actor in send".to_string());
                }
                self.advance(); // consume ,

                let message = self.parse_expression()?;

                if !matches!(self.current_token(), Token::RightParen) {
                    return Err("Expected ')' after send arguments".to_string());
                }
                self.advance(); // consume )

                Ok(Expression::Send(Box::new(actor), Box::new(message)))
            }
            Token::Chan => {
                // Parse channel creation: chan T
                self.advance(); // consume chan

                // Parse the type for the channel
                let channel_type = self.parse_type()?;
                Ok(Expression::ChannelCreate(Box::new(channel_type)))
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token())),
        }
    }

    /// Parses a tuple from a parenthesized expression: (expr, ...)
    /// 
    /// # Arguments
    /// * `first_element` - The first element of the tuple that was already parsed
    /// 
    /// # Returns
    /// A Result containing the Expression AST node if successful, or an error string
    fn parse_tuple_from_paren(&mut self, first_element: Expression) -> Result<Expression, String> {
        // This is called when we have (expr, ...) which indicates a tuple
        let mut elements = vec![first_element];

        // We already know the next token is a comma from the calling context
        self.advance(); // consume ,
        elements.push(self.parse_expression()?);

        while matches!(self.current_token(), Token::Comma) {
            self.advance(); // consume ,
            elements.push(self.parse_expression()?);
        }

        if !matches!(self.current_token(), Token::RightParen) {
            return Err("Expected ')'".to_string());
        }
        self.advance(); // consume )

        Ok(Expression::Tuple(elements))
    }

    /// Parses a variable declaration: let/mut name: Type = value
    ///
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        let mutable = matches!(self.current_token(), Token::Mut);
        self.advance(); // consume let or mut

        if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name

            // Check for lifetime annotation
            let lifetime_annotation = if matches!(self.current_token(), Token::Apostrophe) {
                self.advance(); // consume '
                if let Token::Identifier(lifetime) = self.current_token().clone() {
                    self.advance(); // consume lifetime name
                    Some(lifetime)
                } else {
                    return Err("Expected lifetime identifier after '".to_string());
                }
            } else {
                None
            };

            // Check for type annotation
            let type_annotation = if matches!(self.current_token(), Token::Colon) {
                self.advance(); // consume :
                Some(self.parse_type()?)
            } else {
                None
            };

            // Check for ownership modifier after type annotation
            let ownership_modifier = if matches!(self.current_token(), Token::Ampersand) {
                self.advance(); // consume &
                if matches!(self.current_token(), Token::Mut) {
                    self.advance(); // consume mut
                    Some(OwnershipModifier::MutablyBorrowed)
                } else {
                    Some(OwnershipModifier::Borrowed)
                }
            } else if matches!(self.current_token(), Token::Asterisk) {
                self.advance(); // consume *
                Some(OwnershipModifier::Shared)
            } else {
                None
            };

            if !matches!(self.current_token(), Token::Assign) {
                return Err("Expected '='".to_string());
            }
            self.advance(); // consume =

            let value = self.parse_expression()?;

            Ok(Statement::LetBinding {
                mutable,
                name,
                type_annotation,
                value,
                ownership_modifier,
                lifetime_annotation,
            })
        } else {
            Err("Expected identifier".to_string())
        }
    }

    /// Parses a constant declaration: const name: Type = value
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_const_declaration(&mut self) -> Result<Statement, String> {
        self.advance(); // consume const

        if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name

            // Check for type annotation
            let type_annotation = if matches!(self.current_token(), Token::Colon) {
                self.advance(); // consume :
                Some(self.parse_type()?)
            } else {
                None
            };

            if !matches!(self.current_token(), Token::Assign) {
                return Err("Expected '='".to_string());
            }
            self.advance(); // consume =

            let value = self.parse_expression()?;

            Ok(Statement::ConstBinding {
                name,
                type_annotation,
                value,
            })
        } else {
            Err("Expected identifier".to_string())
        }
    }

    /// Parses a function definition
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_function(&mut self) -> Result<Statement, String> {
        self.advance(); // consume fn

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected function name".to_string());
        };

        if !matches!(self.current_token(), Token::LeftParen) {
            return Err("Expected '('".to_string());
        }
        self.advance(); // consume (

        let parameters = self.parse_parameters()?;

        // Check for return type annotation
        let return_type = if matches!(self.current_token(), Token::Arrow) {
            self.advance(); // consume ->
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_async = false; // Simplified for now

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }

        let body = self.parse_block()?;

        Ok(Statement::Function(FunctionDef {
            name,
            parameters,
            return_type,
            body,
            is_async,
            is_public: false, // Default to private
            is_awaitable: false,  // Default to not awaitable
            effect_annotations: vec![], // Default to no effect annotations
        }))
    }

    /// Parses function parameters: (param1: Type, param2: Type, ...)
    /// 
    /// # Returns
    /// A Result containing a vector of Parameter AST nodes if successful, or an error string
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();

        if matches!(self.current_token(), Token::RightParen) {
            self.advance(); // consume )
            return Ok(params);
        }

        params.push(self.parse_parameter()?);

        while matches!(self.current_token(), Token::Comma) {
            self.advance(); // consume ,
            params.push(self.parse_parameter()?);
        }

        if !matches!(self.current_token(), Token::RightParen) {
            return Err("Expected ')'".to_string());
        }
        self.advance(); // consume )

        Ok(params)
    }

    /// Parses a single function parameter: name: Type
    /// 
    /// # Returns
    /// A Result containing the Parameter AST node if successful, or an error string
    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        // Check for ownership modifier before parameter name
        let ownership_modifier = match self.current_token() {
            Token::Ampersand => {
                self.advance(); // consume &
                if matches!(self.current_token(), Token::Mut) {
                    self.advance(); // consume mut
                    Some(OwnershipModifier::MutablyBorrowed)
                } else {
                    Some(OwnershipModifier::Borrowed)
                }
            },
            Token::Asterisk => {
                self.advance(); // consume *
                Some(OwnershipModifier::Shared) // Using * for shared ownership
            },
            _ => None,
        };

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected parameter name".to_string());
        };

        // Check for lifetime annotation after name
        let lifetime_annotation = if matches!(self.current_token(), Token::Apostrophe) {
            self.advance(); // consume '
            if let Token::Identifier(lifetime) = self.current_token().clone() {
                self.advance(); // consume lifetime name
                Some(lifetime)
            } else {
                return Err("Expected lifetime identifier after '".to_string());
            }
        } else {
            None
        };

        if !matches!(self.current_token(), Token::Colon) {
            return Err("Expected ':'".to_string());
        }
        self.advance(); // consume :

        let type_annotation = self.parse_type()?;

        // For now, no default values in this simplified parser
        Ok(Parameter {
            name,
            type_annotation,
            ownership_modifier,
            lifetime_annotation,
            default_value: None,
            mutability: None,  // Default to no explicit mutability
        })
    }

    /// Parses a type annotation: Int, String, [Int], etc.
    /// 
    /// # Returns
    /// A Result containing the Type AST node if successful, or an error string
    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current_token().clone() {
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "Int" => Ok(Type::Int),
                    "Float" => Ok(Type::Float),
                    "Bool" => Ok(Type::Bool),
                    "String" => Ok(Type::String),
                    "Unit" => Ok(Type::Unit),
                    _ => Ok(Type::Named(name)),
                }
            }
            Token::LeftBracket => {
                // Array type: [Int], [String], etc.
                self.advance(); // consume [
                let inner_type = self.parse_type()?;
                if !matches!(self.current_token(), Token::RightBracket) {
                    return Err("Expected ']'".to_string());
                }
                self.advance(); // consume ]
                Ok(Type::Array(Box::new(inner_type)))
            }
            _ => Err(format!("Expected type, got {:?}", self.current_token())),
        }
    }

    /// Parses a block of statements: { statement1; statement2; ... }
    /// 
    /// # Returns
    /// A Result containing a vector of Statement AST nodes if successful, or an error string
    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut statements = Vec::new();

        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(statements)
    }

    /// Parses a block statement: { statement1; statement2; ... }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_block_statement(&mut self) -> Result<Statement, String> {
        let statements = self.parse_block()?;
        Ok(Statement::Block(statements))
    }

    /// Parses an if statement: if condition { then_block } else { else_block }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume if

        let condition = self.parse_expression()?;

        let then_branch = self.parse_block()?;

        let else_branch = if matches!(self.current_token(), Token::Else) {
            self.advance(); // consume else
            self.parse_block()?
        } else {
            Vec::new()
        };

        Ok(Statement::Expression(Expression::If(
            Box::new(condition),
            then_branch,
            else_branch
        )))
    }

    /// Parses a match statement: match expr { pattern => block, ... }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_match_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // consume match

        let expr = self.parse_expression()?;

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut arms = Vec::new();

        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            let pattern = self.parse_pattern()?;

            // Check for guard condition (optional)
            let guard = if matches!(self.current_token(), Token::If) {
                self.advance(); // consume 'if'
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            if !matches!(self.current_token(), Token::FatArrow) {
                return Err("Expected '=>'".to_string());
            }
            self.advance(); // consume =>

            let body = self.parse_block()?;

            arms.push((pattern, guard, body));
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Expression(Expression::Match(
            Box::new(expr),
            arms
        )))
    }

    /// Parses a pattern for match expressions
    /// 
    /// # Returns
    /// A Result containing the Pattern AST node if successful, or an error string
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        self.parse_or_pattern()
    }

    /// Parses OR patterns (pattern1 | pattern2)
    /// 
    /// # Returns
    /// A Result containing the Pattern AST node if successful, or an error string
    fn parse_or_pattern(&mut self) -> Result<Pattern, String> {
        let mut left = self.parse_basic_pattern()?;

        // Look for | (or) patterns
        while matches!(self.current_token(), Token::Pipe) {
            self.advance(); // consume |
            let right = self.parse_basic_pattern()?;
            left = Pattern::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// Parses basic patterns (identifiers, literals, tuples, arrays, structs)
    /// 
    /// # Returns
    /// A Result containing the Pattern AST node if successful, or an error string
    fn parse_basic_pattern(&mut self) -> Result<Pattern, String> {
        match self.current_token().clone() {
            Token::Identifier(name) => {
                self.advance();
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else {
                    // Check if this is a struct pattern: Name { field: pattern, ... }
                    if matches!(self.current_token(), Token::LeftBrace) {
                        // This is a struct pattern
                        self.advance(); // consume {
                        let mut fields = Vec::new();

                        if !matches!(self.current_token(), Token::RightBrace) {
                            let field_name = if let Token::Identifier(fname) = self.current_token().clone() {
                                self.advance(); // consume field name
                                fname
                            } else {
                                return Err("Expected field name".to_string());
                            };

                            if !matches!(self.current_token(), Token::Colon) {
                                return Err("Expected ':'".to_string());
                            }
                            self.advance(); // consume :

                            let field_pattern = self.parse_or_pattern()?; // Use or pattern for nested or
                            fields.push((field_name, field_pattern));

                            while matches!(self.current_token(), Token::Comma) {
                                self.advance(); // consume ,
                                let field_name = if let Token::Identifier(fname) = self.current_token().clone() {
                                    self.advance(); // consume field name
                                    fname
                                } else {
                                    return Err("Expected field name".to_string());
                                };

                                if !matches!(self.current_token(), Token::Colon) {
                                    return Err("Expected ':'".to_string());
                                }
                                self.advance(); // consume :

                                let field_pattern = self.parse_or_pattern()?; // Use or pattern for nested or
                                fields.push((field_name, field_pattern));
                            }
                        }

                        if !matches!(self.current_token(), Token::RightBrace) {
                            return Err("Expected '}'".to_string());
                        }
                        self.advance(); // consume }

                        Ok(Pattern::Struct(name, fields))
                    } else {
                        Ok(Pattern::Identifier(name))
                    }
                }
            }
            Token::Integer(value) => {
                self.advance();
                Ok(Pattern::Literal(Expression::Integer(value)))
            }
            Token::String(value) => {
                self.advance();
                Ok(Pattern::Literal(Expression::String(value)))
            }
            Token::True => {
                self.advance();
                Ok(Pattern::Literal(Expression::Boolean(true)))
            }
            Token::False => {
                self.advance();
                Ok(Pattern::Literal(Expression::Boolean(false)))
            }
            Token::LeftParen => {
                // Tuple pattern
                self.advance(); // consume (
                let mut patterns = Vec::new();

                if !matches!(self.current_token(), Token::RightParen) {
                    patterns.push(self.parse_or_pattern()?); // Use or pattern for nested or

                    while matches!(self.current_token(), Token::Comma) {
                        self.advance(); // consume ,
                        patterns.push(self.parse_or_pattern()?); // Use or pattern for nested or
                    }
                }

                if !matches!(self.current_token(), Token::RightParen) {
                    return Err("Expected ')'".to_string());
                }
                self.advance(); // consume )

                Ok(Pattern::Tuple(patterns))
            }
            Token::LeftBracket => {
                // Array pattern
                self.advance(); // consume [
                let mut patterns = Vec::new();

                if !matches!(self.current_token(), Token::RightBracket) {
                    patterns.push(self.parse_or_pattern()?); // Use or pattern for nested or

                    while matches!(self.current_token(), Token::Comma) {
                        self.advance(); // consume ,
                        patterns.push(self.parse_or_pattern()?); // Use or pattern for nested or
                    }
                }

                if !matches!(self.current_token(), Token::RightBracket) {
                    return Err("Expected ']'".to_string());
                }
                self.advance(); // consume ]

                Ok(Pattern::Array(patterns))
            }
            _ => Err(format!("Unexpected pattern token: {:?}", self.current_token())),
        }
    }

    /// Parses an actor definition: actor Name { state_fields, handler_functions }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_actor(&mut self) -> Result<Statement, String> {
        self.advance(); // consume actor

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected actor name".to_string());
        };

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut state = Vec::new();
        let mut handlers = Vec::new();

        // Parse actor state (field declarations)
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            if matches!(self.current_token(), Token::Fn) {
                // Parse handler function
                handlers.push(self.parse_function_def()?);
            } else {
                // Parse state field
                let field_name = if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance(); // consume field name
                    name
                } else {
                    return Err("Expected field name".to_string());
                };

                if !matches!(self.current_token(), Token::Colon) {
                    return Err("Expected ':'".to_string());
                }
                self.advance(); // consume :

                let field_type = self.parse_type()?;

                state.push((field_name, field_type));

                if !matches!(self.current_token(), Token::Semicolon) {
                    return Err("Expected ';'".to_string());
                }
                self.advance(); // consume ;
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Actor(ActorDef {
            name,
            state,
            handlers,
        }))
    }

    /// Parses a function definition (used internally for actor handlers)
    /// 
    /// # Returns
    /// A Result containing the FunctionDef AST node if successful, or an error string
    fn parse_function_def(&mut self) -> Result<FunctionDef, String> {
        self.advance(); // consume fn

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected function name".to_string());
        };

        if !matches!(self.current_token(), Token::LeftParen) {
            return Err("Expected '('".to_string());
        }
        self.advance(); // consume (

        let parameters = self.parse_parameters()?;

        // Check for return type annotation
        let return_type = if matches!(self.current_token(), Token::Arrow) {
            self.advance(); // consume ->
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_async = false; // Simplified for now

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }

        let body = self.parse_block()?;

        Ok(FunctionDef {
            name,
            parameters,
            return_type,
            body,
            is_async,
            is_public: false, // Default to private
            is_awaitable: false,  // Default to not awaitable
            effect_annotations: vec![], // Default to no effect annotations
        })
    }

    /// Parses an effect definition: effect Name { operations }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_effect(&mut self) -> Result<Statement, String> {
        self.advance(); // consume effect

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected effect name".to_string());
        };

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut operations = Vec::new();

        // Parse effect operations (function declarations)
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            if matches!(self.current_token(), Token::Fn) {
                operations.push(self.parse_function_def()?);
            } else {
                return Err("Expected function declaration in effect".to_string());
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Effect(EffectDef {
            name,
            operations,
        }))
    }

    /// Parses a class definition: class Name { fields, methods }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_class(&mut self) -> Result<Statement, String> {
        self.advance(); // consume class

        // Check for access modifier
        let access_modifier = if matches!(self.current_token(), Token::Pub) {
            self.advance(); // consume pub
            AccessModifier::Public
        } else {
            AccessModifier::Private  // Default to private
        };

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected class name".to_string());
        };

        // Parse generic parameters if present
        let generics = if matches!(self.current_token(), Token::Less) {
            self.parse_generics()?
        } else {
            Vec::new()
        };

        // Check for inheritance
        let parent = if matches!(self.current_token(), Token::Colon) {
            self.advance(); // consume :
            if let Token::Identifier(parent_name) = self.current_token().clone() {
                self.advance(); // consume parent name
                Some(parent_name)
            } else {
                return Err("Expected parent class name".to_string());
            }
        } else {
            None
        };

        // Check for interface implementation
        let interfaces = if matches!(self.current_token(), Token::Impl) {
            self.advance(); // consume impl
            self.parse_interface_list()?
        } else {
            Vec::new()
        };

        // Set default values for new fields
        let is_abstract = false;  // Default to not abstract

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructors = Vec::new();
        let mut destructors = Vec::new();

        // Parse class members
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            if matches!(self.current_token(), Token::Fn) {
                // Parse method
                let method = self.parse_function_def()?;

                // Check if it's a constructor or destructor
                if method.name == "new" || method.name == "__init__" {
                    constructors.push(ConstructorDef {
                        parameters: method.parameters,
                        body: method.body,
                        access_modifier: AccessModifier::Public, // Constructors are usually public
                    });
                } else if method.name == "drop" || method.name == "__del__" {
                    destructors.push(DestructorDef {
                        body: method.body,
                        access_modifier: AccessModifier::Public, // Destructors are usually public
                    });
                } else {
                    methods.push(method);
                }
            } else if matches!(self.current_token(), Token::Let) || matches!(self.current_token(), Token::Mut) {
                // Parse field
                let field = self.parse_field_def()?;
                fields.push(field);
            } else {
                return Err(format!("Unexpected token in class: {:?}", self.current_token()));
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Class(ClassDef {
            name,
            fields,
            methods,
            parent,
            access_modifier,
            is_abstract,
            generics,
            interfaces,
            constructors,
            destructors,
        }))
    }

    /// Parses a field definition within a class
    /// 
    /// # Returns
    /// A Result containing the FieldDef AST node if successful, or an error string
    fn parse_field_def(&mut self) -> Result<FieldDef, String> {
        // Check for access modifier
        let access_modifier = if matches!(self.current_token(), Token::Pub) {
            self.advance(); // consume pub
            AccessModifier::Public
        } else {
            AccessModifier::Private  // Default to private
        };

        // Check for mutability
        let is_mutable = matches!(self.current_token(), Token::Mut);
        if is_mutable {
            self.advance(); // consume mut
        }

        // Set default value for static (since we removed the Static token)
        let is_static = false;  // Default to non-static

        let field_name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume field name
            name
        } else {
            return Err("Expected field name".to_string());
        };

        if !matches!(self.current_token(), Token::Colon) {
            return Err("Expected ':'".to_string());
        }
        self.advance(); // consume :

        let type_annotation = self.parse_type()?;

        // Check for default value
        let default_value = if matches!(self.current_token(), Token::Assign) {
            self.advance(); // consume =
            Some(self.parse_expression()?)
        } else {
            None
        };

        if !matches!(self.current_token(), Token::Semicolon) {
            return Err("Expected ';'".to_string());
        }
        self.advance(); // consume ;

        Ok(FieldDef {
            name: field_name,
            type_annotation,
            access_modifier,
            is_mutable,
            is_static,
            default_value,
        })
    }

    /// Parses generic type parameters: <T, U, V>
    /// 
    /// # Returns
    /// A Result containing a vector of generic parameter names if successful, or an error string
    fn parse_generics(&mut self) -> Result<Vec<String>, String> {
        if !matches!(self.current_token(), Token::Less) {
            return Ok(Vec::new());
        }
        self.advance(); // consume <

        let mut generics = Vec::new();
        loop {
            if let Token::Identifier(name) = self.current_token().clone() {
                generics.push(name);
                self.advance(); // consume identifier
            } else {
                return Err("Expected generic parameter name".to_string());
            }

            if matches!(self.current_token(), Token::Comma) {
                self.advance(); // consume ,
            } else {
                break;
            }
        }

        if !matches!(self.current_token(), Token::Greater) {
            return Err("Expected '>'".to_string());
        }
        self.advance(); // consume >

        Ok(generics)
    }

    /// Parses a list of interfaces: Interface1, Interface2, ...
    /// 
    /// # Returns
    /// A Result containing a vector of interface names if successful, or an error string
    fn parse_interface_list(&mut self) -> Result<Vec<String>, String> {
        let mut interfaces = Vec::new();
        loop {
            if let Token::Identifier(name) = self.current_token().clone() {
                interfaces.push(name);
                self.advance(); // consume interface name
            } else {
                return Err("Expected interface name".to_string());
            }

            if matches!(self.current_token(), Token::Comma) {
                self.advance(); // consume ,
            } else {
                break;
            }
        }

        Ok(interfaces)
    }

    /// Parses a trait definition: trait Name { method_signatures }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_trait(&mut self) -> Result<Statement, String> {
        self.advance(); // consume trait

        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected trait name".to_string());
        };

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut methods = Vec::new();

        // Parse trait methods (signatures only)
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            if matches!(self.current_token(), Token::Fn) {
                // Parse method signature (without implementation)
                let method_def = self.parse_function_def()?;
                methods.push(method_def);
            } else {
                return Err("Expected function declaration in trait".to_string());
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Trait(TraitDef {
            name,
            type_params: Vec::new(),
            methods,
            associated_types: Vec::new(),
            super_traits: Vec::new(),
        }))
    }

    /// Parses an implementation block: impl TraitName for TypeName { method_implementations }
    /// 
    /// # Returns
    /// A Result containing the Statement AST node if successful, or an error string
    fn parse_impl(&mut self) -> Result<Statement, String> {
        self.advance(); // consume impl

        let trait_name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume trait name
            name
        } else {
            return Err("Expected trait name".to_string());
        };

        if !matches!(self.current_token(), Token::For) {
            return Err("Expected 'for'".to_string());
        }
        self.advance(); // consume for

        let for_type = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume type name
            name
        } else {
            return Err("Expected type name after 'for'".to_string());
        };

        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{'".to_string());
        }
        self.advance(); // consume {

        let mut methods = Vec::new();

        // Parse implementation methods
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            if matches!(self.current_token(), Token::Fn) {
                // Parse method implementation
                let method_def = self.parse_function_def()?;
                methods.push(method_def);
            } else {
                return Err("Expected function declaration in impl block".to_string());
            }
        }

        if !matches!(self.current_token(), Token::RightBrace) {
            return Err("Expected '}'".to_string());
        }
        self.advance(); // consume }

        Ok(Statement::Implementation(ImplDef {
            trait_name,
            for_type,
            type_params: Vec::new(),
            methods,
            associated_types: Vec::new(),
        }))
    }

    /// Parses a macro definition: macro name { body }
    fn parse_macro(&mut self) -> Result<Statement, String> {
        self.advance(); // consume 'macro'

        // Expect macro name
        let name = if let Token::Identifier(name) = self.current_token().clone() {
            self.advance(); // consume name
            name
        } else {
            return Err("Expected macro name".to_string());
        };

        // Initialize parameters vector
        let mut parameters = Vec::new();

        // Expect opening brace for parameters (if any)
        if matches!(self.current_token(), Token::LeftParen) {
            self.advance(); // consume '('

            if !matches!(self.current_token(), Token::RightParen) {
                loop {
                    if let Token::Identifier(param_name) = self.current_token().clone() {
                        parameters.push(param_name);
                        self.advance(); // consume parameter name

                        if matches!(self.current_token(), Token::Comma) {
                            self.advance(); // consume comma
                            continue;
                        }
                    } else {
                        return Err("Expected parameter name in macro definition".to_string());
                    }

                    if matches!(self.current_token(), Token::RightParen) {
                        break;
                    }
                }
            }

            self.advance(); // consume ')'
        } // else: parameters is already an empty vector

        // Expect opening brace for macro body
        if !matches!(self.current_token(), Token::LeftBrace) {
            return Err("Expected '{' in macro definition".to_string());
        }
        self.advance(); // consume '{'

        // Parse macro body statements
        let mut body = Vec::new();
        while !matches!(self.current_token(), Token::RightBrace) && !matches!(self.current_token(), Token::Eof) {
            body.push(self.parse_statement()?);
        }

        if matches!(self.current_token(), Token::RightBrace) {
            self.advance(); // consume '}'
        } else {
            return Err("Expected '}' to close macro definition".to_string());
        }

        Ok(Statement::MacroDefinition(MacroDef {
            name,
            parameters,
            body,
            is_hygienic: true, // Default to hygienic macros
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_program() {
        let input = r#"
        fn main() {
            let x = 42
            print(x)
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse_program();

        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1); // One function

        match &program.statements[0] {
            Statement::Function(func) => {
                assert_eq!(func.name, "main");
                assert_eq!(func.body.len(), 2); // let x = 42 and print(x)
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_variable_declaration() {
        let input = "let x: Int = 42";

        let mut parser = Parser::new(input);
        let result = parser.parse_statement();

        assert!(result.is_ok());

        match result.unwrap() {
            Statement::LetBinding { name, type_annotation, value, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(type_annotation.unwrap(), Type::Int));
                match value {
                    Expression::Integer(42) => {},
                    _ => panic!("Expected Integer(42)"),
                }
            },
            _ => panic!("Expected LetBinding"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let input = "print(\"Hello, World!\")";

        let mut parser = Parser::new(input);
        let result = parser.parse_expression();

        assert!(result.is_ok());

        match result.unwrap() {
            Expression::Call(name, args) => {
                assert_eq!(name, "print");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expression::String(s) => assert_eq!(s, "Hello, World!"),
                    _ => panic!("Expected string argument"),
                }
            },
            _ => panic!("Expected function call"),
        }
    }
}