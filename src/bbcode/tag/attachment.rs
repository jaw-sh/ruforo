use crate::bbcode::ast::Element;
use crate::bbcode::ast::GroupType;
use crate::bbcode::html_constructor::HTMLConstructor;
use crate::bbcode::lexer::Lexer;
use std::cell::Ref;

impl Lexer {
    pub(crate) fn cmd_attachment_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_attachment_arg);
        self.new_group(GroupType::Attachment);
    }

    pub(crate) fn cmd_attachment_arg(&mut self, arg: &str) {
        // Validate inbetween text as an integer.
        for c in arg.chars() {
            if !c.is_digit(10) {
                // Argument is invalid so register the tag as broken.
                if self.current_node.borrow().ele_type() == &GroupType::Attachment {
                    self.current_node
                        .borrow_mut()
                        .set_ele_type(GroupType::Kaput(Box::new(GroupType::Attachment), "attach"));
                } else {
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Attachment), "attach"));
                }
                self.current_node.borrow_mut().add_text(arg);
                return;
            }
        }

        self.attachments.push(arg.parse::<i32>().unwrap());
        self.current_node.borrow_mut().set_arg(arg);
    }

    pub(crate) fn cmd_attachment_close(&mut self) {
        self.end_group(GroupType::Attachment);
    }
}

impl HTMLConstructor {
    pub(crate) fn start_attach_element(&mut self, element: Ref<Element>) {
        if let Some(arg) = element.argument() {
            let id = arg.parse::<i32>().unwrap();
            for attachment in &self.prefetch_data {
                if attachment.id == id {
                    self.output_string.push_str(&attachment.to_html())
                }
            }
        } else {
            log::debug!("No arg.");
        }
    }
    pub(crate) fn end_attach_element(&mut self, element: Ref<Element>) {
        // TODO
    }
}
