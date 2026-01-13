// Logos Programming Language Lexer
// This module provides tokenization functionality for the Logos programming language.
// It converts source code text into a sequence of tokens that can be processed by the parser.

use std::iter::Peekable;
use std::str::Chars;

/// Represents a single token in the Logos programming language
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literal tokens - values that represent data
    Identifier(String),  // Variable, function, or type names
    Integer(i64),        // Integer numbers (e.g., 42, -10)
    Float(f64),          // Floating-point numbers (e.g., 3.14, -2.5)
    String(String),      // String literals (e.g., "hello", 'world')
    Char(char),          // Character literals (e.g., 'a', '\n')
    Boolean(bool),       // Boolean values (true, false)

    // Keyword tokens - reserved words with special meaning
    Fn, Let, Mut, Const, If, Elif, Else, While, For, In,
    Return, Match, Enum, Struct, Class, Trait, Impl, Pub, Type,
    True, False, Nil, Async, Await, Try, Catch, Finally,
    Actor, Spawn, Send, Receive, Effect, Perform, With, Chan, Close,
    Abstract, Private, Protected, Static, Implements,
    Macro,

    // Operator tokens - symbols that perform operations
    Plus, Minus, Multiply, Divide, Modulo, Power,           // Arithmetic operators
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual, // Comparison operators
    And, Or, Not, Assign, PlusAssign, MinusAssign,          // Logical and assignment operators
    PipeForward, PipeBackward, Spaceship,                   // Special operators
    Range,                                                  // Range operator: ..
    LeftArrow,                                              // Channel operator: <-

    // Delimiter tokens - symbols that separate or group code
    LeftParen, RightParen, LeftBrace, RightBrace,    // Parentheses and braces: ( ) { }
    LeftBracket, RightBracket,                        // Brackets: [ ]
    Comma, Dot, Colon,                               // Punctuation: , . :
    Semicolon, Arrow, FatArrow, Underscore,          // Other delimiters: ; -> => _
    
    // Additional operator tokens
    Pipe, Ampersand, Asterisk, Apostrophe,          // | & * '

    // Ownership and mutability operators
    Ref,                    // Reference operator: &
    MutRef,                 // Mutable reference: &mut
    Move,                   // Move operator: move (keyword or prefix)
    Transfer,               // Transfer ownership: transfer (keyword)

    // Special tokens
    At,        // @ symbol for multi-language calls
    Import,    // import keyword
    Index,     // index keyword
    Break,     // break keyword
    Continue,  // continue keyword
    Eof,       // End of file marker
    LessThan,  // '<' character
    GreaterThan, // '>' character
}

