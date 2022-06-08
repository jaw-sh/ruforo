/// A single Instruction output by the tokenizer.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Null,
    Linebreak,
    Tag(String, Option<String>),
    TagClose(String),
    Text(String),
}

impl Instruction {
    /// Converts instruction to string, without syntax.
    pub fn to_inner_string(&self) -> String {
        match self {
            Self::Null => "".to_string(),
            Self::Linebreak => "\n\r".to_string(),
            Self::Tag(tag, arg) => match arg {
                Some(arg) => format!("{}={}", tag, arg),
                None => format!("{}", tag),
            },
            Self::TagClose(tag) => format!("{}", tag),
            Self::Text(text) => format!("{}", text),
        }
    }

    /// Reverses instruction to string.
    pub fn to_tag_string(&self) -> String {
        match self {
            Self::Null | Self::Linebreak | Self::Text(_) => self.to_inner_string(),
            Self::Tag(_, _) => format!("[{}]", self.to_inner_string()),
            Self::TagClose(_) => format!("[/{}]", self.to_inner_string()),
        }
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::Null
    }
}

mod tests {
    #[test]
    fn reverse_null() {
        use super::Instruction;
        let inst = Instruction::Null;

        assert_eq!(inst.to_inner_string(), "");
        assert_eq!(inst.to_tag_string(), "");
    }

    #[test]
    fn reverse_linebreak() {
        use super::Instruction;
        let inst = Instruction::Linebreak;

        assert_eq!(inst.to_inner_string(), "\n\r");
        assert_eq!(inst.to_tag_string(), "\n\r");
    }

    #[test]
    fn reverse_tag() {
        use super::Instruction;
        let tag = "quotebox".to_string();
        let inst = Instruction::Tag(tag, None);

        assert_eq!(inst.to_inner_string(), "quotebox");
        assert_eq!(inst.to_tag_string(), "[quotebox]");

        let tag2 = "url".to_string();
        let inst2 = Instruction::Tag(tag2, Some("https://zombo.com/".to_string()));
        assert_eq!(inst2.to_inner_string(), "url=https://zombo.com/");
        assert_eq!(inst2.to_tag_string(), "[url=https://zombo.com/]");
    }

    #[test]
    fn reverse_tag_close() {
        use super::Instruction;
        let tag = "quotebox".to_string();
        let inst = Instruction::TagClose(tag);

        assert_eq!(inst.to_inner_string(), "quotebox");
        assert_eq!(inst.to_tag_string(), "[/quotebox]");
    }

    #[test]
    fn reverse_text() {
        use super::Instruction;
        let text = "text input :)".to_string();
        let inst = Instruction::Text(text);

        assert_eq!(inst.to_inner_string(), "text input :)");
        assert_eq!(inst.to_tag_string(), "text input :)");
    }
}
