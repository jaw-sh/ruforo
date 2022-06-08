use super::*;

#[test]
fn test_parse_basic_tags() {
    assert_eq!(
        bbcode_to_html("Text is [i]italic[/i] and [b]bold[/b]!"),
        "Text is <i>italic</i> and <b>bold</b>!"
    );

    assert_eq!(
        bbcode_to_html("Text is [b]bold and [i]italics[/i][/b]!"),
        "Text is <b>bold and <i>italics</i></b>!"
    )
}

#[test]
fn test_parse_img_tags() {
    const IMG_URL: &str = "https://zombo.com/images/zombocom.png";

    // unitiatied
    assert_eq!(bbcode_to_html("[/img]"), "");
    // empty
    assert_eq!(bbcode_to_html("[img][/img]"), "");
    // unterminated
    assert_eq!(
        bbcode_to_html(&format!("[img]{}", IMG_URL)),
        format!("<img src=\"{}\" />", IMG_URL)
    );
    // terminated
    assert_eq!(
        bbcode_to_html(&format!("[img]{}[/img]", IMG_URL)),
        format!("<img src=\"{}\" />", IMG_URL)
    );
    // continued
    assert_eq!(
        bbcode_to_html(&format!("[img]{}[/img] [b]zombocom [i]!![/i][/b]", IMG_URL)),
        format!("<img src=\"{}\" /> <b>zombocom <i>!!</i></b>", IMG_URL)
    );
    // errored
    assert_eq!(
        bbcode_to_html(&format!("[img]{}[/img]", "this is not a url")),
        format!("[img]{}[/img]", "this is not a url")
    );
}

#[test]
fn test_parse_incomplete_tag_pairs() {
    assert_eq!(bbcode_to_html("[b]Text[/b]"), "<b>Text</b>");
    assert_eq!(bbcode_to_html("[b]Text"), "<b>Text</b>");
}

#[test]
fn test_parse_linebreaks() {
    assert_eq!(
        bbcode_to_html("Line 1\r\nLine 2\nLine 3\r\n\r\nLine 5"),
        "Line 1<br />Line 2<br />Line 3<br /><br />Line 5"
    );
}

#[test]
fn test_parse_url_tags() {
    const PAGE_URL: &str = "https://zombo.com/";

    assert_eq!(
        bbcode_to_html(&format!("[url]{}[/url]", PAGE_URL)),
        format!("<a href=\"{}\" rel=\"nofollow\">{}</a>", PAGE_URL, PAGE_URL)
    );
    assert_eq!(
        bbcode_to_html(&format!("[url]{}", PAGE_URL)),
        format!("<a href=\"{}\" rel=\"nofollow\">{}</a>", PAGE_URL, PAGE_URL)
    );
}
