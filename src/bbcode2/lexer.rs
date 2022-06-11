use super::ReadMode;
use super::Token;
use url::Url;

/// Struct for BbCode tokenization.
#[derive(Default)]
pub struct Lexer {
    mode: ReadMode,
    current_token: Token,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads and tokenizes BbCode into individual Tokens.
    pub fn tokenize(&mut self, bbcode: &str) -> &Vec<Token> {
        for character in bbcode.chars() {
            self.parse(character);
        }

        self.commit_token();
        &self.tokens
    }

    /// Adds `current_token` to `tokens` and resets `current_token`.
    fn commit_token(&mut self) {
        match self.current_token {
            Token::Null => {}
            Token::Url(ref url) => match Url::parse(url) {
                Ok(_) => {
                    self.tokens.push(self.current_token.clone());
                }
                Err(_) => {
                    if url.len() > 0 {
                        self.tokens.push(Token::Text(url.to_owned()));
                    }
                }
            },
            _ => {
                self.tokens.push(self.current_token.clone());
            }
        }

        self.current_token = Token::Null;
    }

    /// Inserts an token directly into `tokens` and resets `current_token`.
    fn insert_token(&mut self, token: Token) {
        self.tokens.push(token);
        self.current_token = Token::Null;
    }

    #[inline]
    fn parse(&mut self, character: char) {
        match self.mode {
            ReadMode::Linebreak => {
                self.parse_linebreak(character);
            }
            ReadMode::Tag => {
                self.parse_tag(character);
            }
            ReadMode::TagClose => {
                self.parse_tag_close(character);
            }
            ReadMode::TagArg => {
                self.parse_tag_arg(character, false);
            }
            ReadMode::TagArgQuote => {
                self.parse_tag_arg(character, true);
            }
            ReadMode::Text => {
                self.parse_text(character);
            }
            ReadMode::Url(explicit) => {
                self.parse_url(character, explicit);
            }
        }
    }

    /// Intreprets char as plain text input, expecting new tokens.
    fn parse_text(&mut self, character: char) {
        match character {
            //'\\' => {
            //    self.mode = ReadMode::Escape;
            //}
            '[' => {
                self.commit_token();
                self.mode = ReadMode::Tag;
                self.current_token = Token::empty_tag();
            }
            '\r' => {}
            '\n' => {
                self.commit_token();
                self.mode = ReadMode::Linebreak;
            }
            '>' | '<' | '&' | '"' | '\'' => {
                match self.current_token {
                    Token::Text(ref mut contents) => {
                        contents.push_str(&Self::sanitize(character));
                    }
                    _ => {
                        self.current_token = Token::Text(Self::sanitize(character).to_string());
                    }
                }

                if character == '<' {
                    self.commit_token();
                    self.mode = ReadMode::Url(true);
                }
            }
            _ => match self.current_token {
                Token::Text(ref mut contents) => {
                    contents.push(character);
                }
                _ => {
                    self.current_token = Token::Text(character.to_string());
                }
            },
        }
    }

    /// Parses new lines and discards whitespace until next token.
    fn parse_linebreak(&mut self, character: char) {
        match character {
            // Consume tabs.
            '\t' => {}
            // Consume carriage returns.
            // New lines may be \n or \n\r but they are never \r.
            // https://en.wikipedia.org/wiki/Carriage_return
            '\r' => {}
            // Consume whitespace.
            ' ' => {}
            // Unexpected character; finish breaking and return to text parser
            _ => {
                self.insert_token(Token::Linebreak);
                self.mode = ReadMode::Text;
                self.parse_text(character);
            }
        }
    }

    fn parse_escape(&mut self, character: char) {
        self.mode = ReadMode::Text;
        match character {
            '>' | '<' | '&' | '"' | '\'' | '\\' => match self.current_token {
                Token::Tag(ref mut contents, _) => {
                    contents.push_str(&Self::sanitize(character));
                }
                _ => {
                    self.current_token = Token::Text(Self::sanitize(character));
                }
            },
            _ => match self.current_token {
                Token::Text(ref mut contents) => {
                    contents.push(character);
                }
                _ => {
                    self.current_token = Token::Text(character.to_string());
                }
            },
        }
    }

