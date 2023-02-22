use super::Smilies;
use std::collections::HashMap;
use std::ops;

/// Sanitizes a char for HTML.
pub fn sanitize(input: &str) -> String {
    // Some insane person did an extremely detailed benchmark of this.
    // https://lise-henry.github.io/articles/optimising_strings.html
    let len = input.len();
    let mut output: Vec<u8> = Vec::with_capacity(len * 4);

    for c in input.bytes() {
        // https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html
        match c {
            b'<' => output.extend_from_slice(b"&lt;"),
            b'>' => output.extend_from_slice(b"&gt;"),
            b'&' => output.extend_from_slice(b"&amp;"),
            b'\"' => output.extend_from_slice(b"&quot;"),
            b'\'' => output.extend_from_slice(b"&#x27;"),
            _ => output.push(c),
        }
    }

    unsafe { String::from_utf8_unchecked(output) }
}

/// Use the type system to help avoid XSS/script injection vulnerabilities. A SafeHtml string can
/// only be constructed from:
///  - a string which has been sanitized (i.e. <>&"' characters replaced with HTML entities)
///  - a literal string, enforced by the 'static lifetime annotation
///  - concatenations of already constructed SafeHtml strings
/// A SafeHtml string cannot be constructed from unsanitized user input, and this is enforced by
/// the type system. However, this does not guarantee safety against injection. You could
/// specify "<script>" as a string literal and it would be considered safe. But it is easier to
/// audit the string literals than to audit every string operation.
#[derive(Debug)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Return an empty SafeHtml string
    pub fn new() -> Self {
        Self(String::new())
    }

    /// Return an empty SafeHtml string with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Take a literal string and treat it as "safe" HTML. It is up to the user to make sure that
    /// it is truly safe.
    pub fn from_literal(trusted: &'static str) -> Self {
        Self(String::from(trusted))
    }

    /// Return a SafeHtml string with HTML characters escaped
    pub fn sanitize(input: &str) -> Self {
        Self(sanitize(input))
    }

    /// Return a SafeHtml string with HTML characters escaped and emojis replaced. The emoji replacement
    /// strings are implicitly considered safe (future TODO). It is important that sanitization and
    /// replacement are done as one step, because replacing them on arbitrary SafeHtml strings could cause
    /// them to become unsafe.
    pub fn sanitize_and_replace_smilies(input: &str, smilies: &Smilies) -> Self {
        let mut result = sanitize(input);
        let mut hits: u8 = 0;
        let mut hit_map: HashMap<u8, &String> = HashMap::with_capacity(smilies.count());

        for (code, replace_with) in smilies.iter() {
            if result.contains(code) {
                hit_map.insert(hits, replace_with);
                result = result.replace(code, &format!("\r{}", hits));
                hits += 1;
            }
        }

        for (hit, replace_with) in hit_map {
            result = result.replace(&format!("\r{}", hit), replace_with);
        }

        Self(result)
    }

    /// Concatenate self with the given SafeHtml string
    pub fn push(&mut self, string: &Self) {
        self.0.push_str(&string.0)
    }

    /// Concatenate self with the given literal string
    pub fn push_literal(&mut self, string: &'static str) {
        self.0.push_str(&string)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(self) -> String {
        self.0
    }
}

impl AsRef<str> for SafeHtml {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ops::Add<&SafeHtml> for SafeHtml {
    type Output = SafeHtml;

    fn add(mut self, rhs: &SafeHtml) -> SafeHtml {
        self.push(rhs);
        self
    }
}

impl ops::Add<&'static str> for SafeHtml {
    type Output = SafeHtml;

    fn add(mut self, rhs: &'static str) -> SafeHtml {
        self.push_literal(rhs);
        self
    }
}

mod tests {
    use super::super::Smilies;
    use super::SafeHtml;

    #[test]
    fn basic() {
        assert_eq!("", SafeHtml::new().as_ref());
        assert_eq!("", SafeHtml::with_capacity(100).as_ref());
        assert_eq!("<>&\'\"", SafeHtml::from_literal("<>&\'\"").as_ref());
    }

    #[test]
    fn escaped() {
        assert_eq!(
            "&lt;&gt;&amp;&#x27;&quot;",
            SafeHtml::sanitize("<>&\'\"").as_ref()
        );
    }

    #[test]
    fn concatenation() {
        let result = SafeHtml::from_literal("a<bc") + "xy<z" + "_<";
        assert_eq!("a<bcxy<z_<", result.as_ref());

        let user_input = SafeHtml::sanitize("x < y");
        let result = SafeHtml::from_literal("<b>") + &user_input + "</b>";
        assert_eq!("<b>x &lt; y</b>", result.as_ref());

        let mut result = SafeHtml::with_capacity(256);
        result = result + "<b>" + &user_input + "</b>";
        assert_eq!("<b>x &lt; y</b>", result.as_ref());
    }

    #[test]
    fn smilies_replacement() {
        let smilies = Smilies::new_from_tuples(vec![
            (":)".to_owned(), "&#x1F600;".to_owned()),
            (":mad:".to_owned(), "&#x1F620;".to_owned()),
            (":test:".to_owned(), "<test x=\"blah\" />".to_owned()),
        ]);
        assert_eq!(
            "&#x1F620;&#x1F600;",
            SafeHtml::sanitize_and_replace_smilies(":mad::)", &smilies).as_ref()
        );
        assert_eq!(
            "&lt;&amp;my&gt; &#x1F600;&quot;",
            SafeHtml::sanitize_and_replace_smilies("<&my> :)\"", &smilies).as_ref()
        );
        assert_eq!(
            "&lt;<test x=\"blah\" />&gt;",
            SafeHtml::sanitize_and_replace_smilies("<:test:>", &smilies).as_ref()
        );
    }
}
