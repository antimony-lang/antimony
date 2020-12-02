mod lexer;
mod parser;

fn main() {
    let tokens = lexer::tokenize(&"let x = 2");
    // let ast = parser::parse(tokens.into_iter());

    println!("{:?}", tokens)
}
