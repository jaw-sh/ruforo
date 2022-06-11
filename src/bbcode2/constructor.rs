use rctree::Node;
use std::cell::Ref;

use super::{Element, Tag};

/// Converts a Parser's AST into rendered HTML.
pub struct Constructor {}

impl Constructor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&self, node: &Node<Element>) -> String {
        let mut output: String = "".to_string();
        let el = node.borrow();

        output.push_str(&self.element_open(node.borrow()));

        // If we have children, loop through them.
        if node.has_children() {
            for child in node.children() {
                output.push_str(&self.build(&child))
            }
        }
        // If we do not have children, add our text.
        else {
            match el.get_contents() {
                Some(el) => output.push_str(el),
                None => {} // unreachable!(),
            }
        }

        output.push_str(&self.element_close(node.borrow()));

        output
    }

    fn element_open(&self, el: Ref<Element>) -> String {
        use super::tag::{get_tag_by_name, open_simple_tag, self_closing_tag};

        if let Some(tag) = el.get_tag_name() {
            match get_tag_by_name(tag) {
                Tag::Invalid => {}
                Tag::HorizontalRule => return self_closing_tag("hr"),
                Tag::Linebreak => return self_closing_tag("br"),
                Tag::Plain => {}

                Tag::Bold => return open_simple_tag("b"),
                Tag::Italics => return open_simple_tag("i"),
                Tag::Underline => return open_simple_tag("u"),
                Tag::Strikethrough => return open_simple_tag("s"),
            }
        }

        "".to_string()
    }

    fn element_close(&self, el: Ref<Element>) -> String {
        use super::tag::{close_simple_tag, get_tag_by_name};

        if let Some(tag) = el.get_tag_name() {
            match get_tag_by_name(tag) {
                Tag::Invalid => {}
                Tag::Linebreak => {}
                Tag::HorizontalRule => {}
                Tag::Plain => {}

                Tag::Bold => return close_simple_tag("b"),
                Tag::Italics => return close_simple_tag("i"),
                Tag::Underline => return close_simple_tag("u"),
                Tag::Strikethrough => return close_simple_tag("s"),
            }
        }

        "".to_string()
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

        let out = con.build(&ast);
        assert_eq!(out, "Hello, world!".to_owned());
    }

    #[test]
    fn text_only() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();
        let ast = Node::new(Element::new_text(&"Hello, world!".to_owned()));
        let out = con.build(&ast);

        assert_eq!(out, "Hello, world!".to_owned());
    }
}
