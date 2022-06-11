mod constructor;
mod element;
mod lexer;
mod parser;
mod read_mode;
mod tag;
mod token;

pub use constructor::Constructor;
pub use element::{Element, ElementDisplay};
pub use lexer::Lexer;
pub use parser::Parser;
pub use read_mode::ReadMode;
pub use tag::Tag;
pub use token::Token;

/// Generates a string of HTML from an &str of BbCode.
#[no_mangle]
pub fn parse(input: &str) -> String {
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(input);

    let mut parser = Parser::new();
    let ast = parser.parse(tokens);

    let constructor = Constructor::new();
    constructor.build(&ast)
}

mod tests {
    #[test]
    fn inline_tags() {
        use super::parse;

        assert_eq!("<b>Test</b>", parse("[b]Test[/b]"));
        assert_eq!("<i>Test</i>", parse("[i]Test[/i]"));
        assert_eq!("<u>Test</u>", parse("[u]Test[/u]"));
        assert_eq!("<s>Test</s>", parse("[s]Test[/s]"));

        assert_eq!("<b><i>Test</i></b>", parse("[b][i]Test[/i][/b]"));
        assert_eq!("<b><i>Test</i></b>", parse("[b][i]Test[/i]"));
        assert_eq!("<b><i>Test</i></b>", parse("[b][i]Test[/b]"));
        assert_eq!("<b><i>Test</i></b>", parse("[b][i]Test"));
    }

    #[test]
    fn linebreaks() {
        use super::parse;

        assert_eq!("Foo<br />bar", parse("Foo\n\rbar"));
        assert_eq!("Foo<br />bar", parse("Foo\r\nbar"));
        assert_eq!("Foo<br />bar", parse("Foo\r\n\rbar"));
        assert_eq!("Foo<br />bar", parse("Foo\nbar"));

        assert_eq!("Foo<br /><br /><br />bar", parse("Foo\n\n\nbar"));
        assert_eq!(
            "<b>Foo<br /><br /><br />bar</b>",
            parse("[b]Foo\n\n\nbar[/b]")
        );
        assert_eq!("<b>Foo<br /><br /><br />bar</b>", parse("[b]Foo\n\n\nbar"));
    }

    #[test]
    fn misc() {
        use super::parse;

        assert_eq!("<hr />", parse("[hr]"));
        assert_eq!("<hr />", parse("[hr][/hr]"));
        assert_eq!("Foo<hr />Bar", parse("Foo[hr]Bar"));
        assert_eq!("Foo<hr />Bar", parse("Foo[hr]Bar[/hr]"));
        assert_eq!("Foo<hr />Bar", parse("Foo[hr][/hr]Bar"));
        assert_eq!("<b>Foo<hr />Bar</b>", parse("[b]Foo[hr]Bar"));
    }
}
