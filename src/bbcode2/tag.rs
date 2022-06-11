use super::Element;
use phf::phf_map;

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
}

pub fn get_tag_by_name(tag: &str) -> Tag {
    match tag {
        "b" => Tag::Bold,
        "br" => Tag::Linebreak,
        "hr" => Tag::HorizontalRule,
        "i" => Tag::Italics,
        "plain" => Tag::Plain,
        "s" => Tag::Strikethrough,
        "u" => Tag::Underline,
        _ => unreachable!(),
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
