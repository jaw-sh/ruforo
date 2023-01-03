use super::{Element, SafeHtml};
use std::cell::RefMut;
use url::Url;

impl super::Tag {
    pub fn open_img_tag(_: RefMut<Element>) -> SafeHtml {
        SafeHtml::new()
    }

    pub fn fill_img_tag(mut el: RefMut<Element>, contents: &str) -> SafeHtml {
        // Our URL comes from inside the tag.
        if let Ok(url) = Url::parse(contents) {
            match url.scheme() {
                "http" | "https" => {
                    el.clear_contents();
                    let sanitized_url = SafeHtml::sanitize(url.as_str());
                    let empty = SafeHtml::with_capacity(sanitized_url.len() + 32);
                    return empty + "<img src=\"" + &sanitized_url + "\" />";
                }
                _ => {}
            }
        }

        el.set_broken();
        SafeHtml::sanitize(contents)
    }

    pub fn open_url_tag(el: RefMut<Element>) -> SafeHtml {
        if el.is_broken() {
            el.to_open_str()
        } else {
            SafeHtml::new()
        }
    }

    pub fn fill_url_tag(mut el: RefMut<Element>, contents: &str, sanitized: SafeHtml) -> SafeHtml {
        let mut url: Option<Url> = None;

        if let Some(arg) = el.get_argument() {
            url = match url_arg(arg).transpose() {
                Ok(url) => url,
                Err(_) => {
                    el.set_broken();
                    return sanitized;
                }
            }
            // TODO: Check for unfurl="true/false"
        }

        if url.is_none() {
            if let Ok(curl) = Url::parse(contents) {
                url = Some(curl)
            }
        }

        match url {
            Some(url) => {
                let sanitized_url = SafeHtml::sanitize(url.as_str());
                let empty = SafeHtml::with_capacity(sanitized_url.len() + sanitized.len() + 64);
                empty
                    + "<a class=\"bbCode tagUrl\" ref=\"nofollow\" href=\""
                    + &sanitized_url
                    + "\">"
                    + &sanitized
            }
            // If we have no content, we are broken.
            None => {
                el.set_broken();
                sanitized
            }
        }
    }
}

fn url_arg(input: &str) -> Option<Result<Url, &str>> {
    let input = input.strip_prefix('=')?;

    match Url::parse(input) {
        Ok(url) => Some(match url.scheme() {
            "https" => Ok(url),
            "http" => Ok(url),
            _ => Err("Unsupported scheme"),
        }),
        Err(_) => None,
    }
}
