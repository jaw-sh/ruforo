use super::ASTElement;
use super::GroupType;
use rctree::{Node, NodeEdge};
use std::cell::Ref;

/// Struct for generation of HTML strings.
pub struct HTMLConstructor {
    output_string: String,
    pretty_print: bool,
}
impl HTMLConstructor {
    /// Creates a new HTMLConstructor.
    pub fn new(out_len: usize, pretty_print: bool) -> HTMLConstructor {
        let output_string = String::with_capacity(out_len + out_len / 2);
        HTMLConstructor {
            output_string,
            pretty_print,
        }
    }

    /// Generates an HTML string from an ASTElement
    pub fn construct(&mut self, ast: Node<ASTElement>) -> String {
        for node_edge in ast.traverse() {
            match node_edge {
                NodeEdge::Start(node) => self.start_element(node.borrow()),
                NodeEdge::End(node) => self.end_element(node.borrow()),
            }
        }
        self.output_string.clone()
    }

    /// Opens an HTML tag.
    fn start_element(&mut self, element: Ref<ASTElement>) {
        match element.ele_type() {
            GroupType::Text => {
                if let Some(text) = element.text_contents() {
                    self.output_string.push_str(text)
                }
            }
            //GroupType::Paragraph => self.output_string.push_str("<p>"),
            GroupType::Bold => self.output_string.push_str("<b>"),
            GroupType::Strong => self.output_string.push_str("<strong>"),
            GroupType::Italic => self.output_string.push_str("<i>"),
            GroupType::Emphasis => self.output_string.push_str("<em>"),
            GroupType::Underline => self.output_string.push_str("<span class=\"underline\">"),
            GroupType::Strikethrough => self.output_string.push_str("<s>"),
            GroupType::Smallcaps => self.output_string.push_str("<span class=\"smallcaps\">"),
            GroupType::Monospace => self.output_string.push_str("<span class=\"monospace\">"),
            GroupType::Subscript => self.output_string.push_str("<sub>"),
            GroupType::Superscript => self.output_string.push_str("<sup>"),
            GroupType::Spoiler => self.output_string.push_str("<span class=\"spoiler\">"),
            GroupType::Hr => self.output_string.push_str("<hr />"),
            GroupType::Br => self.output_string.push_str("<br />"),
            GroupType::Center => self.output_string.push_str("<div class=\"center\">"),
            GroupType::Right => self.output_string.push_str("<div class=\"right\">"),
            GroupType::Pre => self.output_string.push_str("<pre>"),
            GroupType::Code => self.output_string.push_str("<code>"),
            GroupType::Table => self.output_string.push_str("<table>"),
            GroupType::TableRow => self.output_string.push_str("<tr>"),
            GroupType::TableHeader => self.output_string.push_str("<th>"),
            GroupType::TableData => self.output_string.push_str("<td>"),
            GroupType::TableCaption => self.output_string.push_str("<caption>"),
            GroupType::Header => {
                if let Some(arg) = element.argument() {
                    self.output_string.push_str(&format!("<h{}>", arg));
                }
            }
            GroupType::Colour => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<span style=\"color:{};\">", arg));
                }
            }
            GroupType::Url => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<a href=\"{}\" rel=\"nofollow\">", arg));
                }
            }
            GroupType::Email => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<a href=\"{}\">", arg));
                }
            }
            GroupType::Opacity => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<span style=\"opacity:{};\">", arg));
                }
            }
            GroupType::Size => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<span style=\"font-size:{}rem;\">", arg));
                }
            }
            GroupType::Image => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<img src=\"{}\">", arg));
                }
            }
            GroupType::Figure => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<figure class=\"figure-{}\">", arg));
                }
            }
            GroupType::Quote => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<blockquote data-author=\"{}\">", arg));
                } else {
                    self.output_string.push_str(&"<blockquote>".to_string());
                }
            }
            GroupType::Footnote => {
                if let Some(arg) = element.argument() {
                    self.output_string.push_str(&format!(
                        "<span class=\"footnote\" data-symbol=\"{}\">",
                        arg
                    ));
                } else {
                    self.output_string
                        .push_str(&"<span class=\"footnote\">".to_string());
                }
            }
            GroupType::CodeBlock => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<pre data-language=\"{}\">", arg));
                } else {
                    self.output_string.push_str(&"<pre>".to_string());
                }
            }
            GroupType::List => {
                if let Some(arg) = element.argument() {
                    match arg as &str {
                        "1" | "a" | "A" | "i" | "I" => {
                            self.output_string
                                .push_str(&format!("<ol type=\"{}\">", arg));
                        }
                        "circle" | "square" | "none" => {
                            self.output_string
                                .push_str(&format!("<ul style=\"list-style-type:{};\">", arg));
                        }
                        _ => self.output_string.push_str("<ul>"),
                    }
                } else {
                    self.output_string.push_str("<ul>")
                }
            }
            GroupType::Indent => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<div class=\"indent-{}\">", arg));
                }
            }
            GroupType::ListItem => self.output_string.push_str("<li>"),
            GroupType::Math => self
                .output_string
                .push_str("<span class=\"math_container\">"),
            GroupType::MathBlock => self
                .output_string
                .push_str("<div class=\"math_container\">"),
            GroupType::Embed => {
                if let Some(arg) = element.argument() {
                    self.output_string
                        .push_str(&format!("<div class=\"embed\" data-content=\"{}\">", arg));
                }
            }
            GroupType::Kaput(_, tag) if !self.pretty_print => {
                if let Some(text) = element.text_contents() {
                    if let Some(arg) = element.argument() {
                        self.output_string
                            .push_str(&format!("[{}={}]{}", tag, arg, text));
                    } else {
                        self.output_string.push_str(&format!("[{}]{}", tag, text));
                    }
                } else if let Some(arg) = element.argument() {
                    self.output_string.push_str(&format!("[{}={}]", tag, arg));
                } else {
                    self.output_string.push_str(&format!("[{}]", tag));
                }
            }
            _ => {}
        };
    }

    /// Closes an HTML tag.
    fn end_element(&mut self, element: Ref<ASTElement>) {
        match element.ele_type() {
            //GroupType::Paragraph => self.output_string.push_str("</p>"),
            GroupType::Bold => self.output_string.push_str("</b>"),
            GroupType::Strong => self.output_string.push_str("</strong>"),
            GroupType::Italic => self.output_string.push_str("</i>"),
            GroupType::Emphasis => self.output_string.push_str("</em>"),
            GroupType::Subscript => self.output_string.push_str("</sub>"),
            GroupType::Superscript => self.output_string.push_str("</sup>"),
            GroupType::Strikethrough => self.output_string.push_str("</s>"),
            GroupType::Quote => self.output_string.push_str("</blockquote>"),
            GroupType::Code => self.output_string.push_str("</code>"),
            GroupType::Figure => self.output_string.push_str("</figure>"),
            GroupType::Table => self.output_string.push_str("</table>"),
            GroupType::TableRow => self.output_string.push_str("</tr>"),
            GroupType::TableHeader => self.output_string.push_str("</th>"),
            GroupType::TableData => self.output_string.push_str("</td>"),
            GroupType::TableCaption => self.output_string.push_str("</caption>"),
            GroupType::List => {
                if let Some(arg) = element.argument() {
                    match arg as &str {
                        "1" | "a" | "A" | "i" | "I" => self.output_string.push_str("</ol>"),
                        "circle" | "square" | "none" => self.output_string.push_str("</ul>"),
                        _ => self.output_string.push_str("</ul>"),
                    }
                } else {
                    self.output_string.push_str("</ul>")
                }
            }
            GroupType::ListItem => self.output_string.push_str("</li>"),
            GroupType::Header => {
                if let Some(arg) = element.argument() {
                    self.output_string.push_str(&format!("</h{}>", arg));
                }
            }
            GroupType::Url | GroupType::Email => self.output_string.push_str("</a>"),
            GroupType::Pre | GroupType::CodeBlock => self.output_string.push_str("</pre>"),
            GroupType::Underline
            | GroupType::Smallcaps
            | GroupType::Monospace
            | GroupType::Spoiler
            | GroupType::Colour
            | GroupType::Opacity
            | GroupType::Size
            | GroupType::Footnote
            | GroupType::Math => self.output_string.push_str("</span>"),
            GroupType::Center
            | GroupType::Right
            | GroupType::Indent
            | GroupType::MathBlock
            | GroupType::Embed => self.output_string.push_str("</div>"),
            GroupType::Kaput(_, tag) if !self.pretty_print => {
                if !element.is_void() {
                    self.output_string.push_str(&format!("[/{}]", tag));
                }
            }
            _ => {}
        };
    }
}
