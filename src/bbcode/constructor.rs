use rctree::Node;
use std::{cell::RefMut, collections::HashMap};

use super::{Element, Tag};

/// Converts a Parser's AST into rendered HTML.
#[derive(Default)]
pub struct Constructor {
    // TODO: Build string here, return in build().
    pub emojis: Option<HashMap<String, String>>,
}

impl Constructor {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn build(&self, mut node: Node<Element>) -> String {
        let mut output: String = String::new();

        output.push_str(&self.element_open(node.borrow_mut()));

        // If we have children, loop through them.
        if node.has_children() {
            for child in node.children() {
                output.push_str(&self.build(child))
            }
        }
        // If we do not have children, add our text.
        else {
            let el = node.borrow();
            match el.get_contents() {
                Some(content) => {
                    output.push_str(&match el.can_parent() {
                        true => self.replace_emojis(Self::sanitize(content)),
                        false => Self::sanitize(content),
                    });
                }
                None => {} // unreachable!(),
            }
        }

        output.push_str(&self.element_close(node.borrow_mut()));

        output
    }

    fn element_open(&self, el: RefMut<Element>) -> String {
        use super::tag::*;

        if let Some(tag) = el.get_tag_name() {
            match Tag::get_by_name(tag) {
                Tag::HorizontalRule => Tag::self_closing_tag("hr"),
                Tag::Linebreak => Tag::self_closing_tag("br"),
                Tag::Plain => String::new(), // Not rendered.

                Tag::Bold => Tag::open_simple_tag("b"),
                Tag::Color => Tag::open_color_tag(el),
                Tag::Italics => Tag::open_simple_tag("i"),
                Tag::Underline => Tag::open_simple_tag("u"),
                Tag::Strikethrough => Tag::open_simple_tag("s"),

                Tag::Code => Tag::open_simple_tag("pre"),

                Tag::Image => Tag::open_img_tag(el),
                Tag::Link => Tag::open_url_tag(el),

                _ => el.to_open_str(),
            }
        } else {
            String::new()
        }
    }

    fn element_close(&self, el: RefMut<Element>) -> String {
        // Only named elements close with output.
        if let Some(tag) = el.get_tag_name() {
            // Only unbroken tags render HTML.
            if !el.is_broken() {
                match Tag::get_by_name(tag) {
                    Tag::Invalid => el.to_close_str(),

                    Tag::Bold => Tag::close_simple_tag("b"),
                    Tag::Color => Tag::close_simple_tag("span"),
                    Tag::Italics => Tag::close_simple_tag("i"),
                    Tag::Underline => Tag::close_simple_tag("u"),
                    Tag::Strikethrough => Tag::close_simple_tag("s"),

                    Tag::Code => Tag::close_simple_tag("pre"),

                    Tag::Link => Tag::close_simple_tag("a"),

                    // Self-closing tags do not close.
                    _ => String::new(),
                }
            }
            // Broken tags reverse to original input.
            else {
                el.to_close_str()
            }
        }
        // Unnamed tags reverse to nothing.
        else {
            String::new()
        }
    }

    /// Add emojis
    pub fn replace_emojis(&self, input: String) -> String {
        let mut result = input;

        if let Some(emojis) = &self.emojis {
            for emoji in emojis {
                result = result.replace(emoji.0, emoji.1);
            }
        }

        result
    }

    /// Sanitizes a char for HTML.
    fn sanitize(input: &String) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('\\', "&#x2F;")
    }
}

mod tests {
    #[test]
    fn reusable() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();

        // First pass
        let mut ast = Node::new(Element::new_root());
        ast.append(Node::new(Element::new_text(&"Hello, world!".to_owned())));

        assert_eq!(ast.children().count(), 1);
        assert_eq!(con.build(ast), "Hello, world!".to_owned());

        // Second pass
        let mut ast = Node::new(Element::new_root());
        ast.append(Node::new(Element::new_text(&"Foo, bar!".to_owned())));

        assert_eq!(ast.children().count(), 1);
        assert_eq!(con.build(ast), "Foo, bar!".to_owned());
    }

    #[test]
    fn text_in_empty_nest() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();
        let mut ast = Node::new(Element::new_root());
        let mut child = Node::new(Element::new_root());
        ast.append(child.clone());

        for _ in 1..10 {
            let node = Node::new(Element::new_root());
            let clone = node.clone();
            child.append(node);
            child = clone.clone();
        }
        child.append(Node::new(Element::new_text(&"Hello, world!".to_owned())));

        let out = con.build(ast);
        assert_eq!(out, "Hello, world!".to_owned());
    }

    #[test]
    fn text_only() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();
        let ast = Node::new(Element::new_text(&"Hello, world!".to_owned()));
        let out = con.build(ast);

        assert_eq!(out, "Hello, world!".to_owned());
    }
}
