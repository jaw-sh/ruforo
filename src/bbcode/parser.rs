use super::{Element, ElementDisplay, Token};
use rctree::Node;

/// Struct for parsing BbCode Tokens into an Element tree.
pub struct Parser<'str> {
    /// DOM root
    root: Node<Element<'str>>,
    /// Current traversal node.
    node: Node<Element<'str>>,
}

impl<'str> Parser<'str> {
    pub fn new() -> Self {
        // The rctree's Node<> is a modified RefCell, so cloning is just a ref.
        // See: https://docs.rs/rctree/latest/rctree/struct.Node.html#impl-Clone
        let root = Node::new(Element::new_root());
        let node = root.clone();

        Self { root, node }
    }

    pub fn parse(&mut self, tokens: &'str [Token]) -> Node<Element<'str>> {
        for token in tokens {
            match token {
                Token::Null => {
                    log::warn!("BbCode Lexer received Null instruction, which should not happen.");
                }
                Token::Linebreak(_) => self.add_linebreak(token),
                Token::Tag(_, _, _) => self.open_tag(token, Element::new_from_token(token)),
                Token::TagClose(_, tag) => self.close_tag(token, tag),
                Token::Text(text) => self.add_text(text),
                Token::Url(url) => self.add_url(token, url),
            }
        }

        // Cleanly close all open tags.
        while let Some(_) = self.node.parent() {
            self.close_open_tag(false);
        }
        self.insert_contents_as_node();

        // Unbind and return
        let ast = self.root.clone();
        self.root = Node::new(Element::new_root());
        self.node = self.root.clone();
        ast
    }

    fn add_linebreak(&mut self, token: &'str Token) {
        // If we can linebreak, add a <br />.
        if self.node.borrow().can_linebreak() {
            self.insert_element(Element::new_from_token(token));
        }
        // If we cannot linebreak but can have content, add a regular newline.
        else if self.node.borrow().can_have_content() {
            self.insert_element_broken(Element::new_from_token(token));
        }
        // Should not happen.
        else {
            unreachable!("Parser wanted to add new line to an element without breaks or content.");
        }
    }

    fn add_text(&mut self, text: &'str str) {
        if self.node.borrow().can_have_content() {
            self.insert_element(Element::new_from_text(text));
        } else {
            unreachable!();
        }
    }

    fn add_url(&mut self, token: &'str Token, _: &'str str) {
        self.insert_element(Element::new_from_token(token));
    }

    // Attempts to close the currently open tag.
    // If explicit is true, the user has explicitly closed this element.
    fn close_open_tag(&mut self, explicit: bool) {
        match self.node.parent() {
            Some(parent) => {
                // Set explicitly closed if we have.
                if explicit {
                    self.node.borrow_mut().set_explicit();
                }

                // Move content to a text node if we can parent.
                // In [b]foo[hr]bar[/b], this makes sure bar is in the right spot.
                // In [img]x[/img], a Plain tag, we capture the text content for parsing.
                if self.node.borrow().can_parent() {
                    self.insert_contents_as_node();
                }

                self.node = parent;
            }
            None => unreachable!(),
        };
    }

    /// Attempts to close tag, or all tags to tag we are closing.
    fn close_tag(&mut self, token: &'str Token, tag: &'str str) {
        let mut tag_matched = false;
        let mut closed_tags = 0;

        if tag.len() < 1 {
            log::warn!("Attempted to close a tag with no name.");
        }

        let mut cursor = Some(self.node.clone());
        while let Some(node) = cursor {
            {
                let el = node.borrow();

                // Check if this element is the same tag as what we're closing.
                tag_matched = el.is_tag(tag);

                // Handle nested closure depending on what this element is.
                if match el.get_display_type() {
                    // Inline tags may be closed by early termination of other tags.
                    ElementDisplay::Inline => true,
                    // Other tags may never be closed by other tags.
                    _ => tag_matched,
                } {
                    // Increment counter so we know how many parents we are moving up.
                    closed_tags += 1;
                }
                // Break if we can't close this.
                else {
                    break;
                }
            }

            // If we matched, we end the search now.
            if tag_matched {
                break;
            }

            // If we did not match, we can continue the search.
            cursor = node.parent();
        }

        // If we did not find the tag, we add the closing tag as text instead.
        if !tag_matched {
            self.insert_element(Element::new_from_token(token));
            return;
        }

        // Close all tags needed.
        while closed_tags > 0 {
            match self.node.parent() {
                Some(_) => self.close_open_tag(closed_tags == 1),
                None => unreachable!(),
            };
            closed_tags -= 1;
        }
    }

    fn insert_contents_as_node(&mut self) {
        // rctree will panic if you try DOM manipulation with borrowed elements.
        let el: Option<Element<'str>> = {
            let mut mutel = self.node.borrow_mut();
            mutel.extract_contents()
        };

        // Append text element, if it was created.
        if let Some(el) = el {
            self.node.append(Node::new(el));
        }
    }

