use super::{Element, SafeHtml, Smilies, Tag};
use rctree::Node;
use std::cell::RefMut;

/// Converts a Parser's AST into rendered HTML.
#[derive(Default)]
pub struct Constructor {
    // TODO: Build string here, return in build().
    pub smilies: Smilies,
}

impl Constructor {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn build(&self, mut node: Node<Element>) -> SafeHtml {
        let mut output = SafeHtml::new();

        // If we have children, loop through them.
        if node.has_children() {
            // Both sanitized HTML and raw strings must be maintained for one specific case: URLs.
            // For correctness, url encoding should be performed on the raw string and not the
            // sanitized one with HTML entities. Sanitization performed only after URL encoding
            let mut raw_contents = String::new();
            let mut safe_contents = SafeHtml::new();

            // Are we allowed to have children?
            if node.borrow().can_parent() {
                // Build each child node and append the string to our output.
                for child in node.children() {
                    // Sanity check on tag-in-tag logic.
                    let mut render = true;
                    // If we have a tag name, check if this tag can go into our parents.
                    if let Some(tag) = child.borrow().get_tag_name() {
                        // Check first if this node can accept this tag.
                        if node.borrow().can_parent_tag(tag) {
                            // Then, check each parent upwards.
                            let mut some_parent = node.parent();
                            while let Some(parent) = some_parent {
                                render = parent.borrow().can_parent_tag(tag);
                                if !render {
                                    break;
                                } else {
                                    some_parent = parent.parent();
                                }
                            }
                        } else {
                            render = false;
                        }
                    }

                    if render {
                        raw_contents.push_str(child.borrow().get_raw());
                        safe_contents.push(&self.build(child));
                    } else {
                        let item = child.borrow().get_raw();
                        raw_contents.push_str(item);
                        safe_contents.push(&SafeHtml::sanitize(item));
                    }
                }
            }
            // No, so our contents must be handled literally.
            else {
                for child in node.children() {
                    let item = child.borrow().get_raw();
                    raw_contents.push_str(item);
                    safe_contents.push(&SafeHtml::sanitize(item));
                }
            }

            let res = &self.element_contents(node.borrow_mut(), safe_contents, &raw_contents);
            output.push(&self.element_open(node.borrow_mut()));
            output.push(res);
        }
        // If we do not have children, add our text.
        else {
            let res = {
                let el = node.borrow_mut();
                &match el.get_contents() {
                    Some(contents) => {
                        let sanitized =
                            SafeHtml::sanitize_and_replace_smilies(contents, &self.smilies);
                        self.element_contents(el, sanitized, &contents)
                    }
                    None => self.element_contents(el, SafeHtml::new(), ""),
                }
            };

            output.push(&self.element_open(node.borrow_mut()));
            output.push(res);
        }

        output.push(&self.element_close(node.borrow_mut()));

        output
    }

    fn element_open(&self, el: RefMut<Element>) -> SafeHtml {
        use super::tag::*;

        if let Some(tag) = el.get_tag_name() {
            if !el.is_broken() {
                match Tag::get_by_name(tag) {
                    Tag::HorizontalRule => Tag::self_closing_tag("hr"),
                    Tag::Linebreak => Tag::self_closing_tag("br"),
                    Tag::Plain => SafeHtml::new(), // Not rendered.

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
            }
            // Always render broken tags as raw.
            else {
                el.to_open_str()
            }
        } else {
            SafeHtml::new()
        }
    }

    fn element_contents(
        &self,
        el: RefMut<Element>,
        safe_contents: SafeHtml,
        raw_contents: &str,
    ) -> SafeHtml {
        if let Some(tag) = el.get_tag_name() {
            match Tag::get_by_name(tag) {
                Tag::Image => Tag::fill_img_tag(el, raw_contents),
                Tag::Link => Tag::fill_url_tag(el, raw_contents, safe_contents),
                _ => safe_contents,
            }
        } else {
            safe_contents
        }
    }

    fn element_close(&self, el: RefMut<Element>) -> SafeHtml {
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
                    _ => SafeHtml::new(),
                }
            }
            // Broken tags reverse to original input.
            else {
                el.to_close_str()
            }
        }
        // Unnamed tags reverse to nothing.
        else {
            SafeHtml::new()
        }
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
        ast.append(Node::new(Element::new_from_text("Hello, world!")));

        assert_eq!(ast.children().count(), 1);
        assert_eq!(con.build(ast).take(), "Hello, world!");

        // Second pass
        let mut ast = Node::new(Element::new_root());
        ast.append(Node::new(Element::new_from_text("Foo, bar!")));

        assert_eq!(ast.children().count(), 1);
        assert_eq!(con.build(ast).take(), "Foo, bar!");
    }

    #[test]
    fn smilies() {
        use super::{Constructor, Element, Smilies};
        use rctree::Node;
        use std::collections::HashMap;

        let mut smilies: HashMap<String, String> = HashMap::default();
        smilies.insert(":c".to_string(), "☹️".to_string());
        smilies.insert("cookie".to_string(), "🍪".to_string());
        smilies.insert("ookie".to_string(), "🤢".to_string());

        let con = Constructor {
            smilies: Smilies::new_from_hashmap(&smilies),
        };

        let mut ast = Node::new(Element::new_root());
        ast.append(Node::new(Element::new_from_text(":c I want a cookie!")));

        let out = con.build(ast).take();
        assert_eq!(out, "☹️ I want a 🍪!");
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
        child.append(Node::new(Element::new_from_text("Hello, world!")));

        let out = con.build(ast).take();
        assert_eq!(out, "Hello, world!");
    }

    #[test]
    fn text_only() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();
        let ast = Node::new(Element::new_from_text("Hello, world!"));
        let out = con.build(ast).take();

        assert_eq!(out, "Hello, world!");
    }
}
