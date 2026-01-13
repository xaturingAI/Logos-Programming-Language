fn main() {
    use crate::lexer::{Lexer, Token};

    let input = "@python{print(\"hello\");}";
    let mut lexer = Lexer::new(input);

    println!("Input: {}", input);
    println!("Tokens:");
    loop {
        let token = lexer.next_token();
        println!("  {:?}", token);
        if matches!(token, Token::Eof) {
            break;
        }
    }
}