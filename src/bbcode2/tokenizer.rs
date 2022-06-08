use super::Instruction;
use super::ReadMode;

/// Struct for BbCode tokenization.
#[derive(Default)]
pub struct Tokenizer {
    mode: ReadMode,
    current_instruction: Instruction,
    instructions: Vec<Instruction>,
}

impl Tokenizer {
    /// Creates a new Tokenizer
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads and tokenizes BbCode into individual Instructions.
    pub fn tokenize(&mut self, bbcode: &str) -> &Vec<Instruction> {
        for character in bbcode.chars() {
            self.parse(character);
        }

        self.commit_instruction();
        &self.instructions
    }

    /// Adds `current_instruction` to `instructions` and resets `current_instruction`.
    fn commit_instruction(&mut self) {
        if self.current_instruction != Instruction::Null {
            self.instructions.push(self.current_instruction.clone());
            self.current_instruction = Instruction::Null;
        }
    }

    /// Inserts an instruction directly into `instructions` and resets `current_instruction`.
    fn insert_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
        self.current_instruction = Instruction::Null;
    }

    #[inline]
    fn parse(&mut self, character: char) {
        match &self.mode {
            ReadMode::Text => {
                self.parse_text(character);
            }
            ReadMode::Escape => {
                self.parse_escape(character);
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
            ReadMode::Linebreak => {
                self.parse_linebreak(character);
            }
        }
    }

    /// Intreprets char as plain text input, expecting new instructions.
    fn parse_text(&mut self, character: char) {
        match character {
            '\\' => self.mode = ReadMode::Escape,
            '[' => {
                self.commit_instruction();
                self.mode = ReadMode::Tag;
            }
            '\r' => {}
            '\n' => {
                self.commit_instruction();
                self.mode = ReadMode::Linebreak;
            }
            '>' | '<' | '&' | '"' | '\'' => {
                let san_char = self.sanitize(character);
                match self.current_instruction {
                    Instruction::Text(ref mut contents) => {
                        contents.push_str(&san_char);
                    }
                    _ => {
                        self.current_instruction = Instruction::Text(san_char);
                    }
                }
            }
            _ => match self.current_instruction {
                Instruction::Text(ref mut contents) => {
                    contents.push(character);
                }
                _ => {
                    self.current_instruction = Instruction::Text(character.to_string());
                }
            },
        }
    }

    /// Parses new lines and discards whitespace until next instruction.
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
                self.insert_instruction(Instruction::Linebreak);
                self.mode = ReadMode::Text;
                self.parse_text(character);
            }
        }
    }

    fn parse_escape(&mut self, character: char) {
        self.mode = ReadMode::Text;
        match character {
            '>' | '<' | '&' | '"' | '\'' | '\\' => {
                let san_char = self.sanitize(character);
                match self.current_instruction {
                    Instruction::Tag(ref mut contents, _) => {
                        contents.push_str(&san_char);
                    }
                    _ => {
                        self.current_instruction = Instruction::Text(san_char);
                    }
                }
            }
            _ => match self.current_instruction {
                Instruction::Text(ref mut contents) => {
                    contents.push(character);
                }
                _ => {
                    self.current_instruction = Instruction::Text(character.to_string());
                }
            },
        }
    }

    fn parse_tag(&mut self, character: char) {
        match character {
            // End the tag.
            ']' => {
                self.commit_instruction();
                self.mode = ReadMode::Text;
            }
            // Move to closing tag instruciton.
            '/' => match self.current_instruction {
                // If we've already started our tag, reset to to text.
                Instruction::Tag(_, _) => self.reset_parse_to_text(character),
                // If we've just opened, we can proceed to a closing tag.
                _ => {
                    self.mode = ReadMode::TagClose;
                    self.current_instruction = Instruction::TagClose("".to_string());
                }
            },
            // Hints we should move to arguments
            ' ' | '=' => match self.current_instruction {
                // Begin adding to the arg string, if we have a tag.
                Instruction::Tag(ref tag, _) => {
                    self.current_instruction =
                        Instruction::Tag(tag.to_owned(), Some(character.to_string()));
                    self.mode = ReadMode::TagArg;
                }
                // If we don't have a tag name yet, we choke.
                _ => {
                    self.reset_parse_to_text(character);
                }
            },
            // Add letters
            _ => match self.current_instruction {
                Instruction::Tag(ref mut contents, _) => {
                    contents.push(character);
                }
                _ => {
                    self.current_instruction = Instruction::Tag(character.to_string(), None);
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
                    self.commit_instruction();
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

        match self.current_instruction {
            Instruction::Tag(ref contents, ref mut args) => match args {
                // Add to the Some(string)
                Some(ref mut args) => {
                    args.push(character);
                }
                // Change instruction to include an arg string.
                None => {
                    self.current_instruction =
                        Instruction::Tag(contents.to_string(), Some(character.to_string()));
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
                self.commit_instruction();
                self.mode = ReadMode::Text;
            }
            _ => {
                // if a-Z, commit as tag name
                if character.is_ascii_alphabetic() {
                    match self.current_instruction {
                        Instruction::TagClose(ref mut contents) => {
                            contents.push(character);
                        }
                        _ => {
                            self.current_instruction = Instruction::TagClose(character.to_string())
                        }
                    }
                }
                // otherwise, we have a broken closing tag
                else {
                    self.reset_parse_to_text(character);
                }
            }
        }
    }

    /// Aborts the current ReadMode to Text and converts current instruction to Text.
    /// Supplied char is what choked the parser.
    fn reset_parse_to_text(&mut self, character: char) {
        // Recover existing input.
        let mut text: String = match &self.current_instruction {
            Instruction::Text(content) => {
                log::warn!("Resetting text parse back to text. Should not occur.");
                content.to_string()
            }
            Instruction::Tag(tag, arg) => match arg {
                Some(arg) => format!("[{}{}", tag, arg),
                None => format!("[{}", tag),
            },
            Instruction::TagClose(tag) => format!("[/{}", tag),
            _ => self.current_instruction.to_inner_string(),
        };
        text.push(character);

        self.mode = ReadMode::Text;
        self.current_instruction = Instruction::Text(text);
    }

    /// Sanitizes a char for HTML.
    fn sanitize(&mut self, character: char) -> String {
        match character {
            '<' => "&lt;",
            '>' => "&gt;",
            '&' => "&amp;",
            '"' => "&quot;",
            '\'' => "&#x27;",
            '\\' => "&#x2F;",
            _ => unreachable!(),
        }
        .to_string()
    }
}

mod tests {
    #[test]
    fn linebreak() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("a\n\rb\n\r\r\r\rc\r");

        assert_eq!(t.instructions.len(), 5);

        match &t.instructions[0] {
            Instruction::Text(text) => assert_eq!("a", text),
            _ => assert!(false, "1st instruction was not text."),
        }
        assert!(
            t.instructions[1] == Instruction::Linebreak,
            "2nd instruction not linebreak."
        );
        match &t.instructions[4] {
            Instruction::Text(text) => assert_eq!("c", text),
            _ => assert!(false, "5th instruction was not text."),
        }
    }

    #[test]
    fn sanitize() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("<strong>HTML</strong>");

        assert_eq!(t.instructions.len(), 1);

        match &t.instructions[0] {
            Instruction::Text(text) => assert_eq!("&lt;strong&gt;HTML&lt;/strong&gt;", text),
            _ => assert!(false, "Instruction was not text."),
        }
    }

    #[test]
    fn tag_and_close() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("[b]Bold[/b]");

        assert_eq!(t.instructions.len(), 3);

        match &t.instructions[0] {
            Instruction::Tag(tag, arg) => {
                assert_eq!("b", tag);
                assert_eq!(&None, arg);
            }
            _ => assert!(false, "1st instruction was not a tag."),
        }
        match &t.instructions[1] {
            Instruction::Text(text) => assert_eq!("Bold", text),
            _ => assert!(false, "2nd instruction was not text."),
        }
        match &t.instructions[2] {
            Instruction::TagClose(tag) => {
                assert_eq!("b", tag);
            }
            _ => assert!(false, "3rd instruction was not a closing tag."),
        }
    }

    #[test]
    fn tag_close_terminates() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("[b]Bold[//b]");

        assert_eq!(t.instructions.len(), 3);

        match &t.instructions[2] {
            Instruction::Text(text) => {
                assert_eq!("[//b]", text);
            }
            _ => assert!(false, "3rd instruction was not text."),
        }
    }

    #[test]
    fn tag_open_terminates() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("[b]Bold[b/b]");

        assert_eq!(t.instructions.len(), 3);

        match &t.instructions[2] {
            Instruction::Text(text) => {
                assert_eq!("[b/b]", text);
            }
            _ => assert!(false, "3rd instruction was not text."),
        }
    }

    #[test]
    fn tag_with_arg() {
        use super::{Instruction, Tokenizer};

        let mut t = Tokenizer::new();
        t.tokenize("[url=https://zombo.com]ZOMBO[/url]");

        assert_eq!(t.instructions.len(), 3);

        match &t.instructions[0] {
            Instruction::Tag(tag, arg) => {
                assert_eq!("url", tag);
                assert_eq!(&Some("=https://zombo.com".to_string()), arg);
            }
            _ => assert!(false, "1st instruction was not a tag."),
        }
        match &t.instructions[1] {
            Instruction::Text(text) => assert_eq!("ZOMBO", text),
            _ => assert!(false, "2nd instruction was not text."),
        }
        match &t.instructions[2] {
            Instruction::TagClose(tag) => {
                assert_eq!("url", tag);
            }
            _ => assert!(false, "3rd instruction was not a closing tag."),
        }
    }

    #[test]
    fn tag_with_strange_args() {
        use super::{Instruction, Tokenizer};

        // This content can be parsed as correct because the tokenizer does not care
        // about the validity of the arguments.
        const GIBBERISH: &str = "   👍 wow nice \"[test]\"";

        let mut t = Tokenizer::new();
        t.tokenize(&format!("[url{}]Text[/url]", GIBBERISH));

        assert_eq!(t.instructions.len(), 3);
        match &t.instructions[0] {
            Instruction::Tag(tag, arg) => {
                assert_eq!("url", tag);
                assert_eq!(&Some(GIBBERISH.to_string()), arg);
            }
            _ => assert!(false, "1st instruction was not a tag."),
        }
        match &t.instructions[1] {
            Instruction::Text(text) => assert_eq!("Text", text),
            _ => assert!(false, "2nd instruction was not text."),
        }
        match &t.instructions[2] {
            Instruction::TagClose(tag) => {
                assert_eq!("url", tag);
            }
            _ => assert!(false, "3rd instruction was not a closing tag."),
        }
    }

    #[test]
    fn tag_with_strange_broken_args() {
        use super::{Instruction, Tokenizer};

        const GIBBERISH: &str = "   👍 wow nice [test]";

        let mut t = Tokenizer::new();
        t.tokenize(&format!("[url{}]Text[/url]", GIBBERISH));
        println!("{:?}", t.instructions);

        assert_eq!(t.instructions.len(), 2);

        match &t.instructions[0] {
            Instruction::Text(text) => {
                assert_eq!(&format!("[url{}]Text", GIBBERISH), text);
            }
            _ => assert!(false, "1st instruction was not text."),
        }
    }
}
