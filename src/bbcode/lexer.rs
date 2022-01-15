use super::ast::Element;
use super::ast::GroupType;
use super::Instruction;
use phf::phf_map;
use rctree::Node;

/// Struct for lexing BbCode Instructions into an Element tree.
pub struct Lexer {
    pub current_node: Node<Element>,
    pub anchor: Node<Element>,
    pub next_text_as_arg: Option<fn(&mut Lexer, &str)>,
    pub ignore_tags: Option<&'static str>,
    pub ignore_formatting: bool,
    pub linebreaks_allowed: bool,
    pub preserve_empty: bool,
    /// A list of dependencies to be loaded before HTML construction.
    /// TODO: Make dependencies generic?
    pub attachments: Vec<i32>,
}

impl Lexer {
    /// Creates a new Lexer.
    pub fn new(preserve_empty: bool) -> Lexer {
        Lexer {
            anchor: Node::new(Element::new(GroupType::Anchor)),
            current_node: Node::new(Element::new(GroupType::Document)),
            next_text_as_arg: None,
            ignore_tags: None,
            ignore_formatting: false,
            linebreaks_allowed: true,
            preserve_empty,
            attachments: Vec::new(),
        }
    }

    /// Lexes a vector of Instructions.
    pub fn lex(&mut self, instructions: &[Instruction]) -> Node<Element> {
        self.anchor
            .append(Node::new(Element::new(GroupType::Document)));
        self.current_node = self.anchor.first_child().unwrap();
        self.new_group(GroupType::Paragraph);
        for instruction in instructions {
            self.execute(instruction);
        }
        self.end_group(GroupType::Paragraph);
        self.current_node.ancestors().last().unwrap()
    }

