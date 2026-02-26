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
}

// --- Ditto custom tag ---

fn ditto_tag_def() -> CustomTagDef {
    CustomTagDef {
        name: "ditto".into(),
        tag_type: TagType::Inline,
        has_content: true,
        ..Default::default()
    }
}

struct DittoHandler;

impl CustomTagHandler for DittoHandler {
    fn tag_name(&self) -> &str {
        "ditto"
    }

    fn render(&self, tag: &TagNode, ctx: &RenderContext, output: &mut String) -> bool {
        output.push_str("<button class=\"tagDitto\">");
        ctx.render_children(tag, output);
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
        assert!(parse("[ditto][b]bold[/b][/ditto]").contains("<strong>bold</strong>"));
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
    fn smilies_not_in_code() {
        let mut smilies = HashMap::new();
        smilies.insert(":)".to_string(), "😊".to_string());

        let bbcode = ChatBBCode::new(smilies);
        let result = bbcode.render("[code]:)[/code]");
        assert!(!result.contains("😊"));
    }
}
