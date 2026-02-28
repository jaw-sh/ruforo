use bbcode::{
    CustomTagDef, CustomTagHandler, Parser, RenderConfig, Renderer, TagNode, RenderContext, TagType,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Quick parse for templates (no smilies, no custom config).
pub fn parse(input: &str) -> String {
    let mut parser = Parser::new();
    parser.register_custom_tag(ditto_tag_def());

    let doc = parser.parse(input);

    let mut renderer = Renderer::new();
    renderer.register_handler(Arc::new(DittoHandler));
    renderer.render(&doc)
}

/// Chat-optimized parser+renderer with smilies and custom tags.
pub struct ChatBBCode {
    parser: Parser,
    renderer: Renderer,
}

impl ChatBBCode {
    pub fn new(smilies: HashMap<String, String>) -> Self {
        let mut parser = Parser::new();
        parser.register_custom_tag(ditto_tag_def());

        let config = RenderConfig {
            smilies,
            ..Default::default()
        };
        let mut renderer = Renderer::with_config(config);
        renderer.register_handler(Arc::new(DittoHandler));

        Self { parser, renderer }
    }

    pub fn render(&self, input: &str) -> String {
        let doc = self.parser.parse(input);
        self.renderer.render(&doc)
    }

    pub fn sanitize(input: &str) -> String {
        bbcode::escape_html(input).into_owned()
    }

    /// Check if rendered HTML contains meaningful visible content.
    /// Delegates to `bbcode::has_visible_content`.
    pub fn has_visible_content(html: &str) -> bool {
        bbcode::has_visible_content(html)
    }
}

// --- Ditto custom tag ---

fn ditto_tag_def() -> CustomTagDef {
    CustomTagDef {
        name: "ditto".into(),
        tag_type: TagType::Verbatim,
        has_content: true,
        ..Default::default()
    }
}

struct DittoHandler;

impl CustomTagHandler for DittoHandler {
    fn tag_name(&self) -> &str {
        "ditto"
    }

    fn render(&self, tag: &TagNode, _ctx: &RenderContext, output: &mut String) -> bool {
        let text = tag.inner_text();
        output.push_str("<button class=\"tagDitto\">");
        output.push_str(&bbcode::escape_html(&text));
        output.push_str("</button>");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ditto() {
        assert!(parse("[ditto]click me[/ditto]").contains("tagDitto"));
        assert!(parse("[ditto]click me[/ditto]").contains("click me"));
        // Verbatim: inner BBCode is not processed
        assert!(parse("[ditto][b]bold[/b][/ditto]").contains("[b]bold[/b]"));
        assert!(!parse("[ditto][b]bold[/b][/ditto]").contains("<strong>"));
        assert!(parse("[ditto][/ditto]").contains("tagDitto"));
    }

    #[test]
    fn basic_tags() {
        assert_eq!("<strong>Test</strong>", parse("[b]Test[/b]"));
        assert_eq!("<em>Test</em>", parse("[i]Test[/i]"));
        assert_eq!("<u>Test</u>", parse("[u]Test[/u]"));
        assert_eq!("<s>Test</s>", parse("[s]Test[/s]"));
    }

    #[test]
    fn sanitize() {
        assert_eq!("&lt;b&gt;Test&lt;/b&gt;", parse("<b>Test</b>"));
    }

    #[test]
    fn smilies() {
        let mut smilies = HashMap::new();
        smilies.insert(":)".to_string(), "😊".to_string());
        smilies.insert("cookie".to_string(), "🍪".to_string());
        smilies.insert("ookie".to_string(), "🤢".to_string());

        let bbcode = ChatBBCode::new(smilies);
        let result = bbcode.render(":) I want a cookie!");
        assert!(result.contains("😊"));
        assert!(result.contains("🍪"));
        assert!(!result.contains("🤢")); // "ookie" in "cookie" should not double-match
    }

    #[test]
    fn code_verbatim() {
        // [code] should not parse inner BBCode
        let result = parse("[code][b]bold[/b][/code]");
        assert!(!result.contains("<strong>"));
        assert!(result.contains("[b]bold[/b]"));

        // Also test through ChatBBCode path (with smilies)
        let bbcode = ChatBBCode::new(HashMap::new());
        let result2 = bbcode.render("[code][b]bold[/b][/code]");
        assert!(!result2.contains("<strong>"));
        assert!(result2.contains("[b]bold[/b]"));

        // Sanitize preserves raw BBCode for edit
        assert_eq!(ChatBBCode::sanitize("[code][b]bold[/b][/code]"), "[code][b]bold[/b][/code]");

        // Unclosed [code] still treats remaining content as verbatim
        assert!(parse("[code][b]message").contains("[b]message"));
        assert!(!parse("[code][b]message").contains("<strong>"));
        assert!(parse("[code][b]message[/b]").contains("[b]message[/b]"));
        assert!(!parse("[code][b]message[/b]").contains("<strong>"));
    }

    #[test]
    fn visible_content() {
        // Delegates to bbcode::has_visible_content; thorough tests are in bbcode-rs
        assert!(!ChatBBCode::has_visible_content(&parse("[b][/b]")));
        assert!(ChatBBCode::has_visible_content(&parse("[b]test[/b]")));
        assert!(ChatBBCode::has_visible_content(&parse("[img]https://example.com/img.png[/img]")));
    }

    #[test]
    fn smilies_not_in_code() {
        let mut smilies = HashMap::new();
        smilies.insert(":)".to_string(), "😊".to_string());

        let bbcode = ChatBBCode::new(smilies);
        let result = bbcode.render("[code]:)[/code]");
        assert!(!result.contains("😊"));
    }
}
