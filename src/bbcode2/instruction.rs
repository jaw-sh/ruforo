/// A single Instruction output by the tokenizer.
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Null,
    Tag(String, Option<String>),
    TagClose(String),
    Text(String),
    Linebreak,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::Null
    }
}
