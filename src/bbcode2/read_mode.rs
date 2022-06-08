/// Tokenizer modes.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    Text,
    Escape,
    Linebreak,
    Tag,
    TagArg,
    TagArgQuote,
    TagClose,
}

impl Default for ReadMode {
    fn default() -> Self {
        ReadMode::Text
    }
}
