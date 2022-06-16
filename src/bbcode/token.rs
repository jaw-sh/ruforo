/// A single Token output by the tokenizer.
#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Null,
    Linebreak(&'a str),
    Tag(&'a str, &'a str, Option<&'a str>),
    TagClose(&'a str, &'a str),
    Text(&'a str),
    Url(&'a str),
}

impl<'a> Token<'a> {
    pub fn as_raw(&'a self) -> &'a str {
        match self {
            Self::Null => "",
            Self::Linebreak(raw) => raw,
            Self::Tag(raw, _, _) => raw,
            Self::TagClose(raw, _) => raw,
            Self::Text(text) => text,
            Self::Url(url) => url,
        }
    }

    pub fn is_empty(self) -> bool {
        match self {
            Self::Null => true,
            Self::Linebreak(_) => false,
            Self::Tag(_, tag, arg) => tag.is_empty() && arg.is_none(),
            Self::TagClose(_, tag) => tag.is_empty(),
            Self::Text(text) => text.is_empty(),
            Self::Url(url) => url.is_empty(),
        }
    }
}

impl<'a> Default for Token<'a> {
    fn default() -> Self {
        Token::Null
    }
}

mod tests {
    #[test]
    fn reverse_null() {
        use super::Token;

        let inst = Token::Null;
        assert_eq!(inst.as_raw(), "");
    }

    #[test]
    fn reverse_linebreak() {
        use super::Token;

        let inst = Token::Linebreak("\n\r");
        assert_eq!(inst.as_raw(), "\n\r");

        let inst = Token::Linebreak("\n\r");
        assert_eq!(inst.as_raw(), "\n\r");
    }

    #[test]
    fn reverse_tag() {
        use super::Token;

        let inst = Token::Tag("[foo]", "foo", None);
        assert_eq!(inst.as_raw(), "[foo]");

        let inst = Token::Tag(
            "[url=https://zombo.com/]",
            "url",
            Some("=https://zombo.com/"),
        );
        assert_eq!(inst.as_raw(), "[url=https://zombo.com/]");
    }

    #[test]
    fn reverse_tag_close() {
        use super::Token;

        let inst = Token::TagClose("[/foo]", "foo");
        assert_eq!(inst.as_raw(), "[/foo]");
    }

    #[test]
    fn reverse_text() {
        use super::Token;

        let text = "text input :)";
        let inst = Token::Text(text);
        assert_eq!(inst.as_raw(), text);
    }
}
