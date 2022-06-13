use super::Element;
use std::cell::RefMut;
use url::Url;

impl super::Tag {
    pub fn open_url_tag(el: RefMut<Element>) -> String {
        let mut url: Option<Url> = None;

        if let Some(arg) = el.get_argument() {
            url = url_arg(arg);
            // TODO: Check for unfurl="true/false"
        }

        if url.is_none() {
            if let Some(content) = el.get_contents() {
                match Url::parse(content) {
                    Ok(curl) => url = Some(curl),
                    Err(_) => {}
                }
            }
        }

        match url {
            Some(url) => format!(
                "<a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\"{}\">",
                url.as_str()
            ),
            // If we have no content, we are broken.
            None => Self::open_broken_tag(el),
        }
    }
}

fn url_arg(input: &str) -> Option<Url> {
    let input = input.strip_prefix('=')?;

    match Url::parse(input) {
        Ok(url) => Some(url),
        Err(_) => None,
    }
}
