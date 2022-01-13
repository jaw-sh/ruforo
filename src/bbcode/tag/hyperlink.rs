use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;
use phf::phf_set;

impl Lexer {
    pub(crate) fn cmd_url_bare_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_url_arg);
        self.new_group(GroupType::Url);
    }
    pub(crate) fn cmd_url_arg(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    if self.current_node.borrow().ele_type() == &GroupType::Url {
                        self.current_node
                            .borrow_mut()
                            .set_ele_type(GroupType::Kaput(Box::new(GroupType::Url), "url"));
                    } else {
                        self.new_group(GroupType::Kaput(Box::new(GroupType::Url), "url"));
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
    pub(crate) fn cmd_url_open(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.new_group(GroupType::Url);
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Url), "url"));
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

    pub(crate) fn cmd_embed_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_embed_arg);
        self.end_and_new_group(GroupType::Paragraph, GroupType::Embed);
        self.current_node.borrow_mut().set_void(true);
    }
    pub(crate) fn cmd_embed_arg(&mut self, arg: &str) {
        if arg.starts_with("https://") || arg.starts_with("http://") {
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Embed), "embed"));
                    self.current_node.borrow_mut().set_arg(arg);
                    return;
                }
            }
            self.current_node
                .borrow_mut()
                .set_arg(&format!("http://{}", arg));
        }
    }
    pub(crate) fn cmd_embed_close(&mut self) {
        self.end_and_new_group(GroupType::Embed, GroupType::Paragraph);
    }

    pub(crate) fn cmd_img_open(&mut self) {
        self.next_text_as_arg = Some(Lexer::cmd_img_arg);
        self.new_group(GroupType::Image);
    }
    pub(crate) fn cmd_img_arg(&mut self, arg: &str) {
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
                        self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                        self.current_node.borrow_mut().add_text(arg);
                        self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                    }
                } else {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                }
            } else {
                if self.current_node.borrow().ele_type() == &GroupType::Image {
                    self.end_group(GroupType::Image);
                }
                self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                self.current_node.borrow_mut().add_text(arg);
                self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
            }
        } else {
            for c in arg.chars() {
                if FORBIDDEN_URL_CHARS.contains(&c) {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
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
                        self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                        self.current_node.borrow_mut().add_text(arg);
                        self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                    }
                } else {
                    if self.current_node.borrow().ele_type() == &GroupType::Image {
                        self.end_group(GroupType::Image);
                    }
                    self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                    self.current_node.borrow_mut().add_text(arg);
                    self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                }
            } else {
                if self.current_node.borrow().ele_type() == &GroupType::Image {
                    self.end_group(GroupType::Image);
                }
                self.new_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
                self.current_node.borrow_mut().add_text(arg);
                self.end_group(GroupType::Kaput(Box::new(GroupType::Image), "img"));
            }
        }
    }
    pub(crate) fn cmd_img_close(&mut self) {
        if !self.current_node.borrow().has_arg()
            && self.current_node.borrow().ele_type() == &GroupType::Image
        {
            self.current_node
                .borrow_mut()
                .set_ele_type(GroupType::Kaput(Box::new(GroupType::Image), "img"));
            self.current_node.borrow_mut().set_detachable(false);
        }
        self.end_group(GroupType::Image);
    }
}

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
