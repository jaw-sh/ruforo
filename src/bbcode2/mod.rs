extern crate linkify;

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
    let ast = parser.parse(&tokens);

    let constructor = Constructor::new();
    constructor.build(ast)
}

mod tests {
    #[test]
    fn img() {
        use super::parse;

        assert_eq!("[img]", parse("[img]"));
        assert_eq!("[img][/img]", parse("[img][/img]"));
        assert_eq!(
            "<img src=\"https://zombo.com/images/zombocom.png\" />",
            parse("[img]https://zombo.com/images/zombocom.png[/img]")
        );
        assert_eq!(
            "<img src=\"https://zombo.com/images/zombocom.png\" />",
            parse("[img]https://zombo.com/images/zombocom.png")
        );
    }

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

        const GOOD_COLORS: &[&str] = &["red", "#ff0000"];
        const BAD_COLORS: &[&str] = &["RED", "ff0000", "sneed", ""];

        for good in GOOD_COLORS {
            assert_eq!(
                format!(
                    "<span class=\"bbCode tagColor\" style=\"color: {}\">Hello!</span>",
                    good
                ),
                parse(&format!("[color={}]Hello![/color]", good))
            );
        }

        for bad in BAD_COLORS {
            assert_eq!(
                format!("[color={}]Hello![/color]", bad),
                parse(&format!("[color={}]Hello![/color]", bad))
            );
        }
    }

    #[test]
    fn international_text() {
        use super::parse;

        assert_eq!(
            "I&#x27;d bet it&#x27;s a &quot;test&quot;, yea.",
            parse("I'd bet it's a \"test\", yea.")
        );
        assert_eq!("私は猫<i>です</i>。", parse("私は猫[i]です[/i]。"));
        assert_eq!(
            "全世界無產階級和被壓迫的民族聯合起來！",
            parse("全世界無產階級和被壓迫的民族聯合起來！")
        );
        assert_eq!(
            "<b>СМЕРТЬ</b><br />ВСІМ, ХТО НА ПИРИШКОДІ<br />ДОБУТЬЯ ВІЛЬНОСТІ<br />ТРУДОВОМУ ЛЮДУ.",
            parse(
                "[b]СМЕРТЬ[/b]\n\rВСІМ, ХТО НА ПИРИШКОДІ\n\rДОБУТЬЯ ВІЛЬНОСТІ\n\rТРУДОВОМУ ЛЮДУ."
            )
        );
        assert_eq!("😂🔫", parse("😂🔫"));
    }

    #[test]
    fn invalid() {
        use super::parse;

        assert_eq!("[foo]Test[/foo]", parse("[foo]Test[/foo]"));
        assert_eq!("[foo]Test[/foo]", parse("[plain][foo]Test[/foo][/plain]"));
        assert_eq!("[foo]Test[/bar]", parse("[foo]Test[/bar]"));
        assert_eq!("[foo]Test", parse("[foo]Test"));
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
    fn linkify() {
        use super::parse;

        assert_eq!(
            "Welcome, to <a href=\"https://zombo.com/\" rel=\"nofollow\">https://zombo.com/</a>!",
            parse("Welcome, to https://zombo.com/!")
        );
        assert_eq!(
            "Welcome, to <a href=\"https://zombo.com/\" rel=\"nofollow\">https://zombo.com/</a>!",
            parse("Welcome, to [url]https://zombo.com/[/url]!")
        );
    }

    #[test]
    fn misc() {
        use super::parse;

        // This is a self-closing tag in HTML and I disagree that it should require a closing tag in BBCode.
        assert_eq!("<hr />", parse("[hr]"));
        //assert_eq!("<hr />", parse("[hr][/hr]"));
        assert_eq!("Foo<hr />Bar", parse("Foo[hr]Bar"));
        //assert_eq!("Foo<hr />Bar", parse("Foo[hr]Bar[/hr]"));
        //assert_eq!("Foo<hr />Bar", parse("Foo[hr][/hr]Bar"));
        assert_eq!("<b>Foo<hr />Bar</b>", parse("[b]Foo[hr]Bar"));
    }

    #[test]
    fn plain() {
        use super::parse;

        assert_eq!("[b]Test[/b]", parse("[plain][b]Test[/b][/plain]"));
        assert_eq!("[b]Test[/b]", parse("[plain][b]Test[/b]"));
        assert_eq!("[b]Foo[hr]bar[/b]", parse("[plain][b]Foo[hr]bar[/b]"));
    }

    #[test]
    fn pre() {
        use super::parse;

        assert_eq!("<pre>Test</pre>", parse("[code]Test[/code]"));
        assert_eq!("<pre>Foo\n\rbar</pre>", parse("[code]Foo\n\rbar[/code]"));
    }
}