/// The Lexer struct processes source code and converts it into tokens
pub struct Lexer<'a> {
    input: &'a str,              // The source code to tokenize
    chars: Peekable<Chars<'a>>,  // Iterator over characters with peek capability
    current_char: Option<char>,  // The current character being processed
    position: usize,             // Current position in the input string
    line: usize,                 // Current line number (for error reporting)
    column: usize,               // Current column number (for error reporting)
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer instance for the given input string
    /// 
    /// # Arguments
    /// * `input` - The source code string to tokenize
    /// 
    /// # Returns
    /// A new Lexer instance ready to tokenize the input
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        // Get and consume the first character to process
        let current_char = chars.next();

        Self {
            input,
            chars,           // Remaining characters after the first
            current_char,    // The first character to process
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenizes the entire input string into a vector of tokens
    /// 
    /// # Returns
    /// A vector containing all tokens from the input string
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

    /// Gets the next token from the input stream
    /// 
    /// # Returns
    /// The next token in the input stream
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(ch) = self.current_char {
            let token = match ch {
                // Multi-character tokens: advance in match, main function advances again
                '=' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to second '='
                        Token::Equal     // Main advance() will move past second '=', so both chars consumed
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        Token::FatArrow  // Main advance() will move past '>', so both chars consumed
                    } else {
                        Token::Assign    // Main advance() will move past single '=', so one char consumed
                    }
                },
                '!' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        Token::NotEqual  // Main advance() will move past '=', so both chars consumed
                    } else {
                        Token::Not       // Main advance() will move past single '!', so one char consumed
                    }
                },
                '<' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        Token::LessEqual // Main advance() will move past '=', so both chars consumed
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        Token::Spaceship // Main advance() will move past '>', so both chars consumed
                    } else if self.peek_char() == Some('-') {
                        self.advance();  // Move to '-'
                        Token::LeftArrow // Main advance() will move past '-', so both chars consumed
                    } else {
                        Token::Less      // Main advance() will move past single '<', so one char consumed
                    }
                },
                '>' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        Token::GreaterEqual // Main advance() will move past '=', so both chars consumed
                    } else {
                        Token::Greater   // Main advance() will move past single '>', so one char consumed
                    }
                },
                '&' => {
                    if self.peek_char() == Some('&') {
                        self.advance();  // Move to second '&'
                        Token::And       // Main advance() will move past second '&', so both chars consumed
                    } else {
                        Token::Ampersand // Main advance() will move past single '&', so one char consumed
                    }
                },
                '|' => {
                    if self.peek_char() == Some('|') {
                        self.advance();  // Move to second '|'
                        Token::Or        // Main advance() will move past second '|', so both chars consumed
                    } else if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        Token::PipeForward // Main advance() will move past '>', so both chars consumed
                    } else {
                        Token::Pipe      // Main advance() will move past single '|', so one char consumed
                    }
                },
                '+' => {
                    if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        Token::PlusAssign // Main advance() will move past '=', so both chars consumed
                    } else {
                        Token::Plus      // Main advance() will move past single '+', so one char consumed
                    }
                },
                '-' => {
                    if self.peek_char() == Some('>') {
                        self.advance();  // Move to '>'
                        Token::Arrow     // Main advance() will move past '>', so both chars consumed
                    } else if self.peek_char() == Some('=') {
                        self.advance();  // Move to '='
                        Token::MinusAssign // Main advance() will move past '=', so both chars consumed
                    } else {
                        Token::Minus     // Main advance() will move past single '-', so one char consumed
                    }
                },
                // Single character tokens
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                ',' => Token::Comma,
                '.' => {
                    if self.peek_char() == Some('.') {
                        self.advance();  // Move to second '.'
                        Token::Range     // Main advance() will move past second '.', so both chars consumed
                    } else {
                        Token::Dot       // Main advance() will move past single '.', so one char consumed
                    }
                },
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                '_' => Token::Underscore,
                '*' => Token::Multiply,
                '/' => {
                    if self.peek_char() == Some('/') {
                        // Handle single-line comment: skip until newline
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
                        Token::Divide
                    }
                },
                '%' => Token::Modulo,
                '^' => Token::Power,

                // Tokens that return early (they handle their own advancement)
                '"' => return self.read_string(),
                '\'' => return self.read_char(),

                '@' => {
                    self.advance();
                    return self.read_multilang_call();
                },

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
            };

            // Advance past the current token (for tokens that didn't return early)
            self.advance();
            token
        } else {
            Token::Eof
        }
    }

    /// Advances the lexer to the next character in the input stream
    /// Updates position, line, and column counters accordingly
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
        // Get the next character from the stream
        self.current_char = self.chars.next();
    }

    /// Peeks at the next character without consuming it
    /// 
    /// # Returns
    /// The next character in the input stream, or None if at end
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Skips whitespace characters in the input stream
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Reads a number token (integer or float) from the input stream
    /// 
    /// # Returns
    /// A Token::Integer or Token::Float containing the parsed number
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

    /// Reads a string token from the input stream
    /// Handles escape sequences within strings
    /// 
    /// # Returns
    /// A Token::String containing the parsed string content
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

    /// Reads a character token from the input stream
    /// Handles escape sequences within character literals
    ///
    /// # Returns
    /// A Token::Char containing the parsed character content
    fn read_char(&mut self) -> Token {
        self.advance(); // Skip opening quote

        let mut char_content = String::new();

        while let Some(ch) = self.current_char {
            if ch == '\'' {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance(); // Skip backslash
                if let Some(escaped_char) = self.current_char {
                    match escaped_char {
                        'n' => char_content.push('\n'),
                        't' => char_content.push('\t'),
                        'r' => char_content.push('\r'),
                        '\\' => char_content.push('\\'),
                        '\'' => char_content.push('\''),
                        '"' => char_content.push('"'),
                        _ => char_content.push(escaped_char),
                    }
                    self.advance();
                }
            } else {
                char_content.push(ch);
                self.advance();
            }
        }

        // Extract the first character from the string (since a char literal should only contain one character)
        if let Some(single_char) = char_content.chars().next() {
            Token::Char(single_char)
        } else {
            // If no character was found, return a default character (though this shouldn't happen in valid code)
            Token::Char('\0')
        }
    }

    /// Reads an identifier or keyword token from the input stream
    /// Determines if the identifier is a reserved keyword or a user-defined name
    /// 
    /// # Returns
    /// A Token::Identifier if it's a user-defined name, or the appropriate keyword token
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
            "type" => Token::Type,
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
            "import" => Token::Import,
            "index" => Token::Index,
            "abstract" => Token::Abstract,
            "private" => Token::Private,
            "protected" => Token::Protected,
            "static" => Token::Static,
            "implements" => Token::Implements,
            "macro" => Token::Macro,
            _ => Token::Identifier(identifier),
        }
    }

    /// Reads a multi-language call token (indicated by @ symbol)
    /// 
    /// # Returns
    /// A Token::At indicating the start of a multi-language call
    fn read_multilang_call(&mut self) -> Token {
        // The @ symbol was already consumed by the main function, so we just return Token::At
        // and let the parser handle the language identifier that follows.
        // The parser will see @ followed by an identifier (like python, rust, etc.) and handle accordingly.
        Token::At
    }
}