    fn parse_tag(&mut self, character: char) {
        match character {
            // End the tag.
            ']' => {
                self.commit_token();
                self.mode = ReadMode::Text;
            }
            // Move to closing tag instruciton.
            '/' => {
                // If we've just opened, we can proceed to a closing tag.
                if self.current_token.is_empty() {
                    self.mode = ReadMode::TagClose;
                    self.current_token = Token::empty_tag_close();
                }
                // If we've already started our tag, choke and reset.
                else {
                    self.reset_parse_to_text(character);
                }
            }
            // Hints we should move to arguments
            ' ' | '=' => {
                // Begin adding to the arg string, if we have a tag.
                if !self.current_token.is_empty() {
                    match self.current_token {
                        Token::Tag(ref tag, _) => {
                            self.current_token =
                                Token::Tag(tag.to_owned(), Some(character.to_string()));
                            self.mode = ReadMode::TagArg;
                        }
                        _ => unreachable!(),
                    }
                }
                // If we don't have a tag name yet, we choke.
                else {
                    self.reset_parse_to_text(character);
                }
            }
            // Intolerable break; choke and kill the tag.
            '\n' | '\r' => {
                self.reset_parse_to_text(character);
                return;
            }
            // Add letters
            _ => match self.current_token {
                Token::Tag(ref mut contents, _) => {
                    contents.push(character);
                }
                _ => {
                    self.current_token = Token::Tag(character.to_string(), None);
                }
            },
        }
    }

    /// Parse arguments in a tag.
    /// Arguments are any text after the tag name, before the ].
    fn parse_tag_arg(&mut self, character: char, literal: bool) {
        // If the character should be added to the arg string.
        match character {
            // Close tag if we're not being literal.
            ']' => {
                if !literal {
                    self.commit_token();
                    self.mode = ReadMode::Text;
                    return;
                }
            }
            // Break tag if we're not being literal.
            '[' => {
                if !literal {
                    self.reset_parse_to_text(character);
                    return;
                }
            }
            // Toggle literal reading
            '"' => {
                self.mode = match literal {
                    true => ReadMode::TagArg,
                    false => ReadMode::TagArgQuote,
                };
            }
            // Intolerable break; choke and kill the tag.
            '\n' | '\r' => {
                self.reset_parse_to_text(character);
                return;
            }
            // Append any other character to our arg string.
            _ => {}
        };

        match self.current_token {
            Token::Tag(ref contents, ref mut args) => match args {
                // Add to the Some(string)
                Some(ref mut args) => {
                    args.push(character);
                }
                // Change token to include an arg string.
                None => {
                    self.current_token =
                        Token::Tag(contents.to_string(), Some(character.to_string()));
                }
            },
            _ => {
                unreachable!();
            }
        };
    }

    fn parse_tag_close(&mut self, character: char) {
        match character {
            // close tag
            ']' => {
                self.commit_token();
                self.mode = ReadMode::Text;
            }
            _ => {
                // if a-Z, commit as tag name
                if character.is_ascii_alphabetic() {
                    match self.current_token {
                        Token::TagClose(ref mut contents) => {
                            contents.push(character);
                        }
                        _ => self.current_token = Token::TagClose(character.to_string()),
                    }
                }
                // otherwise, we have a broken closing tag
                else {
                    self.reset_parse_to_text(character);
                }
            }
        }
    }

