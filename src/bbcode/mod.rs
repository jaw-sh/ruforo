extern crate linkify;

mod constructor;
mod element;
mod parser;
mod tag;
mod token;
mod tokenize;

pub use constructor::Constructor;
pub use element::{Element, ElementDisplay};
pub use parser::Parser;
pub use tag::Tag;
pub use token::Token;
pub use tokenize::tokenize;

/// Generates a string of HTML from an &str of BbCode.
#[no_mangle]
pub fn parse(input: &str) -> String {
    let tokens: Vec<Token> = tokenize(input).expect("Failed to unwrap tokens.").1;

    //println!("TOKENS: {:?}", tokens);

    let mut parser = Parser::new();
    let ast = parser.parse(&tokens);

    //for node in ast.descendants() {
    //    println!("{:?}", node);
    //}

    let constructor = Constructor::new();
    constructor.build(ast)
}

#[cfg(test)]
mod tests {
    #[test]
    fn img() {
        use super::parse;

        assert_eq!(
            "<img src=\"https://zombo.com/images/zombocom.png\" />",
            parse("[img]https://zombo.com/images/zombocom.png[/img]")
        );
        assert_eq!(
            "<img src=\"https://zombo.com/images/zombocom.png\" />",
            parse("[img]https://zombo.com/images/zombocom.png")
        );
        assert_eq!("[img][/img]", parse("[img][/img]"));
        assert_eq!("[img]", parse("[img]"));
        assert_eq!("[img]not a link[/img]", parse("[img]not a link[/img]"));
        assert_eq!("[img]not a link", parse("[img]not a link"));
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
                "[b]СМЕРТЬ[/b]\r\nВСІМ, ХТО НА ПИРИШКОДІ\r\nДОБУТЬЯ ВІЛЬНОСТІ\r\nТРУДОВОМУ ЛЮДУ."
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

        assert_eq!("Foo<br />bar", parse("Foo\r\nbar"));
        assert_eq!("Foo<br />bar", parse("Foo\nbar"));
        assert_eq!("Foo<br />\rbar", parse("Foo\n\rbar"));
        assert_eq!("Foo<br />\rbar", parse("Foo\r\n\rbar"));

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
            "Welcome, to <a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"https://zombo.com/\">https://zombo.com/</a>",
            parse("Welcome, to https://zombo.com/")
        );
        assert_eq!(
            "Welcome, to <a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"https://zombo.com/\">https://zombo.com/</a>!",
            parse("Welcome, to [url]https://zombo.com/[/url]!")
        );
        assert_eq!(
            "Welcome, to <b><a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"https://zombo.com/\">https://zombo.com/</a></b>!",
            parse("Welcome, to [b][url]https://zombo.com/[/url][/b]!")
        );
        assert_eq!(
            "Welcome, to <a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"https://zombo.com/\">Zombo.com</a>!",
            parse("Welcome, to [url=https://zombo.com/]Zombo.com[/url]!")
        );
        assert_eq!(
            "<a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"https://zombo.com/\"><img src=\"https://zombo.com/images/zombocom.png\" /></a>",
            parse("[url=https://zombo.com/][img]https://zombo.com/images/zombocom.png[/img][/url]")
        );
        assert_eq!(
            "Welcome, to [url][/url]!",
            parse("Welcome, to [url][/url]!")
        );
        assert_eq!("Welcome, to [url]!", parse("Welcome, to [url]!"));
        assert_eq!("[url][/url]", parse("[url][/url]"));
        assert_eq!("[url]", parse("[url]"));
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
        assert_eq!("<pre>Foo\r\nbar</pre>", parse("[code]Foo\r\nbar[/code]"));
    }

    #[test]
    fn sanitize() {
        use super::parse;

        assert_eq!("&lt;b&gt;Test&lt;/b&gt;", parse("<b>Test</b>"));
    }
}
