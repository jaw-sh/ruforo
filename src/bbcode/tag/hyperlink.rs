use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;
use url::Url;

impl Lexer {
    //
    // [url]
    //
    pub(crate) fn cmd_url_open_arg(&mut self, arg: &str) {
        if let Ok(url) = Url::parse(arg) {
            self.new_group(GroupType::Url);
            self.current_node.borrow_mut().set_arg(url.as_str());
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::Url), "url"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }

    pub(crate) fn cmd_url_open_bare(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_url_arg);
        self.new_group(GroupType::Url);
    }

    pub(crate) fn cmd_url_arg(&mut self, arg: &str) {
        if let Ok(url) = Url::parse(arg) {
            self.current_node.borrow_mut().set_arg(url.as_str());
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::Url), "url"));
            self.current_node.borrow_mut().set_arg(arg);
            return;
        }

        self.new_group(GroupType::Text);
        self.current_node.borrow_mut().add_text(arg);
        self.end_group(GroupType::Text);
    }

    pub(crate) fn cmd_url_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Url
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Kaput(Box::new(GroupType::Url), "url"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Url);
    }

    //
    // [email]
    //
    pub(crate) fn cmd_email_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_email_arg);
        self.new_group(GroupType::Email);
    }

    pub(crate) fn cmd_email_arg(&mut self, arg: &str) {
        self.current_node
            .borrow_mut()
            .set_arg(&format!("mailto:{}", arg));
        self.new_group(GroupType::Text);
        self.current_node.borrow_mut().add_text(arg);
        self.end_group(GroupType::Text);
    }

    pub(crate) fn cmd_email_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Email
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Kaput(Box::new(GroupType::Email), "email"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Email);
    }

    //
    // [img]
    //
    pub(crate) fn cmd_img_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_img_arg);
    }

    pub(crate) fn cmd_img_arg(&mut self, arg: &str) {
        if let Ok(url) = Url::parse(arg) {
            self.new_group(GroupType::Image);
            self.current_node.borrow_mut().set_void(true);
            self.current_node.borrow_mut().set_arg(url.as_str());
            self.end_group(GroupType::Image);
        } else {
            if self.current_node.borrow().ele_type() == &GroupType::Image {
                self.end_group(GroupType::Image);
            }

            self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
            self.current_node.borrow_mut().add_text(arg);
            self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
        }
    }

    pub(crate) fn cmd_img_close(&mut self) {
        if self.current_node.borrow().ele_type() == &GroupType::Image {
            if !self.current_node.borrow().has_arg() {
                self.current_node
                    .borrow_mut()
                    .set_ele_type(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                self.current_node.borrow_mut().set_detachable(false);
            }

            self.end_group(GroupType::Image);
        }
    }
}
