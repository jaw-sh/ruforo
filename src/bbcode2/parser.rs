use super::Element;
use super::ElementDisplay;
use super::Token;
use rctree::Node;

/// Struct for parsing BbCode Tokens into an Element tree.
pub struct Parser {
    /// DOM root
    root: Node<Element>,
    /// Current traversal node.
    node: Node<Element>,
}

impl Parser {
    pub fn new() -> Self {
        // The rctree's Node<> is a modified RefCell, so cloning is just a ref.
        // See: https://docs.rs/rctree/latest/rctree/struct.Node.html#impl-Clone
        let dom = Element::new_root();
        let root = Node::new(dom);
        let node = root.clone();

        Self { root, node }
    }

    pub fn parse(&mut self, tokens: &[Token]) {
        for token in tokens {
            match token {
                Token::Null => {
                    log::warn!("BbCode Lexer received Null instruction, which should not happen.");
                }
                Token::Linebreak => self.add_linebreak(),
                Token::Tag(_, _) => self.open_tag(Element::new_from_token(token)),
                Token::TagClose(tag) => self.close_tag(tag),
                Token::Text(text) => self.add_text(text),
            }
        }
    }

    fn add_linebreak(&mut self) {}

    fn add_text(&mut self, text: &String) {
        self.node.borrow_mut().add_text(text);
    }

    fn close_tag(&mut self, tag: &String) {
        if tag.len() < 1 {
            log::warn!("Attempted to close a tag with no name.");
            return;
        }

        while let Some(parent) = self.node.parent() {
            let (close, end) = {
                let node = self.node.borrow();
                match node.get_display_type() {
                    // Inline tags may be closed by early termination of other tags.
                    ElementDisplay::Inline => (true, node.is_tag(tag)),
                    // Block tags may never be closed by other tags.
                    ElementDisplay::Block => (node.is_tag(tag), true),
                }
            };

            if close {
                self.node = parent.clone();
            }

            if end {
                break;
            }
        }
    }

    fn open_tag(&mut self, el: Element) {
        // We borrow and define like this because it's less cumbersome than dealing with
        // the borrow_mut() everywhere.
        let (parentable, literal, void) = {
            let mutel = self.node.borrow_mut();
            (mutel.can_parent(&el), mutel.is_litreal(), mutel.is_void())
        };

        if parentable {
            // Create a node and define a new reference to it immediately.
            let node = Node::new(el);
            let new_node = node.clone();
            // Append first reference
            self.node.append(node);
            // Set node to other reference
            self.node = new_node;
            return;
        } else if literal {
            return;
        } else if void {
            return;
        }

        unreachable!()
    }
}

mod tests {
    #[test]
    fn add_text_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[Token::Text("Foobar".to_owned())]);

        assert_eq!(
            parser.root.borrow().get_contents(),
            Some(&"Foobar".to_string())
        );
    }

    #[test]
    fn add_bold_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[
            Token::Tag("b".to_owned(), None),
            Token::Text("Foobar".to_owned()),
            Token::TagClose("b".to_owned()),
        ]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        match parser.node.first_child() {
            Some(child) => {
                assert_eq!(child.borrow().get_tag_name(), Some(&"b".to_string()));
                assert_eq!(child.borrow().get_contents(), Some(&"Foobar".to_string()));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn add_em_to_bold_and_early_terminate() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[
            Token::Tag("b".to_owned(), None),
            Token::Tag("i".to_owned(), None),
            Token::Text("Foobar".to_owned()),
            Token::TagClose("b".to_owned()),
        ]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        assert_eq!(parser.node, parser.root);
    }

    #[test]
    fn root_wont_close() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[Token::TagClose("quote".to_owned())]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        assert_eq!(parser.node, parser.root);
    }
}
