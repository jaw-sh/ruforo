use std::fmt::{Display, Formatter, Result};

pub struct UrlToken<'a> {
    pub id: Option<i32>,
    pub name: String,
    pub base_url: &'a str,
    pub class: &'a str,
}

impl Display for UrlToken<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&get_link(self), f)
    }
}

/// Takes a Urlizable struct and returns an HTML string.
pub fn get_link(token: &UrlToken) -> String {
    if let Some(id) = token.id {
        format!(
            "<a class=\"{}\" href=\"{}\">{}</a>",
            token.class,
            format!("/{}/{}/", token.base_url, id),
            token.name
        )
    } else {
        format!("<span class=\"{}\">{}</span>", token.class, token.name)
    }
}
