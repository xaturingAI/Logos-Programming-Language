use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    // Keywords
    Fn, Let, Mut, Const, If, Elif, Else, While, For, In,
    Return, Match, Enum, Struct, Class, Trait, Impl, Pub,
    True, False, Nil, Async, Await, Try, Catch, Finally,
    Actor, Spawn, Send, Receive, Effect, Perform, With, Chan, Close,

    // Operators
    Plus, Minus, Multiply, Divide, Modulo, Power,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    And, Or, Not, Assign, PlusAssign, MinusAssign,
    PipeForward, PipeBackward, Spaceship,
    Range,  // For .. operator
    LeftArrow,  // For channel operations: <-

    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Dot, Colon,
    Semicolon, Arrow, FatArrow, Underscore,
    // Additional operators
    Pipe, Ampersand, Asterisk, Apostrophe,  // Fixed typo: Apos't'rophe

    // Special
    At,
    Break,
    Continue,
    Eof,
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    current_char: Option<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.peek().copied();

        Self {
            input,
            chars,
            current_char,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            tokens.push(token.clone());

            if matches!(token, Token::Eof) {
                break;
            }
        }

        tokens
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.current_char {
            match ch {
                // Multi-character tokens: advance in match
                '=' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to second '='
                        self.advance();  // Move past both '=' characters
                        Token::Equal
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        self.advance();  // Move past both '=' and '>'
                        Token::FatArrow
                    } else {
                        self.advance();  // Move past single '='
                        Token::Assign
                    }
                },
                '!' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        self.advance();  // Move past both '!' and '='
                        Token::NotEqual
                    } else {
                        self.advance();  // Move past single '!'
                        Token::Not
                    }
                },
                '<' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        self.advance();  // Move past both '<' and '='
                        Token::LessEqual
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        self.advance();  // Move past both '<' and '>'
                        Token::Spaceship
                    } else if self.peek_char() == Some('-') {
                        self.advance();  // Move to '-'
                        self.advance();  // Move past both '<' and '-'
                        Token::LeftArrow
                    } else {
                        self.advance();  // Move past single '<'
                        Token::Less
                    }
                },
                '>' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        self.advance();  // Move past both '>' and '='
                        Token::GreaterEqual
                    } else {
                        self.advance();  // Move past single '>'
                        Token::Greater
                    }
                },
                '&' => {
                    if self.peek_char() == Some('&') {
                        self.advance();  // Move to second '&'
                        self.advance();  // Move past both '&'
                        Token::And       // This is for &&
                    } else {
                        self.advance();  // Move past single '&'
                        Token::Ampersand // This is for single &
                    }
                },
                '|' => {
                    if self.peek_char() == Some('|') {
                        self.advance();  // Move to second '|'
                        self.advance();  // Move past both '|'
                        Token::Or        // This is for ||
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        self.advance();  // Move past both '|' and '>'
                        Token::PipeForward // This is for |> 
                    } else {
                        self.advance();  // Move past single '|'
                        Token::Pipe      // This is for single |
                    }
                },
                '+' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        self.advance();  // Move past both '+' and '='
                        Token::PlusAssign
                    } else {
                        self.advance();  // Move past single '+'
                        Token::Plus
                    }
                },
                '-' => {
                    if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        self.advance();  // Move past both '-' and '>'
                        Token::Arrow
                    } else if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        self.advance();  // Move past both '-' and '='
                        Token::MinusAssign
                    } else {
                        self.advance();  // Move past single '-'
                        Token::Minus
                    }
                },
                // Single character tokens
                '(' => { self.advance(); Token::LeftParen },
                ')' => { self.advance(); Token::RightParen },
                '{' => { self.advance(); Token::LeftBrace },
                '}' => { self.advance(); Token::RightBrace },
                '[' => { self.advance(); Token::LeftBracket },
                ']' => { self.advance(); Token::RightBracket },
                ',' => { self.advance(); Token::Comma },
                '.' => {
                    if self.peek_char() == Some('.') {
                        self.advance();  // Move to second '.'
                        self.advance();  // Move past both '.'
                        Token::Range
                    } else {
                        self.advance();  // Move past single '.'
                        Token::Dot
                    }
                },
                ':' => { self.advance(); Token::Colon },
                ';' => { self.advance(); Token::Semicolon },
                '_' => { self.advance(); Token::Underscore },
                '*' => { self.advance(); Token::Multiply },
                '/' => {
                    if self.peek_char() == Some('/') {
                        // Handle single-line comment: skip until newline
                        self.advance(); // consume the first '/'
                        self.advance(); // consume the second '/'
                        while let Some(ch) = self.current_char {
                            if ch == '\n' {
                                break; // stop at newline, don't consume it
                            }
                            self.advance();
                        }
                        // After skipping comment, get the next token
                        return self.next_token();
                    } else {
                        self.advance();  // Move past single '/'
                        Token::Divide
                    }
                },
                '%' => { self.advance(); Token::Modulo },
                '^' => { self.advance(); Token::Power },
                '@' => {
                    return self.read_multilang_call();
                },

                // Tokens that return early (they handle their own advancement)
                '"' => return self.read_string(),
                '\'' => return self.read_string(),

                // Other characters
                _ => {
                    if ch.is_ascii_digit() {
                        return self.read_number();
                    } else if ch.is_alphabetic() || ch == '_' {
                        return self.read_identifier_or_keyword();
                    } else {
                        // Skip unknown character
                        self.advance();
                        return self.next_token();
                    }
                }
            }
        } else {
            Token::Eof
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.position += 1;
        self.current_char = self.chars.next();
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut number_str = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' {
                // Check if the next character after this dot is also a dot (range operator "..")
                if self.peek_char() == Some('.') {
                    // This is a range operator "..", so we stop here and don't consume the second dot
                    // The second dot will be processed as the next token
                    break;
                } else {
                    // This is a decimal point in a float, add it and continue
                    number_str.push(ch);
                    self.advance();
                }
            } else {
                break;
            }
        }

        // At this point, we have collected the number string
        // If it contains a '.', it should be parsed as a float
        // But we need to be careful about cases like "3.14." which would be invalid
        // If the string ends with a dot and we have other digits, that's an incomplete float
        if number_str.ends_with('.') && number_str.len() > 1 {
            // Remove the trailing dot - this means we had a number like "3." which should be "3" (integer)
            // and the dot should be parsed separately
            number_str.pop();
            if number_str.is_empty() {
                // This shouldn't happen if numbers start with digits
                return Token::Dot;
            }
        }

        if number_str.contains('.') {
            if let Ok(value) = number_str.parse::<f64>() {
                Token::Float(value)
            } else {
                panic!("Invalid float literal: {}", number_str);
            }
        } else {
            if let Ok(value) = number_str.parse::<i64>() {
                Token::Integer(value)
            } else {
                panic!("Invalid integer literal: {}", number_str);
            }
        }
    }

    fn read_string(&mut self) -> Token {
        let quote = self.current_char.unwrap(); // Store the opening quote
        self.advance(); // Skip opening quote

        let mut string_content = String::new();

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance(); // Skip backslash
                if let Some(escaped_char) = self.current_char {
                    match escaped_char {
                        'n' => string_content.push('\n'),
                        't' => string_content.push('\t'),
                        'r' => string_content.push('\r'),
                        '\\' => string_content.push('\\'),
                        '"' => string_content.push('"'),
                        '\'' => string_content.push('\''),
                        _ => string_content.push(escaped_char),
                    }
                    self.advance();
                }
            } else {
                string_content.push(ch);
                self.advance();
            }
        }

        Token::String(string_content)
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match identifier.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "const" => Token::Const,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "return" => Token::Return,
            "match" => Token::Match,
            "enum" => Token::Enum,
            "struct" => Token::Struct,
            "class" => Token::Class,
            "trait" => Token::Trait,
            "impl" => Token::Impl,
            "pub" => Token::Pub,
            "true" => Token::True,
            "false" => Token::False,
            "nil" => Token::Nil,
            "async" => Token::Async,
            "await" => Token::Await,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "actor" => Token::Actor,
            "spawn" => Token::Spawn,
            "send" => Token::Send,
            "receive" => Token::Receive,
            "effect" => Token::Effect,
            "perform" => Token::Perform,
            "with" => Token::With,
            "chan" => Token::Chan,
            "close" => Token::Close,
            "break" => Token::Break,
            "continue" => Token::Continue,
            _ => Token::Identifier(identifier),
        }
    }


    fn read_multilang_call(&mut self) -> Token {
        // The @ symbol is already the current character, so advance past it
        self.advance(); // Skip '@'
        
        // Skip whitespace after @
        self.skip_whitespace();
        
        // Read the language identifier (alphanumeric + underscore)
        let mut lang_identifier = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                lang_identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Skip whitespace after language identifier
        self.skip_whitespace();
        
        // Expect opening brace
        if self.current_char == Some('{') {
            self.advance(); // Skip '{'
            
            // For now, just return the At token since the parser will handle the multilang call
            // The actual parsing of the language and embedded code will happen in the parser
            return Token::At;
        } else {
            // If there's no opening brace, treat as regular identifier
            return Token::Identifier(format!("@{}", lang_identifier));
        }
    }
}

fn main() {
    let input = "42";
    let mut lexer = Lexer::new(input);
    
    println!("Input: '{}'", input);
    let tokens = lexer.tokenize();
    for token in tokens {
        println!("{:?}", token);
    }
}