    /// Accepts a character expecting to build a URL.
    /// `explicit` is set when the URL is to be encapsulated in an <> like email.
    fn parse_url(&mut self, character: char, explicit: bool) {
        match character {
            // Explicit terminators.
            '\n' => {
                self.commit_token();
                self.mode = ReadMode::Text;
                self.parse_linebreak(character);
                return;
            }
            '>' => {
                self.commit_token();
                self.mode = ReadMode::Text;
                self.parse_text(character);
                return;
            }
            // Non-explicit terminators.
            ' ' => {
                if !explicit {
                    self.commit_token();
                    self.mode = ReadMode::Text;
                    self.parse_text(character);
                    return;
                }
            }
            _ => {}
        }

        match self.current_token {
            Token::Url(ref mut url) => {
                url.push(character);
            }
            _ => {
                self.current_token = Token::Url(character.to_string());
            }
        }
    }

    /// Aborts the current ReadMode to Text and converts current token to Text.
    /// Supplied char is what choked the parser.
    fn reset_parse_to_text(&mut self, character: char) {
        // Recover existing input.
        let text: String = match &self.current_token {
            Token::Text(content) => {
                log::warn!("Resetting text parse back to text. Should not occur.");
                content.to_string()
            }
            Token::Tag(tag, arg) => match arg {
                Some(arg) => format!("[{}{}", tag, arg),
                None => format!("[{}", tag),
            },
            Token::TagClose(tag) => format!("[/{}", tag),
            _ => self.current_token.to_inner_string(),
        };

        self.mode = ReadMode::Text;
        self.current_token = Token::Text(text);
        self.parse_text(character);
    }

    /// Sanitizes a char for HTML.
    fn sanitize(character: char) -> String {
        match character {
            '<' => "&lt;",
            '>' => "&gt;",
            '&' => "&amp;",
            '"' => "&quot;",
            '\'' => "&#x27;",
            '\\' => "&#x2F;",
            _ => unreachable!(),
        }
        .to_owned()
    }
}

mod tests {
    #[test]
    fn linebreak() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("a\n\rb\n\r\r\r\rc\r");

        assert_eq!(t.tokens.len(), 5);

