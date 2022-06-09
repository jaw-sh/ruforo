mod element;
mod lexer;
mod parser;
mod read_mode;
mod token;

pub use element::{Element, ElementDisplay};
pub use lexer::Lexer;
pub use parser::Parser;
pub use read_mode::ReadMode;
pub use token::Token;

/// Generates a string of HTML from an &str of BbCode.
#[no_mangle]
pub fn bbcode_to_html2(input: &str) {
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(input);

    //let mut lexer = Lexer::new(true);
    //HTMLConstructor {
    //    output_string: String::with_capacity(input.len() + input.len() / 2),
    //    pretty_print: false,
    //}
    //.construct(dom)
}
