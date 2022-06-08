extern crate phf;
extern crate rctree;

pub mod ast;
pub mod html_constructor;
pub mod lexer;
pub mod tag;
pub mod tokenizer;

#[cfg(test)]
mod tests;

pub use self::html_constructor::HTMLConstructor;
pub use self::lexer::Lexer;
pub use self::tokenizer::Tokenizer;

/// Generates a string of HTML from an &str of BbCode.
/// This function produces *pretty* output, meaning that any eroneously written BbCode encountered or empty tags will be removed from the final output.
/// # Examples
///
/// ```
///use ruforo::bbcode::bbcode_to_html;
///
///assert_eq!(bbcode_to_html("I'm [i]italic[/i] and [b]bold![/b]"),
///		"<p>I&#x27m <i>italic</i> and <b>bold!</b></p>");
///
///assert_eq!(bbcode_to_html("[quote][/quote]"),
///		"");
/// ```
#[no_mangle]
pub fn bbcode_to_html(input: &str) -> String {
    let mut tokenizer = Tokenizer::new();
    let mut lexer = Lexer::new(true);

    let dom = lexer.lex(tokenizer.tokenize(input));

    HTMLConstructor {
        output_string: String::with_capacity(input.len() + input.len() / 2),
        pretty_print: false,
    }
    .construct(dom)
}

/// Types of argument for Instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Colour(String),
    Url(String),
    Quote(String),
}

/// A single Instruction output by the tokenizer.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Null,
    Tag(String, Option<String>),
    Text(String),
    Parabreak(String),
    Linebreak,
}
impl Default for Instruction {
    fn default() -> Self {
        Instruction::Null
    }
}
