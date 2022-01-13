use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;

impl Lexer {
    pub(crate) fn cmd_table_open(&mut self) {
        self.end_and_new_group(GroupType::Paragraph, GroupType::Table);
        self.linebreaks_allowed = false;
    }
    pub(crate) fn cmd_table_close(&mut self) {
        self.end_and_new_group(GroupType::Table, GroupType::Paragraph);
        self.linebreaks_allowed = true;
    }

    pub(crate) fn cmd_table_row_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::Table {
            self.new_group(GroupType::TableRow);
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::TableRow), "tr"));
        }
    }
    pub(crate) fn cmd_table_row_close(&mut self) {
        self.end_group(GroupType::TableRow);
    }

    pub(crate) fn cmd_table_header_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::TableRow {
            self.new_group(GroupType::TableHeader);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::TableHeader), "th"));
        }
    }
    pub(crate) fn cmd_table_header_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableHeader);
    }

    pub(crate) fn cmd_table_data_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::TableRow {
            self.new_group(GroupType::TableData);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::TableData), "td"));
        }
    }
    pub(crate) fn cmd_table_data_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableData);
    }

    pub(crate) fn cmd_table_caption_open(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::Table {
            self.new_group(GroupType::TableCaption);
            self.new_group(GroupType::Paragraph);
        } else {
            self.new_group(GroupType::Kaput(
                Box::new(GroupType::TableCaption),
                "caption",
            ));
        }
    }
    pub(crate) fn cmd_table_caption_close(&mut self) {
        if self.current_node.borrow_mut().ele_type() == &GroupType::Paragraph {
            self.end_group(GroupType::Paragraph);
        }
        self.end_group(GroupType::TableCaption);
    }
}
