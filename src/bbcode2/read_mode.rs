/// Tokenizer modes.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    Text,
    Escape,
    Linebreak,
    Tag,
    TagPrimaryArg,
}

impl Default for ReadMode {
    fn default() -> Self {
        ReadMode::Text
    }
}
