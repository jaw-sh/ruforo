use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;

impl Lexer {
    pub(crate) fn cmd_code_open(&mut self) {
        self.ignore_tags = Some("/code");
        self.new_group(GroupType::Code);
    }
    pub(crate) fn cmd_code_close(&mut self) {
        self.end_group(GroupType::Code);
        self.ignore_tags = None;
    }

    pub(crate) fn cmd_codeblock_bare_open(&mut self) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::CodeBlock);
        self.ignore_tags = Some("/codeblock");
        self.ignore_formatting = true;
    }
    pub(crate) fn cmd_codeblock_open(&mut self, arg: &str) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::CodeBlock);
        self.ignore_tags = Some("/codeblock");
        self.ignore_formatting = true;
        self.current_node.borrow_mut().set_arg(arg);
    }
    pub(crate) fn cmd_codeblock_close(&mut self) {
        self.end_and_new_group(GroupType::CodeBlock, GroupType::Paragraph);
        self.ignore_tags = None;
        self.ignore_formatting = false;
    }

    pub(crate) fn cmd_footnote_bare_open(&mut self) {
        self.new_group(GroupType::Footnote);
    }
    pub(crate) fn cmd_footnote_open(&mut self, arg: &str) {
        self.new_group(GroupType::Footnote);
        self.current_node.borrow_mut().set_arg(arg);
    }
    pub(crate) fn cmd_footnote_close(&mut self) {
        self.end_group(GroupType::Footnote);
    }

    pub(crate) fn cmd_math_open(&mut self) {
        self.new_group(GroupType::Math);
        self.ignore_tags = Some("/math");
        self.ignore_formatting = true;
    }
    pub(crate) fn cmd_math_close(&mut self) {
        self.end_group(GroupType::Math);
        self.ignore_tags = None;
        self.ignore_formatting = false;
    }

    pub(crate) fn cmd_mathblock_open(&mut self) {
        self.end_and_kill_new_group(GroupType::Paragraph, GroupType::MathBlock);
        self.ignore_tags = Some("/mathblock");
        self.ignore_formatting = true;
    }
    pub(crate) fn cmd_mathblock_close(&mut self) {
        self.ignore_tags = Some("/mathblock");
        self.ignore_formatting = false;
        self.end_and_new_group(GroupType::MathBlock, GroupType::Paragraph);
    }

    pub(crate) fn cmd_plain_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Plain);
        self.ignore_formatting = true;
    }
    pub(crate) fn cmd_plain_close(&mut self) {
        self.end_group(GroupType::Plain);
        self.ignore_formatting = false;
        self.new_group(GroupType::Plain);
    }

    pub(crate) fn cmd_pre_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Pre);
        self.ignore_formatting = true;
    }
    pub(crate) fn cmd_pre_close(&mut self) {
        self.end_group(GroupType::Pre);
        self.ignore_formatting = false;
        self.new_group(GroupType::Paragraph);
    }

    pub(crate) fn cmd_quote_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Quote);
        self.new_group(GroupType::Paragraph);
    }
    pub(crate) fn cmd_quote_arg_open(&mut self, arg: &str) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Quote);
        self.current_node.borrow_mut().set_arg(arg);
        self.new_group(GroupType::Paragraph);
    }
    pub(crate) fn cmd_quote_close(&mut self) {
        self.end_group(GroupType::Paragraph);
        self.end_group(GroupType::Quote);
    }
}