    fn insert_element(&mut self, el: Element<'str>) -> Node<Element<'str>> {
        self.insert_contents_as_node();

        // Append the linebreak itself, if we can.
        let node = Node::new(el);
        self.node.append(node.clone());

        node
    }

    fn insert_element_broken(&mut self, el: Element<'str>) -> Node<Element<'str>> {
        let mut node = self.insert_element(el);

        node.borrow_mut().set_broken();

        node
    }

    /// Attempts to add element as child to current node and move current node to new element.
    fn open_tag(&mut self, token: &'str Token, el: Element<'str>) {
        // Can this tag parent any other element?
        if self.node.borrow().can_parent() {
            //  Can the new tag have any content at all?
            if el.can_have_content() {
                // Insert the element and move our pointer.
                self.node = self.insert_element(el);
            } else {
                // If we are inserting a void element, insert then do not move pointer.
                self.insert_element(el);
            }
        }
        // No, so it is a Literal tag like [code][/code].
        else if self.node.borrow().can_have_content() {
            self.add_text(token.as_raw());
        }
        // This is a self-closing tag, like [img]. Try to insert as an argument.
        else {
            // Do we already have contents?
            if !self.node.borrow().has_argument() {
                self.node.borrow_mut().set_argument(token.as_raw());
            }
            // Argument present, so the node is just broken.
            else {
                self.node.borrow_mut().set_broken();
                self.add_text(token.as_raw());
            }
        }
    }
}

mod tests {
    #[test]
    fn add_text_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[Token::Text("Foobar")]);

        assert_eq!(
            ast.first_child().unwrap().borrow().get_contents(),
            Some("Foobar")
        );
    }

    #[test]
    fn add_bold_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[
            Token::Tag("[b]", "b", None),
            Token::Text("Foobar"),
            Token::TagClose("[b]", "b"),
        ]);

        assert_eq!(ast.borrow().get_contents(), None);
        match ast.first_child() {
            Some(child) => {
                assert_eq!(child.borrow().get_tag_name(), Some("b"));
                match child.first_child() {
                    Some(child) => {
                        assert_eq!(child.borrow().get_contents(), Some("Foobar"));
                    }
                    None => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn add_em_to_bold_and_early_terminate() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[
            Token::Tag("[b]", "b", None),
            Token::Tag("[i]", "i", None),
            Token::Text("Foobar"),
            Token::TagClose("[b]", "b"),
        ]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        assert_eq!(parser.node, parser.root);
    }

    #[test]
    fn reusable() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[
            Token::Text("foo"),
            Token::Linebreak("\n\r"),
            Token::Text("bar"),
        ]);

        assert_eq!(ast.children().count(), 3);
        assert_eq!(
            ast.children().nth(0).unwrap().borrow().get_contents(),
            Some("foo")
        );

        // re-use it
        let ast = parser.parse(&[
            Token::Text("bar"),
            Token::Linebreak("\n\r"),
            Token::Text("foo"),
        ]);

        assert_eq!(ast.children().count(), 3);
        assert_eq!(
            ast.children().nth(0).unwrap().borrow().get_contents(),
            Some("bar")
        );

        // once more for good luck
        let ast = parser.parse(&[
            Token::Text("fris"),
            Token::Linebreak("\n\r"),
            Token::Text("bee"),
        ]);

        assert_eq!(ast.children().count(), 3);
        assert_eq!(
            ast.children().nth(2).unwrap().borrow().get_contents(),
            Some("bee")
        );
    }

    #[test]
    fn root_linebreak() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[
            Token::Tag("[b]", "b", None),
            Token::Text("a"),
            Token::Linebreak("\n\r"),
            Token::Text("b"),
            Token::TagClose("[b]", "b"),
        ]);

        let children = ast.children();
        assert_eq!(children.count(), 1);

        let b = ast.children().nth(0).unwrap();

        assert_eq!(b.borrow().get_tag_name(), Some("b"));
        assert_eq!(b.borrow().is_explicit(), true);
        assert_eq!(b.borrow().is_broken(), false);

        let children = b.children();
        assert_eq!(children.count(), 3);

        let mut children = b.children();
        assert_eq!(children.nth(0).unwrap().borrow().get_contents(), Some("a"));

        let mut children = b.children();
        assert_eq!(children.nth(1).unwrap().borrow().can_have_content(), false);

        let mut children = b.children();
        assert_eq!(children.nth(2).unwrap().borrow().get_contents(), Some("b"));
    }

    #[test]
    fn root_wont_close() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[Token::TagClose("[/quote]", "quote")]);
        let el = ast.first_child().unwrap();

        assert_eq!(el.borrow().get_tag_name(), None);
        assert_eq!(el.borrow().get_contents(), Some("[/quote]"));
        assert_eq!(ast.borrow().is_broken(), false);
        assert_eq!(parser.node, parser.root);
    }
}
