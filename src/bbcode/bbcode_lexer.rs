use super::ASTElement;
use super::GroupType;
use super::Instruction;
use phf::{phf_map, phf_set};
use rctree::Node;

/// Struct for lexing BBCode Instructions into an ASTElement tree.
pub struct BBCodeLexer {
    current_node: Node<ASTElement>,
    anchor: Node<ASTElement>,
    next_text_as_arg: Option<fn(&mut BBCodeLexer, &str)>,
    ignore_tags: Option<&'static str>,
    ignore_formatting: bool,
    linebreaks_allowed: bool,
    preserve_empty: bool,
}

impl BBCodeLexer {
    /// Creates a new BBCodeLexer.
    pub fn new(preserve_empty: bool) -> BBCodeLexer {
        BBCodeLexer {
            anchor: Node::new(ASTElement::new(GroupType::Anchor)),
            current_node: Node::new(ASTElement::new(GroupType::Document)),
            next_text_as_arg: None,
            ignore_tags: None,
            ignore_formatting: false,
            linebreaks_allowed: true,
            preserve_empty,
        }
    }

    /// Lexes a vector of Instructions.
    pub fn lex(&mut self, instructions: &[Instruction]) -> Node<ASTElement> {
        self.anchor
            .append(Node::new(ASTElement::new(GroupType::Document)));
        self.current_node = self.anchor.first_child().unwrap();
        self.new_group(GroupType::Paragraph);
        for instruction in instructions {
            self.execute(instruction);
        }
        self.end_group(GroupType::Paragraph);
        self.current_node.ancestors().last().unwrap()
    }

    /// Matches Instruction types.
    fn execute(&mut self, instruction: &Instruction) {
        if let Some(arg_cmd) = self.next_text_as_arg {
            match instruction {
                Instruction::Text(param) => {
                    arg_cmd(self, param);
                }
                _ => {
                    self.next_text_as_arg = None;
                    self.execute(instruction);
                }
            }
        } else {
            match instruction {
                Instruction::Text(param) => {
                    self.new_group(GroupType::Text);
                    self.current_node.borrow_mut().add_text(&param);
                    self.end_group(GroupType::Text);
                }
                Instruction::Tag(param, arg) => {
                    if let Some(command) = self.ignore_tags {
                        if param == command {
                            self.parse_tag(&param, arg);
                        } else {
                            let tag_text = format!("[{}{}]", param, {
                                if let Some(argu) = arg {
                                    argu
                                } else {
                                    ""
                                }
                            });
                            self.new_group(GroupType::Text);
                            self.current_node.borrow_mut().add_text(&tag_text);
                            self.end_group(GroupType::Text);
                        }
                    } else {
                        self.parse_tag(&param, arg);
                    }
                }
                Instruction::Parabreak(param) => {
                    if self.ignore_formatting {
                        self.new_group(GroupType::Text);
                        self.current_node.borrow_mut().add_text(&param);
                        self.end_group(GroupType::Text);
                    } else {
                        self.end_and_new_group(GroupType::Paragraph, GroupType::Paragraph);
                    }
                }
                Instruction::Linebreak => {
                    if self.ignore_formatting {
                        self.new_group(GroupType::Text);
                        self.current_node.borrow_mut().add_text(&"\n".to_string());
                        self.end_group(GroupType::Text);
                    } else if self.linebreaks_allowed {
                        self.new_group(GroupType::Br);
                        self.current_node.borrow_mut().set_void(true);
                        self.end_group(GroupType::Br);
                    }
                }
                _ => {}
            }
        }
    }

    /// Creates a new ASTElement.
    fn new_group(&mut self, ele_type: GroupType) {
        self.current_node
            .append(Node::new(ASTElement::new(ele_type)));
        self.current_node = self.current_node.last_child().unwrap();
    }

    // Closes groups when the current group is the target group.
    fn close_same_group(&mut self) {
        match self.current_node.parent() {
            None => {}
            Some(parent) => {
                if !self.preserve_empty {
                    if (!self.current_node.has_children()
                        && (!self.ignore_formatting
                            && if let Some(text) = self.current_node.borrow().text_contents() {
                                text.trim().is_empty()
                            } else {
                                true
                            })
                        && !self.current_node.borrow().is_void()
                        && (self.current_node.borrow().is_detachable()))
                        || (self.current_node.borrow().is_broken()
                            && !self.current_node.has_children())
                    {
                        self.current_node.detach();
                    }
                } else if self.current_node.borrow().ele_type() == &GroupType::Paragraph
                    && !self.current_node.has_children()
                {
                    self.current_node.detach();
                }
                self.current_node = parent;
            }
        };
    }

