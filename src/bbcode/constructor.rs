use super::{Element, Smilies, Tag};
use rctree::Node;
use std::cell::RefMut;
use std::collections::HashMap;

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

    pub fn build(&self, mut node: Node<Element>) -> String {
        let mut output: String = String::new();

        // If we have children, loop through them.
        if node.has_children() {
            let mut contents: String = String::new();

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
                        contents.push_str(&self.build(child))
                    } else {
                        contents.push_str(&Self::sanitize(child.borrow().get_raw()));
                    }
                }
            }
            // No, so our contents must be handled literally.
            else {
                for child in node.children() {
                    contents.push_str(&Self::sanitize(child.borrow().get_raw()));
                }
            }

            let res = &self.element_contents(node.borrow_mut(), contents);
            output.push_str(&self.element_open(node.borrow_mut()));
            output.push_str(res);
        }
        // If we do not have children, add our text.
        else {
            let res = {
                let el = node.borrow_mut();
                &match el.get_contents() {
                    Some(contents) => {
                        self.element_contents(el, self.replace_emojis(Self::sanitize(contents)))
                    }
                    None => self.element_contents(el, String::new()),
                }
            };

            output.push_str(&self.element_open(node.borrow_mut()));
            output.push_str(res);
        }

        output.push_str(&self.element_close(node.borrow_mut()));

        output
    }

    fn element_open(&self, el: RefMut<Element>) -> String {
        use super::tag::*;

        if let Some(tag) = el.get_tag_name() {
            if !el.is_broken() {
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
            }
            // Always render broken tags as raw.
            else {
                el.to_open_str()
            }
        } else {
            String::new()
        }
    }

    fn element_contents(&self, el: RefMut<Element>, contents: String) -> String {
        if let Some(tag) = el.get_tag_name() {
            match Tag::get_by_name(tag) {
                Tag::Image => Tag::fill_img_tag(el, contents),
                Tag::Link => Tag::fill_url_tag(el, contents),
                _ => contents,
            }
        } else {
            contents
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
        let mut hits: u8 = 0;
        let mut hit_map: HashMap<u8, &String> = HashMap::with_capacity(self.smilies.count());

        for (code, replace_with) in self.smilies.iter() {
            if result.contains(code) {
                hit_map.insert(hits, replace_with);
                result = result.replace(code, &format!("\r{}", hits));
                hits += 1;
            }
        }

        for (hit, replace_with) in hit_map {
            result = result.replace(&format!("\r{}", hit), replace_with);
        }

        result
    }

    /// Sanitizes a char for HTML.
    pub fn sanitize(input: &str) -> String {
        // Some insane person did an extremely detailed benchmark of this.
        // https://lise-henry.github.io/articles/optimising_strings.html
        let len = input.len();
        let mut output: Vec<u8> = Vec::with_capacity(len * 4);

        for c in input.bytes() {
            // https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html
            match c {
                b'<' => output.extend_from_slice(b"&lt;"),
                b'>' => output.extend_from_slice(b"&gt;"),
                b'&' => output.extend_from_slice(b"&amp;"),
                b'\"' => output.extend_from_slice(b"&quot;"),
                b'\'' => output.extend_from_slice(b"&#x27;"),
                _ => output.push(c),
            }
        }

        unsafe { String::from_utf8_unchecked(output) }
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
        assert_eq!(con.build(ast), "Hello, world!");

        // Second pass
        let mut ast = Node::new(Element::new_root());
        ast.append(Node::new(Element::new_from_text("Foo, bar!")));

        assert_eq!(ast.children().count(), 1);
        assert_eq!(con.build(ast), "Foo, bar!");
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

        let out = con.build(ast);
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

        let out = con.build(ast);
        assert_eq!(out, "Hello, world!");
    }

    #[test]
    fn text_only() {
        use super::{Constructor, Element};
        use rctree::Node;

        let con = Constructor::new();
        let ast = Node::new(Element::new_from_text("Hello, world!"));
        let out = con.build(ast);

        assert_eq!(out, "Hello, world!");
    }
}
