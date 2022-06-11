/// Tokenizer modes.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    Text,
    Linebreak,
    Tag,
    TagArg,
    TagArgQuote,
    TagClose,
    Url(bool),
}

impl Default for ReadMode {
    fn default() -> Self {
        Self::Text
    }
}
