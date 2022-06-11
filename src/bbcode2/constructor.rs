use rctree::Node;
use std::cell::RefMut;

use super::{Element, Tag};

/// Converts a Parser's AST into rendered HTML.
pub struct Constructor {}

impl Constructor {
    pub fn new() -> Self {
        Self {}
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
                Some(el) => output.push_str(el),
                None => {} // unreachable!(),
            }
        }

        output.push_str(&self.element_close(node.borrow_mut()));

        output
    }

    fn element_open(&self, el: RefMut<Element>) -> String {
        use super::tag::*;

        if let Some(tag) = el.get_tag_name() {
            match get_tag_by_name(tag) {
                Tag::HorizontalRule => self_closing_tag("hr"),
                Tag::Linebreak => self_closing_tag("br"),
                Tag::Plain => String::new(), // Not rendered.

                Tag::Bold => open_simple_tag("b"),
                Tag::Italics => open_simple_tag("i"),
                Tag::Underline => open_simple_tag("u"),
                Tag::Strikethrough => open_simple_tag("s"),

                Tag::Code => open_simple_tag("pre"),

                Tag::Image => open_img_tag(el),

                _ => el.to_open_str(),
            }
        } else {
            String::new()
        }
    }

    fn element_close(&self, el: RefMut<Element>) -> String {
        use super::tag::{close_simple_tag, get_tag_by_name};

        if let Some(tag) = el.get_tag_name() {
            match get_tag_by_name(tag) {
                Tag::Invalid => {
                    if el.is_explicit() {
                        el.to_close_str()
                    } else {
                        String::new()
                    }
                }

                Tag::Bold => close_simple_tag("b"),
                Tag::Italics => close_simple_tag("i"),
                Tag::Underline => close_simple_tag("u"),
                Tag::Strikethrough => close_simple_tag("s"),

                Tag::Code => close_simple_tag("pre"),

                Tag::Image => {
                    if el.is_broken() && el.is_explicit() {
                        "[/img]".to_owned()
                    } else {
                        String::new()
                    }
                }

                _ => String::new(),
            }
        } else {
            String::new()
        }
    }
}

mod tests {
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
