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
fn test_parse_linebreaks() {
    assert_eq!(
        bbcode_to_html("Line 1\r\nLine 2\nLine 3\r\n\r\nLine 5"),
        "Line 1<br />Line 2<br />Line 3<br /><br />Line 5"
    );
}
