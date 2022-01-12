#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_basic_tags() {
        use ruforo::bbcode::bbcode_to_html;

        assert_eq!(
            bbcode_to_html("I'm [i]italic[/i] and [b]bold![/b]"),
            "<p>I&#x27m <i>italic</i> and <b>bold!</b></p>"
        );
    }
}
