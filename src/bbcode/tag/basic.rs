use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;

impl Lexer {
    pub(crate) fn cmd_bold_open(&mut self) {
        self.new_group(GroupType::Bold);
    }
    pub(crate) fn cmd_bold_close(&mut self) {
        self.end_group(GroupType::Bold);
    }

    pub(crate) fn cmd_italic_open(&mut self) {
        self.new_group(GroupType::Italic);
    }
    pub(crate) fn cmd_italic_close(&mut self) {
        self.end_group(GroupType::Italic);
    }

    pub(crate) fn cmd_strong_open(&mut self) {
        self.new_group(GroupType::Strong);
    }
    pub(crate) fn cmd_strong_close(&mut self) {
        self.end_group(GroupType::Strong);
    }

    pub(crate) fn cmd_emphasis_open(&mut self) {
        self.new_group(GroupType::Emphasis);
    }
    pub(crate) fn cmd_emphasis_close(&mut self) {
        self.end_group(GroupType::Emphasis);
    }

    pub(crate) fn cmd_underline_open(&mut self) {
        self.new_group(GroupType::Underline);
    }
    pub(crate) fn cmd_underline_close(&mut self) {
        self.end_group(GroupType::Underline);
    }

    pub(crate) fn cmd_smallcaps_open(&mut self) {
        self.new_group(GroupType::Smallcaps);
    }
    pub(crate) fn cmd_smallcaps_close(&mut self) {
        self.end_group(GroupType::Smallcaps);
    }

    pub(crate) fn cmd_strikethrough_open(&mut self) {
        self.new_group(GroupType::Strikethrough);
    }
    pub(crate) fn cmd_strikethrough_close(&mut self) {
        self.end_group(GroupType::Strikethrough);
    }

    pub(crate) fn cmd_monospace_open(&mut self) {
        self.new_group(GroupType::Monospace);
    }
    pub(crate) fn cmd_monospace_close(&mut self) {
        self.end_group(GroupType::Monospace);
    }

    pub(crate) fn cmd_subscript_open(&mut self) {
        self.new_group(GroupType::Subscript);
    }
    pub(crate) fn cmd_subscript_close(&mut self) {
        self.end_group(GroupType::Subscript);
    }

    pub(crate) fn cmd_superscript_open(&mut self) {
        self.new_group(GroupType::Superscript);
    }
    pub(crate) fn cmd_superscript_close(&mut self) {
        self.end_group(GroupType::Superscript);
    }

    pub(crate) fn cmd_spoiler_open(&mut self) {
        self.new_group(GroupType::Spoiler);
    }
    pub(crate) fn cmd_spoiler_close(&mut self) {
        self.end_group(GroupType::Spoiler);
    }
}
