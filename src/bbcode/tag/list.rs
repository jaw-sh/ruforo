use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;
use phf::phf_set;

impl Lexer {
    pub(crate) fn cmd_list_bare_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::List);
        self.linebreaks_allowed = false;
    }
    pub(crate) fn cmd_list_open(&mut self, arg: &str) {
        if LIST_TYPES.contains(arg as &str) {
            self.end_and_new_group(GroupType::Paragraph, GroupType::List);
            self.current_node.borrow_mut().set_arg(arg);
            self.linebreaks_allowed = false;
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::List), "list"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }
    pub(crate) fn cmd_list_close(&mut self) {
        self.end_and_new_group(GroupType::List, GroupType::Paragraph);
        self.linebreaks_allowed = true;
    }
    pub(crate) fn cmd_list_item(&mut self) {
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
                self.new_group(GroupType::Kaput(Box::new(GroupType::ListItem), "*"));
                self.current_node.borrow_mut().set_void(true);
            }
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::ListItem), "*"));
            self.current_node.borrow_mut().set_void(true);
        }
    }
}

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
