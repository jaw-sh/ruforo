extern crate nom;

mod embed;
mod font;

use super::Element;
use std::{borrow::BorrowMut, cell::RefMut};
use url::Url;

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

    // Block Tags
    Code,

    // Embed Tags
    Image,
    Link,
}

impl Tag {
    pub fn get_by_name(tag: &str) -> Tag {
        match tag {
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
    pub fn open_broken_tag(mut el: RefMut<Element>) -> String {
        el.borrow_mut().set_broken();
        el.to_open_str()
    }

    /// Returns <tagname>
    pub fn open_simple_tag(tag: &str) -> String {
        format!("<{}>", &tag)
    }

    /// Returns </tagname>
    pub fn close_simple_tag(tag: &str) -> String {
        format!("</{}>", &tag)
    }

    /// Returns <tagname />
    pub fn self_closing_tag(tag: &str) -> String {
        format!("<{} />", &tag)
    }

    pub fn open_img_tag(mut el: RefMut<Element>) -> String {
        // Our URL comes from inside the tag.
        if let Some(contents) = el.get_contents() {
            match Url::parse(contents) {
                Ok(url) => match url.scheme() {
                    "http" | "https" => {
                        el.clear_contents();
                        return format!("<img src=\"{}\" />", url.as_str());
                    }
                    _ => {}
                },
                Err(_) => {}
            }
        }

        // If we have no content, we are broken.
        Self::open_broken_tag(el)
    }
}
