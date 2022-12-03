extern crate nom;

mod embed;
mod font;

use super::{Element, SafeHtml};
use std::{borrow::BorrowMut, cell::RefMut};

pub enum Tag {
    // Unique Tags
    Invalid,
    Linebreak,
    HorizontalRule,
    Plain,

    // Inline Tags
    Bold,
    Color,
    Italics,
    Underline,
    Strikethrough,

    // Formatting Tags
    Code,
    Pre,

    // Embed Tags
    Image,
    Link,
}

impl Tag {
    pub fn get_by_name(tag: &str) -> Tag {
        match &*tag.to_lowercase() {
            "b" => Tag::Bold,
            "br" => Tag::Linebreak,
            "color" => Tag::Color,
            "code" => Tag::Code,
            "hr" => Tag::HorizontalRule,
            "i" => Tag::Italics,
            "img" => Tag::Image,
            "plain" => Tag::Plain,
            "s" => Tag::Strikethrough,
            "u" => Tag::Underline,
            "url" => Tag::Link,
            _ => Tag::Invalid,
        }
    }

    /// Sets el to broken, returns [tagname].
    pub fn open_broken_tag(mut el: RefMut<Element>) -> SafeHtml {
        el.borrow_mut().set_broken();
        el.to_open_str()
    }

    /// Returns <tagname>
    pub fn open_simple_tag(tag: &'static str) -> SafeHtml {
        SafeHtml::with_capacity(16) + "<" + tag + ">"
    }

    /// Returns </tagname>
    pub fn close_simple_tag(tag: &'static str) -> SafeHtml {
        SafeHtml::with_capacity(16) + "</" + tag + ">"
    }

    /// Returns <tagname />
    pub fn self_closing_tag(tag: &'static str) -> SafeHtml {
        SafeHtml::with_capacity(16) + "<" + tag + " />"
    }
}
