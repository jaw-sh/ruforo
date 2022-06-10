use super::Element;
use rctree::Node;

/// Converts a Parser's AST into rendered HTML.
pub struct Constructor {}

impl Constructor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(&self, node: &Node<Element>) -> String {
        let mut output: String = "".to_string();
        let el = node.borrow();

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
                None => unreachable!(),
            }
        }

        output
    }
}

mod tests {
    #[test]
    fn test_in_empty_nest() {
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