/// Public function to tokenize input source code
/// 
/// # Arguments
/// * `input` - The source code string to tokenize
/// 
/// # Returns
/// * `Ok(Vec<Token>)` containing the tokenized result if successful
/// * `Err` with error details if tokenization failed
pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_tokens() {
        let input = "=+-*/%(){}[];,.!";
        let mut lexer = Lexer::new(input);

        let tokens = vec![
            Token::Assign,      // "=" should be Assign, not Equal
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::Modulo,
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
            Token::LeftBracket,
            Token::RightBracket,
            Token::Semicolon,
            Token::Comma,
            Token::Dot,
            Token::Not,
            Token::Eof,
        ];

        for expected in tokens {
            let actual = lexer.next_token();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_identifiers_and_keywords() {
        let input = "let if else true false fn main";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::False);
        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::Identifier("main".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_numbers() {
        let input = "42 3.14 0";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Integer(42));
        assert_eq!(lexer.next_token(), Token::Float(3.14));
        assert_eq!(lexer.next_token(), Token::Integer(0));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_strings() {
        let input = r#""hello" 'world'"#;
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
        assert_eq!(lexer.next_token(), Token::String("world".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_multilang_calls() {
        let input = "@rust{let x = 42;}";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::At);  // The @ symbol should be recognized
        assert_eq!(lexer.next_token(), Token::Let); // The 'let' keyword after @rust{
        // The rest continues as normal tokenization
    }

    #[test]
    fn test_valid_multilang_call() {
        // Test a proper multilang call: @language{code}
        let input = "@python{print(\"hello\");}";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::At);      // @ symbol indicating multilang call
        // The rest of the tokens depend on how the embedded code is tokenized
        // This is what matters for the multilang functionality
    }
}