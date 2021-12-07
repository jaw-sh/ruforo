use askama_actix::Template;
use std::ops::Range;

const PAGINATOR_LOOK_AHEAD: i32 = 2;

/// [1] 2 3 ... 13
/// 1 2 [3] 4 5 ... 13
/// 1 2 3 4 [5] 6 7 ... 13
/// 1 ... 4 5 [6] 7 8 ... 13
/// 1 ... 7 8 [9] 10 11 12 13
/// 1 ... 9 10 [11] 12 13
/// 1 ... 11 12 [13]
#[derive(Debug)]
pub struct Paginator {
    pub base_url: String,
    pub this_page: i32,
    pub page_count: i32,
}

#[derive(Template)]
#[template(path = "util/paginator.html")]
struct PaginatorTemplate<'a> {
    paginator: &'a Paginator,
}

pub trait PaginatorToHtml {
    fn as_html(&self) -> String;
    fn has_pages(&self) -> bool;
    fn get_first_pages(&self) -> Range<i32>;
    fn get_inner_pages(&self) -> Option<Range<i32>>;
    fn get_last_pages(&self) -> Option<Range<i32>>;
}

impl PaginatorToHtml for Paginator {
    fn has_pages(&self) -> bool {
        self.page_count > 1
    }

    fn get_first_pages(&self) -> Range<i32> {
        if 1 + PAGINATOR_LOOK_AHEAD < self.this_page - PAGINATOR_LOOK_AHEAD {
            // if 1+lookahead is less than page-lookahead, we only show page 1
            // i.e. any page starting with 6
            1..1
        } else if self.this_page + PAGINATOR_LOOK_AHEAD < self.page_count - PAGINATOR_LOOK_AHEAD {
            // if our lookahead is less than the lookbehind of the last page, show up to our lookahead.
            // i.e. on page 4 of 9, show 1-6 ... 9
            1..(self.this_page + PAGINATOR_LOOK_AHEAD)
        } else {
            // otherwise, just show all pages.
            // i.e. 5 of 9 is the greatest extent possible
            1..self.page_count
        }
    }

    fn get_inner_pages(&self) -> Option<Range<i32>> {
        if 1 + PAGINATOR_LOOK_AHEAD >= self.this_page - PAGINATOR_LOOK_AHEAD {
            // if 1+lookahead is less than page-lookahead, we only have first pages
            None
        } else if self.this_page + PAGINATOR_LOOK_AHEAD >= self.page_count - PAGINATOR_LOOK_AHEAD {
            // if our lookahead is gt/eq the lookbehind of the last page, we merge our cursor to the last pages
            None
        } else {
            // otherwise, show the lookahead and look behind
            // i.e. 1 .. 4 5 [6] 7 8 .. 11 (minimum number)
            Some((self.this_page - PAGINATOR_LOOK_AHEAD)..(self.this_page + PAGINATOR_LOOK_AHEAD))
        }
    }

    fn get_last_pages(&self) -> Option<Range<i32>> {
        if 1 + PAGINATOR_LOOK_AHEAD >= self.this_page - PAGINATOR_LOOK_AHEAD {
            // if 1+lookahead is less than page-lookahead, we only have first pages
            None
        } else if self.this_page + PAGINATOR_LOOK_AHEAD < self.page_count - PAGINATOR_LOOK_AHEAD {
            // if our lookahead is less than the lookbehind of the last page, show the last page
            Some(self.page_count..self.page_count)
        } else {
            // otherwise, show from the lookbehind of the cursor to the last page.
            Some((self.this_page - PAGINATOR_LOOK_AHEAD)..self.page_count)
        }
    }

    fn as_html(&self) -> String {
        if self.has_pages() {
            let mut buffer = String::new();
            let template = PaginatorTemplate { paginator: &self };
            if template.render_into(&mut buffer).is_err() {
                "[Paginator Util Error]".to_owned()
            } else {
                buffer
            }
        } else {
            "".to_owned()
        }
    }
}

#[derive(Template)]
#[template(path = "create_user.html")]
pub struct CreateUserTemplate<'a> {
    pub logged_in: bool,
    pub username: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub logged_in: bool,
    pub user_id: Option<i32>,
    pub username: Option<&'a str>,
    pub token: Option<&'a str>,
}