    // Closes groups when the current group is not the target group.
    fn close_diff_group(&mut self, group_stack: &mut Vec<GroupShorthand>, ele_type: GroupType) {
        let mut go = true;
        while go {
            let my_type = self.current_node.borrow().ele_type().clone();
            match my_type {
                GroupType::Paragraph if ele_type != GroupType::Paragraph => {
                    go = false;
                    if !self.current_node.has_children() {
                        self.current_node.detach();
                    }
                }
                GroupType::List if ele_type != GroupType::List => {
                    go = false;
                }
                GroupType::Document if ele_type != GroupType::Document => {
                    go = false;
                }
                _ => {
                    if my_type == ele_type {
                        go = false;
                    } else if let GroupType::Broken(some_box, _) = my_type.clone() {
                        let unpacked_type = *some_box;
                        if unpacked_type == ele_type {
                            go = false;
                        } else if unpacked_type != GroupType::ListItem {
                            group_stack.push(GroupShorthand {
                                ele_type: my_type,
                                arg: self.current_node.borrow().argument().clone(),
                            });
                        }
                    } else {
                        group_stack.push(GroupShorthand {
                            ele_type: my_type,
                            arg: self.current_node.borrow().argument().clone(),
                        });
                    }

                    match self.current_node.parent() {
                        None => {
                            go = false;
                        }
                        Some(parent) => {
                            if !self.preserve_empty {
                                if (!self.current_node.has_children()
                                    && (!self.ignore_formatting
                                        && if let Some(text) =
                                            self.current_node.borrow().text_contents()
                                        {
                                            text.trim().is_empty()
                                        } else {
                                            true
                                        })
                                    && !self.current_node.borrow().is_void()
                                    && (self.current_node.borrow().is_detachable()))
                                    || (self.current_node.borrow().is_broken()
                                        && !self.current_node.has_children())
                                {
                                    self.current_node.detach();
                                }
                            } else if self.current_node.borrow().ele_type() == &GroupType::Paragraph
                                && !self.current_node.has_children()
                            {
                                self.current_node.detach();
                            }
                            self.current_node = parent;
                        }
                    };
                }
            }
        }
    }

    // Reopens closed groups after another element has closed.
    fn reopen_groups(&mut self, group_stack: &mut Vec<GroupShorthand>) {
        while !group_stack.is_empty() {
            let group = group_stack.pop().unwrap();
            self.new_group(group.ele_type.clone());
            if let Some(arg) = group.arg {
                self.current_node.borrow_mut().set_arg(&arg);
            }
        }
    }

    /// Moves current working node up to the current node's parent and then creates a new element,
    /// preserving the formatting from the previous.
    fn end_and_new_group(&mut self, ele_type: GroupType, new_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow().ele_type() == &ele_type {
            self.close_same_group();
            self.new_group(new_type);
        } else if self.current_node.borrow().is_broken() {
            let mut same = false;
            if let GroupType::Broken(some_box, _) = self.current_node.borrow().ele_type().clone() {
                let unpacked_type = *some_box;
                if unpacked_type == ele_type {
                    same = true;
                }
            }
            if same {
                self.close_same_group();
                self.new_group(new_type);
            } else {
                let mut group_stack = Vec::new();
                self.close_diff_group(&mut group_stack, ele_type);
                self.new_group(new_type);
                self.reopen_groups(&mut group_stack);
            }
        } else {
            let mut group_stack = Vec::new();
            self.close_diff_group(&mut group_stack, ele_type);
            self.new_group(new_type);
            self.reopen_groups(&mut group_stack);
        }
    }

    /// Moves current working node up to the current node's parent and then creates a new element,
    /// *without* preserving formatting from the previous element.
    fn end_and_kill_new_group(&mut self, ele_type: GroupType, new_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow_mut().ele_type() == &ele_type {
            self.close_same_group();
            self.new_group(new_type);
        } else if self.current_node.borrow().is_broken() {
            let mut same = false;
            if let GroupType::Broken(some_box, _) = self.current_node.borrow().ele_type().clone() {
                let unpacked_type = *some_box;
                if unpacked_type == ele_type {
                    same = true;
                }
            }
            if same {
                self.close_same_group();
                self.new_group(new_type);
            } else {
                let mut group_stack = Vec::new();
                self.close_diff_group(&mut group_stack, ele_type);
                self.new_group(new_type);
            }
        } else {
            let mut group_stack = Vec::new();
            self.close_diff_group(&mut group_stack, ele_type);
            self.new_group(new_type);
        }
    }

    /// Moves current working node up to the current node's parent.
    fn end_group(&mut self, ele_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow_mut().ele_type() == &ele_type {
            self.close_same_group();
        } else if self.current_node.borrow().is_broken() {
            let mut same = false;
            if let GroupType::Broken(some_box, _) = self.current_node.borrow().ele_type().clone() {
                let unpacked_type = *some_box;
                if unpacked_type == ele_type {
                    same = true;
                }
            }
            if same {
                self.close_same_group();
            } else {
                let mut group_stack = Vec::new();
                self.close_diff_group(&mut group_stack, ele_type);
                self.reopen_groups(&mut group_stack);
            }
        } else {
            let mut group_stack = Vec::new();
            self.close_diff_group(&mut group_stack, ele_type);
            self.reopen_groups(&mut group_stack);
        }
    }

