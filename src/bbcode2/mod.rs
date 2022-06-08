mod instruction;
mod read_mode;
mod tag;
mod tokenizer;

pub use instruction::Instruction;
pub use read_mode::ReadMode;
pub use tokenizer::Tokenizer;

/// Generates a string of HTML from an &str of BbCode.
#[no_mangle]
pub fn bbcode_to_html2(input: &str) {
    let mut tokenizer = Tokenizer::new();
    let tokenized = tokenizer.tokenize(input);
    //let mut lexer = Lexer::new(true);
    //let dom = lexer.lex();
    //HTMLConstructor {
    //    output_string: String::with_capacity(input.len() + input.len() / 2),
    //    pretty_print: false,
    //}
    //.construct(dom)
}
