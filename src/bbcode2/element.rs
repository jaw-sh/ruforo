use super::Token;

#[derive(Debug, Clone)]
pub enum ElementDisplay {
    Inline,
    Block,
}

impl Default for ElementDisplay {
    fn default() -> Self {
        Self::Inline
    }
}

/// A single element of a BbCode Abstract Syntax Tree (AST).
#[derive(Debug, Default, Clone)]
pub struct Element {
    /// Tag name.
    /// If set, this element should defer some logic to BbCode tags.
    tag: Option<String>,
    /// Tag arguments.
    /// If set, this element contains content after the tag name.
    argument: Option<String>,
    /// Tag content.
    /// If set, this element contains text.
    /// Example: \[quote\]What doth life?\[/quote\]
    contents: Option<String>,
    /// Types determine what other elements this one can safely embed in or close.
    display: ElementDisplay,

    /// If true, this tag is malformed and should revert to text.
    /// Example: \[url=gibberish\]bad url\[/url\]
    is_broken: bool,
    /// If true, the contents of this tag are always literal.
    /// Example: \[code\]my bbcode here\[code\]
    is_literal: bool,
    /// If true, this element is not allowed to contain anything at all, including text.
    /// Example: \[hr\] tags, linebreaks.
    is_void: bool,
}

impl Element {
    /// Converts a Lexer's Token into a Parser's Element.
    pub fn new_from_token(token: &Token) -> Self {
        match token {
            Token::Linebreak => Self {
                tag: Some("br".to_owned()),
                is_void: true,
                ..Self::default()
            },
            Token::Tag(tag, arg) => Self {
                tag: Some(tag.to_owned()),
                argument: arg.to_owned(),
                ..Self::default()
            },
            Token::Text(text) => Self::new_text(text),
            _ => unreachable!(),
        }
    }

    // Text-only element
    pub fn new_text(text: &String) -> Self {
        Self {
            contents: Some(text.to_owned()),
            ..Self::default()
        }
    }

    /// DOM Root
    pub fn new_root() -> Self {
        Self {
            display: ElementDisplay::Block,
            ..Self::default()
        }
    }

    pub fn add_text(&mut self, text: &String) {
        // Add text if possible.
        if !self.is_void {
            match self.contents {
                Some(ref mut contents) => contents.push_str(text),
                None => self.contents = Some(text.to_owned()),
            };
            return;
        }

        unreachable!("Parser trying to add text to void element.")
    }

    /// If true, this node can accept the given element as a child.
    /// If false, reason it cannot.
    pub fn can_parent(&self, node: &Element) -> bool {
        !self.is_literal && !self.is_void
    }

    pub fn extract_contents(&mut self) -> Option<Element> {
        let res = match &self.contents {
            Some(text) => Some(Self::new_text(text)),
            None => None,
        };
        self.contents = None;
        res
    }

    pub fn get_contents(&self) -> Option<&String> {
        self.contents.as_ref()
    }

    pub fn get_display_type(&self) -> ElementDisplay {
        self.display.to_owned()
    }

    pub fn get_tag_name(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    /// If true, all contents must never be parsed.
    pub fn is_literal(&self) -> bool {
        self.is_literal
    }

    pub fn is_tag(&self, other: &String) -> bool {
        match &self.tag {
            Some(ours) => ours == other,
            None => false,
        }
    }

    /// If true, no element or text may be added to this element.
    pub fn is_void(&self) -> bool {
        self.is_void
    }
}