    /// Parses tag Instructions.
    fn parse_tag(&mut self, tag: &str, args: &Option<String>) {
        match args {
            Some(primary_arg) => match ONE_ARG_CMD.get(tag) {
                Some(cmd) => cmd(self, primary_arg),
                None => self.execute(&Instruction::Text(format!("[{}={}]", tag, primary_arg))),
            },
            None => match NO_ARG_CMD.get(tag) {
                Some(cmd) => cmd(self),
                None => self.execute(&Instruction::Text(format!("[{}]", tag))),
            },
        }
    }

    /*-- COMMANDS --*/
    fn cmd_bold_open(&mut self) {
        self.new_group(GroupType::Bold);
    }
    fn cmd_bold_close(&mut self) {
        self.end_group(GroupType::Bold);
    }

    fn cmd_italic_open(&mut self) {
        self.new_group(GroupType::Italic);
    }
    fn cmd_italic_close(&mut self) {
        self.end_group(GroupType::Italic);
    }

    fn cmd_strong_open(&mut self) {
        self.new_group(GroupType::Strong);
    }
    fn cmd_strong_close(&mut self) {
        self.end_group(GroupType::Strong);
    }

    fn cmd_emphasis_open(&mut self) {
        self.new_group(GroupType::Emphasis);
    }
    fn cmd_emphasis_close(&mut self) {
        self.end_group(GroupType::Emphasis);
    }

    fn cmd_underline_open(&mut self) {
        self.new_group(GroupType::Underline);
    }
    fn cmd_underline_close(&mut self) {
        self.end_group(GroupType::Underline);
    }

    fn cmd_smallcaps_open(&mut self) {
        self.new_group(GroupType::Smallcaps);
    }
    fn cmd_smallcaps_close(&mut self) {
        self.end_group(GroupType::Smallcaps);
    }

    fn cmd_strikethrough_open(&mut self) {
        self.new_group(GroupType::Strikethrough);
    }
    fn cmd_strikethrough_close(&mut self) {
        self.end_group(GroupType::Strikethrough);
    }

    fn cmd_monospace_open(&mut self) {
        self.new_group(GroupType::Monospace);
    }
    fn cmd_monospace_close(&mut self) {
        self.end_group(GroupType::Monospace);
    }

    fn cmd_subscript_open(&mut self) {
        self.new_group(GroupType::Subscript);
    }
    fn cmd_subscript_close(&mut self) {
        self.end_group(GroupType::Subscript);
    }

    fn cmd_superscript_open(&mut self) {
        self.new_group(GroupType::Superscript);
    }
    fn cmd_superscript_close(&mut self) {
        self.end_group(GroupType::Superscript);
    }

    fn cmd_spoiler_open(&mut self) {
        self.new_group(GroupType::Spoiler);
    }
    fn cmd_spoiler_close(&mut self) {
        self.end_group(GroupType::Spoiler);
    }

