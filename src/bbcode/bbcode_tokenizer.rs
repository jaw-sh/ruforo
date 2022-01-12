use super::Instruction;

/// Tokenizer modes.
#[derive(Debug, PartialEq)]
enum ReadMode {
    Text,
    Escape,
    Tag,
    TagPrimaryArg,
    Parabreak,
}

impl Default for ReadMode {
    fn default() -> Self {
        ReadMode::Text
    }
}

/// Struct for BBCode tokenization.
#[derive(Default)]
pub struct BBCodeTokenizer {
    mode: ReadMode,
    current_instruction: Instruction,
    instructions: Vec<Instruction>,
}

impl BBCodeTokenizer {
    /// Creates a new BBCodeTokenizer
    pub fn new() -> Self {
        Default::default()
    }
    /// Reads and tokenizes BBCode into individual Instructions.
    pub fn tokenize(&mut self, bbcode: &str) -> &Vec<Instruction> {
        let bbcode_chars = bbcode.chars();
        for character in bbcode_chars {
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
                ReadMode::TagPrimaryArg => {
                    self.parse_tag_primary_arg(character);
                }
                ReadMode::Parabreak => {
                    self.parse_parabreak(character);
                }
            }
        }
        self.set_cur_instruction();
        &self.instructions
    }

    /// s characters.
    fn parse_text(&mut self, character: char) {
        match character {
            '\\' => self.mode = ReadMode::Escape,
            '[' => {
                self.set_cur_instruction();
                self.mode = ReadMode::Tag;
            }
            '\r' => {}
            '\n' => {
                self.set_cur_instruction();
                self.mode = ReadMode::Parabreak;
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

    /// s paragraph breaks.
    fn parse_parabreak(&mut self, character: char) {
        match character {
            '\t' => {
                self.set_new_instruction(Instruction::Parabreak("\n\t".to_string()));
                self.mode = ReadMode::Text;
            }
            // Consume carriage returns.
            '\r' => {}
            // Consume whitespace.
            ' ' => {}
            _ => {
                self.set_new_instruction(Instruction::Linebreak);
                self.mode = ReadMode::Text;
                self.parse_text(character);
            }
        }
    }

    /// s escaped charcters.
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
    /// s BBCode tags.
    fn parse_tag(&mut self, character: char) {
        match character {
            ']' => {
                self.set_cur_instruction();
                self.mode = ReadMode::Text;
            }
            '=' => {
                self.mode = ReadMode::TagPrimaryArg;
            }
            '>' | '<' | '&' | '"' | '\'' | '\\' => {
                let san_char = self.sanitize(character);
                match self.current_instruction {
                    Instruction::Tag(ref mut contents, _) => {
                        contents.push_str(&san_char);
                    }
                    _ => {
                        self.current_instruction = Instruction::Tag(san_char, None);
                    }
                }
            }
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
    /// s BBCode tag arguments.
    fn parse_tag_primary_arg(&mut self, character: char) {
        match character {
            ']' => {
                self.set_cur_instruction();
                self.mode = ReadMode::Text;
            }
            '>' | '<' | '&' | '"' | '\'' | '\\' => {
                let san_char = self.sanitize(character);
                match self.current_instruction {
                    Instruction::Tag(ref mut contents, ref mut args) => match args {
                        Some(ref mut primarg) => {
                            primarg.push_str(&san_char);
                        }
                        None => {
                            self.current_instruction =
                                Instruction::Tag((*contents).to_string(), Some(san_char));
                        }
                    },
                    _ => {
                        unreachable!();
                    }
                }
            }
            _ => match self.current_instruction {
                Instruction::Tag(ref mut contents, ref mut args) => match args {
                    Some(ref mut primarg) => {
                        primarg.push(character);
                    }
                    None => {
                        self.current_instruction =
                            Instruction::Tag((*contents).to_string(), Some(character.to_string()));
                    }
                },
                _ => {
                    unreachable!();
                }
            },
        }
    }

    /// Adds current instruction to instruction vector and restes current instruction.
    fn set_cur_instruction(&mut self) {
        if self.current_instruction != Instruction::Null {
            self.instructions.push(self.current_instruction.clone());
            self.current_instruction = Instruction::Null;
        }
    }

    /// Adds a given instruction to instruction vector and resets current instruction.
    fn set_new_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
        self.current_instruction = Instruction::Null;
    }

    /// Sanitizes characters for HTML.
    fn sanitize(&mut self, character: char) -> String {
        match character {
            '<' => "&lt",
            '>' => "&gt",
            '&' => "&amp",
            '"' => "&quot",
            '\'' => "&#x27",
            '\\' => "&#x2F",
            _ => unreachable!(),
        }
        .to_string()
    }
}
