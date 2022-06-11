use super::{Tag, Token};

#[derive(Debug, Clone)]
pub enum ElementDisplay {
    /// Element which may not be closed by its interiors.
    Block,
    /// Element which renders inline may be closed automatically in some situations.
    Inline,
    /// Element with content not parsed to BBCode.
    Plain,
    /// Element with content which whitespace is always preserved.
    Preformatted,
    /// Element with no content.
    Selfclosing,
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
    /// When parsing arguments, elements may break, which defaults rendering as text.
    broken: bool,
    // If true, this element was explicitly defined and closed.
    explicit: bool,
}

impl Element {
    fn new_for_tag(tag: &String, arg: &Option<String>) -> Self {
        use super::tag::get_tag_by_name;

        let mut el = Self {
            tag: Some(tag.to_owned()),
            argument: arg.to_owned(),
            ..Self::default()
        };

        // Adjust display
        el.display = match get_tag_by_name(tag) {
            Tag::Invalid => {
                el.broken = true;
                ElementDisplay::Inline
            }
            Tag::Linebreak => unreachable!(),
            Tag::HorizontalRule => ElementDisplay::Selfclosing,
            Tag::Plain => ElementDisplay::Plain,
            Tag::Code => ElementDisplay::Preformatted,
            Tag::Image => ElementDisplay::Plain,
            _ => ElementDisplay::Inline,
        };

        el
    }

    /// Converts a Lexer's Token into a Parser's Element.
    pub fn new_from_token(token: &Token) -> Self {
        match token {
            Token::Linebreak => Self {
                tag: Some("br".to_owned()),
                display: ElementDisplay::Selfclosing,
                ..Self::default()
            },
            Token::Tag(tag, arg) => Self::new_for_tag(tag, arg),
            Token::Text(text) => Self::new_text(text),
            Token::Url(url) => Self {
                tag: Some("url".to_owned()),
                contents: Some(url.to_owned()),
                explicit: true,
                ..Default::default()
            },
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
        match self.display {
            ElementDisplay::Selfclosing => {
                unreachable!("Parser trying to insert text in self-closing element.")
            }
            _ => {
                // Set our contents to include new text.
                match self.contents {
                    Some(ref mut contents) => contents.push_str(text),
                    None => self.contents = Some(text.to_owned()),
                }
            }
        }
    }

    /// If true, this node can have text.
    /// If false, it should never contain anything.
    pub fn can_have_content(&self) -> bool {
        match self.display {
            ElementDisplay::Selfclosing => false,
            _ => true,
        }
    }

    /// If true, this node can accept <br/> tags.
    /// If false, it depends on other checks what it can accept.
    pub fn can_linebreak(&self) -> bool {
        match self.display {
            ElementDisplay::Preformatted => false,
            ElementDisplay::Selfclosing => false,
            _ => true,
        }
    }

    /// If true, this node can accept the given element as a child.
    /// If false, it should never have child tag elements.
    pub fn can_parent(&self) -> bool {
        match self.display {
            ElementDisplay::Plain => false,
            ElementDisplay::Preformatted => false,
            ElementDisplay::Selfclosing => false,
            _ => true,
        }
    }

    pub fn clear_contents(&mut self) {
        self.contents = None;
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

    pub fn is_broken(&self) -> bool {
        self.broken
    }

    pub fn is_explicit(&self) -> bool {
        self.explicit
    }

    pub fn is_tag(&self, other: &String) -> bool {
        match &self.tag {
            Some(ours) => ours == other,
            None => false,
        }
    }

    pub fn set_broken(&mut self) {
        self.broken = true;
    }

    pub fn set_explicit(&mut self) {
        self.explicit = true;
    }

    /// Unwinds element into an opening tag string.
    pub fn to_open_str(&self) -> String {
        match &self.tag {
            Some(tag) => match &self.argument {
                Some(argument) => format!("[{}{}]", tag, argument),
                None => format!("[{}]", tag),
            },
            None => match &self.argument {
                Some(argument) => format!("[{}]", argument),
                None => "[/]".to_string(),
            },
        }
    }

    /// Unwinds element into an closing tag string.
    pub fn to_close_str(&self) -> String {
        match &self.tag {
            Some(tag) => format!("[/{}]", tag),
            None => "[/]".to_string(),
        }
    }
}
