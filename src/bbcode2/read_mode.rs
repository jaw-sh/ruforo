/// Tokenizer modes.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    Text,
    Escape,
    Linebreak,
    Tag,
    TagClose,
    TagPrimaryArg,
}

impl Default for ReadMode {
    fn default() -> Self {
        ReadMode::Text
    }
}