        match &t.tokens[0] {
            Token::Text(text) => assert_eq!("a", text),
            _ => assert!(false, "1st token was not text."),
        }
        assert!(t.tokens[1] == Token::Linebreak, "2nd token not linebreak.");
        match &t.tokens[4] {
            Token::Text(text) => assert_eq!("c", text),
            _ => assert!(false, "5th token was not text."),
        }
    }

    #[test]
    fn sanitize() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("<strong>HTML</strong>");

        let mut output = String::with_capacity(64);

        for token in &t.tokens {
            match token {
                Token::Text(ref text) => output.push_str(text),
                _ => unreachable!(),
            }
        }

        assert_eq!("&lt;strong&gt;HTML&lt;/strong&gt;", output);
    }

    #[test]
    fn tag_and_close() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("[b]Bold[/b]");

        assert_eq!(t.tokens.len(), 3);

        match &t.tokens[0] {
            Token::Tag(tag, arg) => {
                assert_eq!("b", tag);
                assert_eq!(&None, arg);
            }
            _ => assert!(false, "1st token was not a tag."),
        }
        match &t.tokens[1] {
            Token::Text(text) => assert_eq!("Bold", text),
            _ => assert!(false, "2nd token was not text."),
        }
        match &t.tokens[2] {
            Token::TagClose(tag) => {
                assert_eq!("b", tag);
            }
            _ => assert!(false, "3rd token was not a closing tag."),
        }
    }

    #[test]
    fn tag_close_terminates() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("[b]Bold[//b]");

        assert_eq!(t.tokens.len(), 3);

        match &t.tokens[2] {
            Token::Text(text) => {
                assert_eq!("[//b]", text);
            }
            _ => assert!(false, "3rd token was not text."),
        }
    }

    #[test]
    fn tag_open_terminates() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("[b]Bold[b/b]");

        assert_eq!(t.tokens.len(), 3);

        match &t.tokens[2] {
            Token::Text(text) => {
                assert_eq!("[b/b]", text);
            }
            _ => assert!(false, "3rd token was not text."),
        }
    }

    #[test]
    fn tag_with_arg() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("[url=https://zombo.com]ZOMBO[/url]");

        assert_eq!(t.tokens.len(), 3);

        match &t.tokens[0] {
            Token::Tag(tag, arg) => {
                assert_eq!("url", tag);
                assert_eq!(&Some("=https://zombo.com".to_string()), arg);
            }
            _ => assert!(false, "1st token was not a tag."),
        }
        match &t.tokens[1] {
            Token::Text(text) => assert_eq!("ZOMBO", text),
            _ => assert!(false, "2nd token was not text."),
        }
        match &t.tokens[2] {
            Token::TagClose(tag) => {
                assert_eq!("url", tag);
            }
            _ => assert!(false, "3rd token was not a closing tag."),
        }
    }

    #[test]
    fn tag_with_strange_args() {
        use super::{Lexer, Token};

        // This content can be parsed as correct because the Lexer does not care
        // about the validity of the arguments.
        const GIBBERISH: &str = "   👍 wow nice \"[test]\"";
        let mut t = Lexer::new();
        t.tokenize(&format!("[url{}]Text[/url]", GIBBERISH));

        assert_eq!(t.tokens.len(), 3);
        match &t.tokens[0] {
            Token::Tag(tag, arg) => {
                assert_eq!("url", tag);
                assert_eq!(&Some(GIBBERISH.to_string()), arg);
            }
            _ => assert!(false, "1st token was not a tag."),
        }
        match &t.tokens[1] {
            Token::Text(text) => assert_eq!("Text", text),
            _ => assert!(false, "2nd token was not text."),
        }
        match &t.tokens[2] {
            Token::TagClose(tag) => {
                assert_eq!("url", tag);
            }
            _ => assert!(false, "3rd token was not a closing tag."),
        }
    }

    #[test]
    fn tag_with_strange_broken_args() {
        use super::{Lexer, Token};

        const GIBBERISH: &str = "   👍 wow nice [ test ]";
        let mut t = Lexer::new();
        t.tokenize(&format!("[url{}]Text[/url]", GIBBERISH));
        //println!("{:?}", t.tokens);

        assert_eq!(t.tokens.len(), 3);

        match &t.tokens[0] {
            Token::Text(t1) => match &t.tokens[1] {
                Token::Text(t2) => {
                    assert_eq!(format!("{}{}", t1, t2), format!("[url{}]Text", GIBBERISH));
                }
                _ => assert!(false, "2nd token was not text."),
            },
            _ => assert!(false, "1st token was not text."),
        }
        match &t.tokens[2] {
            Token::TagClose(tag) => {
                assert_eq!(tag, "url");
            }
            _ => assert!(false, "3rd token was not a tag close."),
        }
    }

    #[test]
    fn tag_with_strange_broken_newline_args() {
        use super::{Lexer, Token};

        // parse a tag with a linebreak
        let mut t = Lexer::new();
        t.tokenize("[quote\nbox]");

        assert_eq!(t.tokens.len(), 3);

        if let Token::Text(ref text) = t.tokens[0] {
            assert_eq!(text, "[quote");
        } else {
            assert!(false, "1st token was not text.");
        }

        assert!(
            Token::Linebreak == t.tokens[1],
            "2nd token was not a linebreak."
        );

        if let Token::Text(ref text) = t.tokens[2] {
            assert_eq!(text, "box]");
        } else {
            assert!(false, "3rd token was not text.");
        }
    }

    #[test]
    fn url() {
        use super::{Lexer, Token};

        let mut t = Lexer::new();
        t.tokenize("<https://zombo.com/>");

        let mut output = String::with_capacity(64);
        let mut found_url = false;

        for token in &t.tokens {
            match token {
                Token::Text(ref text) => output.push_str(text),
                Token::Url(ref url) => {
                    found_url = true;
                    assert_eq!("https://zombo.com/", url);
                    output.push_str(url)
                }
                _ => unreachable!(),
            }
        }

        assert!(found_url, "Did not encounter URL token.");
        assert_eq!("&lt;https://zombo.com/&gt;", output);
    }
}
