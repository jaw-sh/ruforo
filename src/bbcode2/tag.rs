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
    Italics,
    Underline,
    Strikethrough,

    // Block Tags
    Code,

    // Embed Tags
    Image,
    Link,
}

pub fn get_tag_by_name(tag: &str) -> Tag {
    match tag {
        "b" => Tag::Bold,
        "br" => Tag::Linebreak,
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
    el.borrow_mut().set_broken();

    // Return raw bbcode
    if let Some(contents) = el.get_contents() {
        format!("[img]{}", contents)
    } else {
        "[img]".to_string()
    }
}

pub fn open_url_tag(mut el: RefMut<Element>) -> String {
    // Our URL comes from inside the tag.
    if let Some(contents) = el.get_contents() {
        match Url::parse(contents) {
            Ok(url) => match url.scheme() {
                "http" | "https" => {
                    el.clear_contents();
                    return format!(
                        "<a href=\"{}\" rel=\"nofollow\">{}</a>",
                        url.as_str(),
                        url.as_str()
                    );
                }
                _ => {}
            },
            Err(_) => {}
        }
    }

    // If we have no content, we are broken.
    el.borrow_mut().set_broken();

    // Return raw bbcode
    if let Some(contents) = el.get_contents() {
        format!("[url]{}", contents)
    } else {
        "[url]".to_string()
    }
}