    fn cmd_h1_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"1".to_string());
    }
    fn cmd_h1_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_h2_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"2".to_string());
    }
    fn cmd_h2_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_h3_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"3".to_string());
    }
    fn cmd_h3_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_h4_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"4".to_string());
    }
    fn cmd_h4_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_h5_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"5".to_string());
    }
    fn cmd_h5_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_h6_open(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Header);
        self.current_node.borrow_mut().set_arg(&"6".to_string());
    }
    fn cmd_h6_close(&mut self) {
        self.end_group(GroupType::Header);
        self.new_group(GroupType::Paragraph);
    }

    fn cmd_plain_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Plain);
        self.ignore_formatting = true;
    }
    fn cmd_plain_close(&mut self) {
        self.end_group(GroupType::Plain);
        self.ignore_formatting = false;
        self.new_group(GroupType::Plain);
    }

    fn cmd_pre_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Pre);
        self.ignore_formatting = true;
    }
    fn cmd_pre_close(&mut self) {
        self.end_group(GroupType::Pre);
        self.ignore_formatting = false;
        self.new_group(GroupType::Paragraph);
    }

    fn cmd_colour_open(&mut self, arg: &str) {
        if (arg.starts_with('#') && arg.len() == 7
            || arg.len() == 4
                && arg
                    .trim_start_matches('#')
                    .chars()
                    .all(|c| c.is_ascii_hexdigit()))
            || WEB_COLOURS.contains(arg)
        {
            self.new_group(GroupType::Colour);
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::Colour), "colour"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }
    fn cmd_colour_bare_open(&mut self) {
        self.new_group(GroupType::Broken(Box::new(GroupType::Colour), "colour"));
    }
    fn cmd_color_open(&mut self, arg: &str) {
        if (arg.starts_with('#') && arg.len() == 7
            || arg.len() == 4
                && arg
                    .trim_start_matches('#')
                    .chars()
                    .all(|c| c.is_ascii_hexdigit()))
            || WEB_COLOURS.contains(arg)
        {
            self.new_group(GroupType::Colour);
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::Colour), "color"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }
    fn cmd_color_bare_open(&mut self) {
        self.new_group(GroupType::Broken(Box::new(GroupType::Colour), "color"));
    }
    fn cmd_colour_close(&mut self) {
        self.end_group(GroupType::Colour);
    }

    fn cmd_url_bare_open(&mut self) {
        self.next_text_as_arg = Some(BBCodeLexer::cmd_url_arg);
        self.new_group(GroupType::Url);
    }
    fn cmd_url_arg(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    if self.current_node.borrow().ele_type() == &GroupType::Url {
                        self.current_node
                            .borrow_mut()
                            .set_ele_type(GroupType::Broken(Box::new(GroupType::Url), "url"));
                    } else {
                        self.new_group(GroupType::Broken(Box::new(GroupType::Url), "url"));
                    }
                    self.current_node.borrow_mut().add_text(arg);
                    return;
                }
            }
            self.current_node
                .borrow_mut()
                .set_arg(&format!("http://{}", arg));
        }
        self.new_group(GroupType::Text);
        self.current_node.borrow_mut().add_text(arg);
        self.end_group(GroupType::Text);
    }
    fn cmd_url_open(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.new_group(GroupType::Url);
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    self.new_group(GroupType::Broken(Box::new(GroupType::Url), "url"));
                    self.current_node.borrow_mut().set_arg(arg);
                    return;
                }
            }
            self.new_group(GroupType::Url);
            self.current_node
                .borrow_mut()
                .set_arg(&format!("http://{}", arg));
        }
    }
    fn cmd_url_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Url
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Broken(Box::new(GroupType::Url), "url"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Url);
    }

    fn cmd_email_open(&mut self) {
        self.next_text_as_arg = Some(BBCodeLexer::cmd_email_arg);
        self.new_group(GroupType::Email);
    }
    fn cmd_email_arg(&mut self, arg: &str) {
        self.current_node
            .borrow_mut()
            .set_arg(&format!("mailto:{}", arg));
        self.new_group(GroupType::Text);
        self.current_node.borrow_mut().add_text(arg);
        self.end_group(GroupType::Text);
    }
    fn cmd_email_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Email
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Broken(Box::new(GroupType::Email), "email"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Email);
    }

    fn cmd_img_open(&mut self) {
        self.next_text_as_arg = Some(BBCodeLexer::cmd_img_arg);
        self.new_group(GroupType::Image);
    }
    fn cmd_img_arg(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            if let Some(index) = arg.rfind('.') {
                if let Some(suffix) = arg.get(index..) {
                    if ACCEPTED_IMAGE_TYPES.contains(suffix) {
                        self.new_group(GroupType::Image);
                        self.current_node.borrow_mut().set_void(true);
                        self.current_node.borrow_mut().set_arg(arg);
                        self.end_group(GroupType::Image);
                    } else {
                        if self.current_node.borrow().ele_type() == &GroupType::Image {
                            self.end_group(GroupType::Image);
                        }
                        self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                        self.current_node.borrow_mut().add_text(arg);
                        self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    }
                } else {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                }
            } else {
                if self.current_node.borrow().ele_type() == &GroupType::Image {
                    self.end_group(GroupType::Image);
                }
                self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                self.current_node.borrow_mut().add_text(arg);
                self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
            }
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    return;
                }
            }
            if let Some(index) = arg.rfind('.') {
                if let Some(suffix) = arg.get(index..) {
                    if ACCEPTED_IMAGE_TYPES.contains(suffix) {
                        self.new_group(GroupType::Image);
                        self.current_node.borrow_mut().set_void(true);
                        self.current_node
                            .borrow_mut()
                            .set_arg(&format!("http://{}", arg));
                        self.end_group(GroupType::Image);
                    } else {
                        if self.current_node.borrow().ele_type() == &GroupType::Image {
                            self.end_group(GroupType::Image);
                        }
                        self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                        self.current_node.borrow_mut().add_text(arg);
                        self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    }
                } else {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                }
            } else {
                if self.current_node.borrow().ele_type() == &GroupType::Image {
                    self.end_group(GroupType::Image);
                }
                self.new_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
                self.current_node.borrow_mut().add_text(arg);
                self.end_group(GroupType::Broken(Box::new(GroupType::Image), "img"));
            }
        }
    }
    fn cmd_img_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Image
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Broken(Box::new(GroupType::Image), "img"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Image);
    }

    fn cmd_opacity_open(&mut self, arg: &str) {
        let mut divisor = 1.0;
        let arg_string;
        if arg.ends_with('%') {
            arg_string = arg.trim_end_matches('%');
            divisor = 100.0;
        } else {
            arg_string = arg;
        }
        match arg_string.parse::<f32>() {
            Ok(mut val) => {
                val /= divisor;
                if val < 0.0 {
                    val = 0.0;
                } else if val > 1.0 {
                    val = 1.0;
                }
                self.new_group(GroupType::Opacity);
                self.current_node.borrow_mut().set_arg(&val.to_string());
            }
            Err(_) => {
                self.new_group(GroupType::Broken(Box::new(GroupType::Opacity), "opacity"));
                self.current_node.borrow_mut().set_arg(arg);
            }
        }
    }
    fn cmd_opacity_bare_open(&mut self) {
        self.new_group(GroupType::Broken(Box::new(GroupType::Opacity), "opacity"));
    }
    fn cmd_opacity_close(&mut self) {
        self.end_group(GroupType::Opacity);
    }

    fn cmd_size_open(&mut self, arg: &str) {
        let mut divisor = 1.0;
        let arg_string;
        if arg.ends_with("em") {
            arg_string = arg.trim_end_matches("em");
        } else {
            arg_string = arg;
            divisor = 16.0;
        }
        match arg_string.parse::<f32>() {
            Ok(mut val) => {
                val /= divisor;
                if val < 0.5 {
                    val = 0.5;
                } else if val > 2.0 {
                    val = 2.0;
                }
                self.new_group(GroupType::Size);
                self.current_node.borrow_mut().set_arg(&val.to_string());
            }
            Err(_) => {
                self.new_group(GroupType::Broken(Box::new(GroupType::Size), "size"));
                self.current_node.borrow_mut().set_arg(arg);
            }
        }
    }
    fn cmd_size_bare_open(&mut self) {
        self.new_group(GroupType::Broken(Box::new(GroupType::Size), "size"));
    }
    fn cmd_size_close(&mut self) {
        self.end_group(GroupType::Size);
    }

    fn cmd_quote_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Quote);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_quote_arg_open(&mut self, arg: &str) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Quote);
        self.current_node.borrow_mut().set_arg(arg);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_quote_close(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.end_group(GroupType::Quote);
    }

    fn cmd_footnote_bare_open(&mut self) {
        self.new_group(GroupType::Footnote);
    }
    fn cmd_footnote_open(&mut self, arg: &str) {
        self.new_group(GroupType::Footnote);
        self.current_node.borrow_mut().set_arg(arg);
    }
    fn cmd_footnote_close(&mut self) {
        self.end_group(GroupType::Footnote);
    }

    fn cmd_code_open(&mut self) {
        self.ignore_tags = Some("/code");
        self.new_group(GroupType::Code);
    }
    fn cmd_code_close(&mut self) {
        self.end_group(GroupType::Code);
        self.ignore_tags = None;
    }

    fn cmd_codeblock_bare_open(&mut self) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::CodeBlock);
        self.ignore_tags = Some("/codeblock");
        self.ignore_formatting = true;
    }
    fn cmd_codeblock_open(&mut self, arg: &str) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::CodeBlock);
        self.ignore_tags = Some("/codeblock");
        self.ignore_formatting = true;
        self.current_node.borrow_mut().set_arg(arg);
    }
    fn cmd_codeblock_close(&mut self) {
        self.end_and_new_group(GroupType::CodeBlock, GroupType::Paragraph);
        self.ignore_tags = None;
        self.ignore_formatting = false;
    }

    fn cmd_figure_open(&mut self, arg: &str) {
        if arg == "right" || arg == "left" {
            self.end_and_new_group(GroupType::Paragraph, GroupType::Figure);
            self.current_node.borrow_mut().set_arg(arg);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::Figure), "figure"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }
    fn cmd_figure_close(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.end_and_new_group(GroupType::Figure, GroupType::Paragraph);
    }

    fn cmd_embed_open(&mut self) {
        self.next_text_as_arg = Some(BBCodeLexer::cmd_embed_arg);
        self.end_and_new_group(GroupType::Paragraph, GroupType::Embed);
        self.current_node.borrow_mut().set_void(true);
    }
    fn cmd_embed_arg(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    self.new_group(GroupType::Broken(Box::new(GroupType::Embed), "embed"));
                    self.current_node.borrow_mut().set_arg(arg);
                    return;
                }
            }
            self.current_node
                .borrow_mut()
                .set_arg(&format!("http://{}", arg));
        }
    }
    fn cmd_embed_close(&mut self) {
        self.end_and_new_group(GroupType::Embed, GroupType::Paragraph);
    }

    fn cmd_list_bare_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::List);
        self.linebreaks_allowed = false;
    }
    fn cmd_list_open(&mut self, arg: &str) {
        if LIST_TYPES.contains(arg as &str) {
            self.end_and_new_group(GroupType::Paragraph, GroupType::List);
            self.current_node.borrow_mut().set_arg(arg);
            self.linebreaks_allowed = false;
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::List), "list"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }
    fn cmd_list_close(&mut self) {
        self.end_and_new_group(GroupType::List, GroupType::Paragraph);
        self.linebreaks_allowed = true;
    }
    fn cmd_list_item(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::List {
            self.end_and_new_group(GroupType::ListItem, GroupType::ListItem);
            self.new_group(GroupType::Paragraph);
        } else if let Some(parent) = self.current_node.parent() {
            if parent.borrow().ele_type() == &GroupType::ListItem {
                if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
                    self.end_group(GroupType::Paragraph);
                }
                self.end_and_new_group(GroupType::ListItem, GroupType::ListItem);
                self.new_group(GroupType::Paragraph);
            } else {
                self.new_group(GroupType::Broken(Box::new(GroupType::ListItem), "*"));
                self.current_node.borrow_mut().set_void(true);
            }
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::ListItem), "*"));
            self.current_node.borrow_mut().set_void(true);
        }
    }

    fn cmd_table_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Table);
        self.linebreaks_allowed = false;
    }
    fn cmd_table_close(&mut self) {
        self.end_and_new_group(GroupType::Table, GroupType::Paragraph);
        self.linebreaks_allowed = true;
    }
    fn cmd_table_row_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::Table {
            self.new_group(GroupType::TableRow);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::TableRow), "tr"));
        }
    }
    fn cmd_table_row_close(&mut self) {
        self.end_group(GroupType::TableRow);
    }
    fn cmd_table_header_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::TableRow {
            self.new_group(GroupType::TableHeader);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::TableHeader), "th"));
        }
    }
    fn cmd_table_header_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableHeader);
    }
    fn cmd_table_data_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::TableRow {
            self.new_group(GroupType::TableData);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Broken(Box::new(GroupType::TableData), "td"));
        }
    }
    fn cmd_table_data_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableData);
    }
    fn cmd_table_caption_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::Table {
            self.new_group(GroupType::TableCaption);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Broken(
                Box::new(GroupType::TableCaption),
                "caption",
            ));
        }
    }
    fn cmd_table_caption_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableCaption);
    }

    fn cmd_math_open(&mut self) {
        self.new_group(GroupType::Math);
        self.ignore_tags = Some("/math");
        self.ignore_formatting = true;
    }
    fn cmd_math_close(&mut self) {
        self.end_group(GroupType::Math);
        self.ignore_tags = None;
        self.ignore_formatting = false;
    }

    fn cmd_mathblock_open(&mut self) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::MathBlock);
        self.ignore_tags = Some("/mathblock");
        self.ignore_formatting = true;
    }
    fn cmd_mathblock_close(&mut self) {
        self.ignore_tags = Some("/mathblock");
        self.ignore_formatting = false;
        self.end_and_new_group(GroupType::MathBlock, GroupType::Paragraph);
    }

    fn cmd_hr(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.new_group(GroupType::Hr);
        self.current_node.borrow_mut().set_void(true);
        self.end_group(GroupType::Hr);
        self.new_group(GroupType::Paragraph);
    }

    fn cmd_preline_open(&mut self) {
        self.new_group(GroupType::PreLine);
        self.ignore_formatting = true;
    }
    fn cmd_preline_close(&mut self) {
        self.ignore_formatting = false;
        self.end_group(GroupType::PreLine);
    }

    fn cmd_indent_open(&mut self, arg: &str) {
        match arg {
            "1" | "2" | "3" | "4" => {
                self.end_and_new_group(GroupType::Paragraph, GroupType::Indent);
                self.current_node.borrow_mut().set_arg(arg);
                self.new_group(GroupType::Paragraph);
            }
            _ => {
                self.new_group(GroupType::Broken(Box::new(GroupType::Indent), "indent"));
                self.current_node.borrow_mut().set_arg(arg);
            }
        }
    }
    fn cmd_indent_bare_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Indent);
        self.current_node.borrow_mut().set_arg(&"1".to_string());
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_indent_close(&mut self) {
        self.end_and_new_group(GroupType::Indent, GroupType::Paragraph);
    }

    fn cmd_center_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Center);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_center_close(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.end_and_new_group(GroupType::Center, GroupType::Paragraph);
    }

    fn cmd_right_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Right);
        self.new_group(GroupType::Paragraph);
    }
    fn cmd_right_close(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.end_and_new_group(GroupType::Right, GroupType::Paragraph);
    }
}
/// Static compile-time map of tags without arguments to lexer commands.
static NO_ARG_CMD: phf::Map<&'static str, fn(&mut BBCodeLexer)> = phf_map! {
    "b" => BBCodeLexer::cmd_bold_open,
    "/b" => BBCodeLexer::cmd_bold_close,
    "i" => BBCodeLexer::cmd_italic_open,
    "/i" => BBCodeLexer::cmd_italic_close,
    "s" => BBCodeLexer::cmd_strikethrough_open,
    "/s" => BBCodeLexer::cmd_strikethrough_close,
    "strong" => BBCodeLexer::cmd_strong_open,
    "/strong" => BBCodeLexer::cmd_strong_close,
    "em" => BBCodeLexer::cmd_emphasis_open,
    "/em" => BBCodeLexer::cmd_emphasis_close,
    "u" => BBCodeLexer::cmd_underline_open,
    "/u" => BBCodeLexer::cmd_underline_close,
    "smcaps" => BBCodeLexer::cmd_smallcaps_open,
    "/smcaps" => BBCodeLexer::cmd_smallcaps_close,
    "mono" => BBCodeLexer::cmd_monospace_open,
    "/mono" => BBCodeLexer::cmd_monospace_close,
    "sub" => BBCodeLexer::cmd_subscript_open,
    "/sub" => BBCodeLexer::cmd_subscript_close,
    "sup" => BBCodeLexer::cmd_superscript_open,
    "/sup" => BBCodeLexer::cmd_superscript_close,
    "spoiler" => BBCodeLexer::cmd_spoiler_open,
    "/spoiler" => BBCodeLexer::cmd_spoiler_close,
    "hr" => BBCodeLexer::cmd_hr,
    "center" => BBCodeLexer::cmd_center_open,
    "/center" => BBCodeLexer::cmd_center_close,
    "right" => BBCodeLexer::cmd_right_open,
    "/right" => BBCodeLexer::cmd_right_close,
    "color" => BBCodeLexer::cmd_color_bare_open,
    "colour" => BBCodeLexer::cmd_colour_bare_open,
    "/color" => BBCodeLexer::cmd_colour_close,
    "/colour" => BBCodeLexer::cmd_colour_close,
    "opacity" => BBCodeLexer::cmd_opacity_bare_open,
    "/opacity" => BBCodeLexer::cmd_opacity_close,
    "size" => BBCodeLexer::cmd_size_bare_open,
    "/size" => BBCodeLexer::cmd_size_close,
    "url" => BBCodeLexer::cmd_url_bare_open,
    "/url" => BBCodeLexer::cmd_url_close,
    "quote" => BBCodeLexer::cmd_quote_open,
    "/quote" => BBCodeLexer::cmd_quote_close,
    "code" => BBCodeLexer::cmd_code_open,
    "/code" => BBCodeLexer::cmd_code_close,
    "codeblock" => BBCodeLexer::cmd_codeblock_bare_open,
    "/codeblock" => BBCodeLexer::cmd_codeblock_close,
    "img" => BBCodeLexer::cmd_img_open,
    "/img" => BBCodeLexer::cmd_img_close,
    "h1" => BBCodeLexer::cmd_h1_open,
    "/h1" => BBCodeLexer::cmd_h1_close,
    "h2" => BBCodeLexer::cmd_h2_open,
    "/h2" => BBCodeLexer::cmd_h2_close,
    "h3" => BBCodeLexer::cmd_h3_open,
    "/h3" => BBCodeLexer::cmd_h3_close,
    "h4" => BBCodeLexer::cmd_h4_open,
    "/h4" => BBCodeLexer::cmd_h4_close,
    "h5" => BBCodeLexer::cmd_h5_open,
    "/h5" => BBCodeLexer::cmd_h5_close,
    "h6" => BBCodeLexer::cmd_h6_open,
    "/h6" => BBCodeLexer::cmd_h6_close,
    "plain" => BBCodeLexer::cmd_plain_open,
    "/plain" => BBCodeLexer::cmd_plain_close,
    "pre" => BBCodeLexer::cmd_pre_open,
    "/pre" => BBCodeLexer::cmd_pre_close,
    "footnote" => BBCodeLexer::cmd_footnote_bare_open,
    "/footnote" => BBCodeLexer::cmd_footnote_close,
    "/figure" => BBCodeLexer::cmd_figure_close,
    "list" => BBCodeLexer::cmd_list_bare_open,
    "/list" => BBCodeLexer::cmd_list_close,
    "*" => BBCodeLexer::cmd_list_item,
    "table" => BBCodeLexer::cmd_table_open,
    "/table" => BBCodeLexer::cmd_table_close,
    "tr" => BBCodeLexer::cmd_table_row_open,
    "/tr" => BBCodeLexer::cmd_table_row_close,
    "th" => BBCodeLexer::cmd_table_header_open,
    "/th" => BBCodeLexer::cmd_table_header_close,
    "td" => BBCodeLexer::cmd_table_data_open,
    "/td" => BBCodeLexer::cmd_table_data_close,
    "caption" => BBCodeLexer::cmd_table_caption_open,
    "/caption" => BBCodeLexer::cmd_table_caption_close,
    "pre-line" => BBCodeLexer::cmd_preline_open,
    "/pre-line" => BBCodeLexer::cmd_preline_close,
    "indent" => BBCodeLexer::cmd_indent_bare_open,
    "/indent" => BBCodeLexer::cmd_indent_close,
    "math" => BBCodeLexer::cmd_math_open,
    "/math" => BBCodeLexer::cmd_math_close,
    "mathblock" => BBCodeLexer::cmd_mathblock_open,
    "/mathblock" => BBCodeLexer::cmd_mathblock_close,
    "embed" => BBCodeLexer::cmd_embed_open,
    "/embed" => BBCodeLexer::cmd_embed_close,
    "email" => BBCodeLexer::cmd_email_open,
    "/email" => BBCodeLexer::cmd_email_close,
};
/// Static compile-time map of tags with single arguments to lexer commands.
static ONE_ARG_CMD: phf::Map<&'static str, fn(&mut BBCodeLexer, &str)> = phf_map! {
    "color" => BBCodeLexer::cmd_color_open,
    "colour" => BBCodeLexer::cmd_colour_open,
    "url" => BBCodeLexer::cmd_url_open,
    "opacity" => BBCodeLexer::cmd_opacity_open,
    "size" => BBCodeLexer::cmd_size_open,
    "quote" => BBCodeLexer::cmd_quote_arg_open,
    "codeblock" => BBCodeLexer::cmd_codeblock_open,
    "footnote" => BBCodeLexer::cmd_footnote_open,
    "figure" => BBCodeLexer::cmd_figure_open,
    "list" => BBCodeLexer::cmd_list_open,
    "indent" => BBCodeLexer::cmd_indent_open,
};
/// Static compile-time set of valid HTML web colours.
static WEB_COLOURS: phf::Set<&'static str> = phf_set! {
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgrey",
    "darkgreen",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "grey",
    "green",
    "greenyellow",
    "honeydew",
    "hotpink",
    "indianred ",
    "indigo ",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgrey",
    "lightgreen",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "transparant",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
    "Aliceblue",
    "Antiquewhite",
    "Aqua",
    "Aquamarine",
    "Azure",
    "Beige",
    "Bisque",
    "Black",
    "Blanchedalmond",
    "Blue",
    "Blueviolet",
    "Brown",
    "Burlywood",
    "Cadetblue",
    "Chartreuse",
    "Chocolate",
    "Coral",
    "Cornflowerblue",
    "Cornsilk",
    "Crimson",
    "Cyan",
    "Darkblue",
    "Darkcyan",
    "Darkgoldenrod",
    "Darkgray",
    "Darkgrey",
    "Darkgreen",
    "Darkkhaki",
    "Darkmagenta",
    "Darkolivegreen",
    "Darkorange",
    "Darkorchid",
    "Darkred",
    "Darksalmon",
    "Darkseagreen",
    "Darkslateblue",
    "Darkslategray",
    "Darkslategrey",
    "Darkturquoise",
    "Darkviolet",
    "Deeppink",
    "Deepskyblue",
    "Dimgray",
    "Dimgrey",
    "Dodgerblue",
    "Firebrick",
    "Floralwhite",
    "Forestgreen",
    "Fuchsia",
    "Gainsboro",
    "Ghostwhite",
    "Gold",
    "Goldenrod",
    "Gray",
    "Grey",
    "Green",
    "Greenyellow",
    "Honeydew",
    "Hotpink",
    "Indianred ",
    "Indigo ",
    "Ivory",
    "Khaki",
    "Lavender",
    "Lavenderblush",
    "Lawngreen",
    "Lemonchiffon",
    "Lightblue",
    "Lightcoral",
    "Lightcyan",
    "Lightgoldenrodyellow",
    "Lightgray",
    "Lightgrey",
    "Lightgreen",
    "Lightpink",
    "Lightsalmon",
    "Lightseagreen",
    "Lightskyblue",
    "Lightslategray",
    "Lightslategrey",
    "Lightsteelblue",
    "Lightyellow",
    "Lime",
    "Limegreen",
    "Linen",
    "Magenta",
    "Maroon",
    "Mediumaquamarine",
    "Mediumblue",
    "Mediumorchid",
    "Mediumpurple",
    "Mediumseagreen",
    "Mediumslateblue",
    "Mediumspringgreen",
    "Mediumturquoise",
    "Mediumvioletred",
    "Midnightblue",
    "Mintcream",
    "Mistyrose",
    "Moccasin",
    "Navajowhite",
    "Navy",
    "Oldlace",
    "Olive",
    "Olivedrab",
    "Orange",
    "Orangered",
    "Orchid",
    "Palegoldenrod",
    "Palegreen",
    "Paleturquoise",
    "Palevioletred",
    "Papayawhip",
    "Peachpuff",
    "Peru",
    "Pink",
    "Plum",
    "Powderblue",
    "Purple",
    "Rebeccapurple",
    "Red",
    "Rosybrown",
    "Royalblue",
    "Saddlebrown",
    "Salmon",
    "Sandybrown",
    "Seagreen",
    "Seashell",
    "Sienna",
    "Silver",
    "Skyblue",
    "Slateblue",
    "Slategray",
    "Slategrey",
    "Snow",
    "Springgreen",
    "Steelblue",
    "Tan",
    "Teal",
    "Thistle",
    "Tomato",
    "Turquoise",
    "Transparant",
    "Violet",
    "Wheat",
    "White",
    "Whitesmoke",
    "Yellow",
    "Yellowgreen"
};

/// Static compile-time set of accepted image types.
static ACCEPTED_IMAGE_TYPES: phf::Set<&'static str> = phf_set! {
    ".jpg",
    ".jpeg",
    ".pjpeg",
    ".pjp",
    ".jfif",
    ".png",
    ".apng",
    ".gif",
    ".bmp",
    //".svg", Dangerous!
    ".webp",
};

/// Static compile-time set of forbidden URL characters.
static FORBIDDEN_URL_CHARS: phf::Set<char> = phf_set! {
    ':',
    ';',
    '*',
    '#',
    '{',
    '}',
    '|',
    '^',
    '~',
    '[',
    ']',
    '`',
};

/// Static compile-time set of accepted list types.
static LIST_TYPES: phf::Set<&'static str> = phf_set! {
    "1",
    "a",
    "A",
    "i",
    "I",
    "circle",
    "square",
    "none",
};

/// A simplified representation of an element used when closing and reopening groups.
pub struct GroupShorthand {
    pub ele_type: GroupType,
    pub arg: Option<String>,
}
