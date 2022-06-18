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
pub struct Element<'str> {
    // Raw data provided by the token.
    raw: Option<&'str str>,
    /// Tag name.
    /// If set, this element should defer some logic to BbCode tags.
    tag: Option<&'str str>,
    /// Tag arguments.
    /// If set, this element contains content after the tag name.
    argument: Option<&'str str>,
    /// Tag content.
    /// If set, this element contains text.
    /// Example: \[quote\]What doth life?\[/quote\]
    contents: Option<&'str str>,
    /// Types determine what other elements this one can safely embed in or close.
    display: ElementDisplay,
    /// When parsing arguments, elements may break, which defaults rendering as text.
    broken: bool,
    // If true, this element was explicitly defined and closed.
    explicit: bool,
}

impl<'str> Element<'str> {
    fn new_for_tag(raw: &'str str, tag: &'str str, arg: Option<&'str str>) -> Self {
        let mut el = Self {
            raw: Some(raw),
            tag: Some(tag),
            argument: arg,
            ..Self::default()
        };

        // Adjust display
        el.display = match Tag::get_by_name(tag) {
            Tag::Invalid => {
                el.broken = true;
                ElementDisplay::Inline
            }
            Tag::Linebreak => ElementDisplay::Selfclosing,
            Tag::HorizontalRule => ElementDisplay::Selfclosing,
            Tag::Plain => ElementDisplay::Plain,
            Tag::Code => ElementDisplay::Preformatted,
            Tag::Image => ElementDisplay::Plain,
            Tag::Link => ElementDisplay::Inline,
            _ => ElementDisplay::Inline,
        };

        el
    }

    // Text-only element
    pub fn new_from_text(text: &'str str) -> Self {
        Self {
            raw: Some(text),
            contents: Some(text),
            ..Self::default()
        }
    }

    /// Converts a Lexer's Token into a Parser's Element.
    pub fn new_from_token(token: &'str Token) -> Self {
        match token {
            Token::Linebreak(raw) => Self {
                raw: Some(raw),
                tag: Some("br"),
                display: ElementDisplay::Selfclosing,
                ..Self::default()
            },
            Token::Tag(raw, tag, arg) => Self::new_for_tag(raw, tag, *arg),
            Token::TagClose(raw, _) => Self::new_from_text(raw), // Closing tags are consumed unless they are unpaired.
            Token::Text(text) => Self::new_from_text(text),
            Token::Url(url) => Self {
                tag: Some("url"),
                raw: Some(url),
                contents: Some(url),
                explicit: true,
                ..Default::default()
            },
            _ => unreachable!(),
        }
    }

    /// DOM Root
    pub fn new_root() -> Self {
        Self {
            display: ElementDisplay::Block,
            ..Self::default()
        }
    }

    //pub fn add_text(&mut self, text: &'str str) {
    //    match self.display {
    //        ElementDisplay::Selfclosing => {
    //            unreachable!("Parser trying to insert text in self-closing element.")
    //        }
    //        _ => {
    //            // Set our contents to include new text.
    //            match self.contents {
    //                Some(ref mut contents) => contents.push_str(text),
    //                None => self.contents = Some(text),
    //            }
    //        }
    //    }
    //}

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

    /// Exceptions list for tags.
    pub fn can_parent_tag(&self, theirs: &'str str) -> bool {
        // A last resort for exceptional tags.
        // Almost all cases for parentage should be handled through ElementDisplay.
        match self.tag {
            Some(ours) => match Tag::get_by_name(ours) {
                Tag::Link => match Tag::get_by_name(theirs) {
                    Tag::Link => false,
                    _ => true,
                },
                _ => true,
            },
            None => true,
        }
    }

    pub fn clear_contents(&mut self) {
        self.contents = None;
    }

    pub fn extract_contents(&mut self) -> Option<Element<'str>> {
        let res = match self.contents {
            Some(text) => Some(Self::new_from_text(text)),
            None => None,
        };
        self.contents = None;
        res
    }

    pub fn get_argument(&self) -> Option<&'str str> {
        self.argument
    }

    pub fn get_contents(&self) -> Option<&'str str> {
        self.contents
    }

    pub fn get_display_type(&self) -> ElementDisplay {
        self.display.to_owned()
    }

    pub fn get_tag_name(&self) -> Option<&'str str> {
        self.tag
    }

    pub fn get_raw(&self) -> &'str str {
        self.raw.unwrap_or("")
    }

    pub fn has_argument(&self) -> bool {
        self.argument.unwrap_or("").len() > 0
    }

    pub fn has_contents(&self) -> bool {
        self.contents.unwrap_or("").len() > 0
    }

    pub fn is_broken(&self) -> bool {
        self.broken
    }

    pub fn is_explicit(&self) -> bool {
        self.explicit
    }

    pub fn is_tag(&self, other: &str) -> bool {
        match self.tag {
            Some(ours) => ours == other,
            None => false,
        }
    }

    pub fn set_argument(&mut self, input: &'str str) {
        self.argument = Some(input);
    }

    pub fn set_broken(&mut self) {
        self.broken = true;
    }

    pub fn set_contents(&mut self, input: &'str str) {
        self.contents = Some(input);
    }

    pub fn set_explicit(&mut self) {
        self.explicit = true;
    }

    /// Unwinds element into an opening tag string.
    pub fn to_open_str(&self) -> String {
        match self.raw {
            Some(raw) => raw,
            None => "",
        }
        .to_owned()
        //match &self.tag {
        //    Some(tag) => match &self.argument {
        //        Some(argument) => format!("[{}{}]", tag, argument),
        //        None => format!("[{}]", tag),
        //    },
        //    None => match &self.argument {
        //        Some(argument) => format!("[{}]", argument),
        //        None => "[/]".to_string(),
        //    },
        //}
    }

    /// Unwinds element into an closing tag string.
    pub fn to_close_str(&self) -> String {
        // Only explicitly closed tags reverse to BbCode
        if self.is_explicit() {
            match &self.tag {
                Some(tag) => format!("[/{}]", tag),
                None => "[/]".to_string(),
            }
        }
        // Broken, implicitly closed tags reverse to nothing.
        else {
            String::new()
        }
    }
}