    /// Matches Instruction types.
    pub(crate) fn execute(&mut self, instruction: &Instruction) {
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

    /// Creates a new Element.
    pub(crate) fn new_group(&mut self, ele_type: GroupType) {
        self.current_node.append(Node::new(Element::new(ele_type)));
        self.current_node = self.current_node.last_child().unwrap();
    }

    // Closes groups when the current group is the target group.
    pub(crate) fn close_same_group(&mut self) {
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
                        || (self.current_node.borrow().is_kaput()
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
    pub(crate) fn close_diff_group(
        &mut self,
        group_stack: &mut Vec<GroupShorthand>,
        ele_type: GroupType,
    ) {
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
                    } else if let GroupType::Kaput(some_box, _) = my_type.clone() {
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
                                    || (self.current_node.borrow().is_kaput()
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
    pub(crate) fn reopen_groups(&mut self, group_stack: &mut Vec<GroupShorthand>) {
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
    pub(crate) fn end_and_new_group(&mut self, ele_type: GroupType, new_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow().ele_type() == &ele_type {
            self.close_same_group();
            self.new_group(new_type);
        } else if self.current_node.borrow().is_kaput() {
            let mut same = false;
            if let GroupType::Kaput(some_box, _) = self.current_node.borrow().ele_type().clone() {
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
    pub(crate) fn end_and_kill_new_group(&mut self, ele_type: GroupType, new_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow_mut().ele_type() == &ele_type {
            self.close_same_group();
            self.new_group(new_type);
        } else if self.current_node.borrow().is_kaput() {
            let mut same = false;
            if let GroupType::Kaput(some_box, _) = self.current_node.borrow().ele_type().clone() {
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
    pub(crate) fn end_group(&mut self, ele_type: GroupType) {
        if let Some(mut kid) = self.current_node.last_child() {
            if kid.borrow().ele_type() == &GroupType::Br {
                kid.detach();
            }
        }
        if self.current_node.borrow_mut().ele_type() == &ele_type {
            self.close_same_group();
        } else if self.current_node.borrow().is_kaput() {
            let mut same = false;
            if let GroupType::Kaput(some_box, _) = self.current_node.borrow().ele_type().clone() {
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
    pub(crate) fn parse_tag(&mut self, tag: &str, args: &Option<String>) {
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
    /// Consumes the input without doing anything.
    pub(crate) fn cmd_consume(&mut self) {
        // Intentionally empty.
    }
}

/// A simplified representation of an element used when closing and reopening groups.
pub struct GroupShorthand {
    pub ele_type: GroupType,
    pub arg: Option<String>,
}

/// Static compile-time map of tags without arguments to lexer commands.
static NO_ARG_CMD: phf::Map<&'static str, fn(&mut Lexer)> = phf_map! {
    "b" => Lexer::cmd_bold_open,
    "/b" => Lexer::cmd_bold_close,
    "i" => Lexer::cmd_italic_open,
    "/i" => Lexer::cmd_italic_close,
    "s" => Lexer::cmd_strikethrough_open,
    "/s" => Lexer::cmd_strikethrough_close,
    "strong" => Lexer::cmd_strong_open,
    "/strong" => Lexer::cmd_strong_close,
    "em" => Lexer::cmd_emphasis_open,
    "/em" => Lexer::cmd_emphasis_close,
    "u" => Lexer::cmd_underline_open,
    "/u" => Lexer::cmd_underline_close,
    "smcaps" => Lexer::cmd_smallcaps_open,
    "/smcaps" => Lexer::cmd_smallcaps_close,
    "mono" => Lexer::cmd_monospace_open,
    "/mono" => Lexer::cmd_monospace_close,
    "sub" => Lexer::cmd_subscript_open,
    "/sub" => Lexer::cmd_subscript_close,
    "sup" => Lexer::cmd_superscript_open,
    "/sup" => Lexer::cmd_superscript_close,
    "spoiler" => Lexer::cmd_spoiler_open,
    "/spoiler" => Lexer::cmd_spoiler_close,
    "hr" => Lexer::cmd_hr,
    "/hr" => Lexer::cmd_consume,
    "center" => Lexer::cmd_center_open,
    "/center" => Lexer::cmd_center_close,
    "right" => Lexer::cmd_right_open,
    "/right" => Lexer::cmd_right_close,
    "color" => Lexer::cmd_color_bare_open,
    "colour" => Lexer::cmd_color_bare_open,
    "/color" => Lexer::cmd_color_close,
    "/colour" => Lexer::cmd_color_close,
    "opacity" => Lexer::cmd_opacity_bare_open,
    "/opacity" => Lexer::cmd_opacity_close,
    "size" => Lexer::cmd_size_bare_open,
    "/size" => Lexer::cmd_size_close,
    "url" => Lexer::cmd_url_bare_open,
    "/url" => Lexer::cmd_url_close,
    "quote" => Lexer::cmd_quote_open,
    "/quote" => Lexer::cmd_quote_close,
    "code" => Lexer::cmd_code_open,
    "/code" => Lexer::cmd_code_close,
    "codeblock" => Lexer::cmd_codeblock_bare_open,
    "/codeblock" => Lexer::cmd_codeblock_close,
    "img" => Lexer::cmd_img_open,
    "/img" => Lexer::cmd_img_close,
    "plain" => Lexer::cmd_plain_open,
    "/plain" => Lexer::cmd_plain_close,
    "pre" => Lexer::cmd_pre_open,
    "/pre" => Lexer::cmd_pre_close,
    "footnote" => Lexer::cmd_footnote_bare_open,
    "/footnote" => Lexer::cmd_footnote_close,
    "list" => Lexer::cmd_list_bare_open,
    "/list" => Lexer::cmd_list_close,
    "*" => Lexer::cmd_list_item,
    "table" => Lexer::cmd_table_open,
    "/table" => Lexer::cmd_table_close,
    "tr" => Lexer::cmd_table_row_open,
    "/tr" => Lexer::cmd_table_row_close,
    "th" => Lexer::cmd_table_header_open,
    "/th" => Lexer::cmd_table_header_close,
    "td" => Lexer::cmd_table_data_open,
    "/td" => Lexer::cmd_table_data_close,
    "caption" => Lexer::cmd_table_caption_open,
    "/caption" => Lexer::cmd_table_caption_close,
    "pre-line" => Lexer::cmd_preline_open,
    "/pre-line" => Lexer::cmd_preline_close,
    "indent" => Lexer::cmd_indent_bare_open,
    "/indent" => Lexer::cmd_indent_close,
    "math" => Lexer::cmd_math_open,
    "/math" => Lexer::cmd_math_close,
    "mathblock" => Lexer::cmd_mathblock_open,
    "/mathblock" => Lexer::cmd_mathblock_close,
    "embed" => Lexer::cmd_embed_open,
    "/embed" => Lexer::cmd_embed_close,
    "email" => Lexer::cmd_email_open,
    "/email" => Lexer::cmd_email_close,
    "attach" => Lexer::cmd_attachment_open,
    "/attach" => Lexer::cmd_attachment_close,
};

/// Static compile-time map of tags with single arguments to lexer commands.
static ONE_ARG_CMD: phf::Map<&'static str, fn(&mut Lexer, &str)> = phf_map! {
    "color" => Lexer::cmd_color_open,
    "colour" => Lexer::cmd_color_open,
    "url" => Lexer::cmd_url_open,
    "opacity" => Lexer::cmd_opacity_open,
    "size" => Lexer::cmd_size_open,
    "quote" => Lexer::cmd_quote_arg_open,
    "codeblock" => Lexer::cmd_codeblock_open,
    "footnote" => Lexer::cmd_footnote_open,
    "list" => Lexer::cmd_list_open,
    "indent" => Lexer::cmd_indent_open,
};
