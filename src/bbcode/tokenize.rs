use super::Token;
use nom::branch::{alt, permutation};
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::{consumed, map, peek, recognize, rest};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, tuple};
use nom::IResult;
use url::Url;

/// Tokenizer accepts string primitive and returns a Nom parser result.
pub fn tokenize<'a>(input: &'a str) -> IResult<&str, Vec<Token>> {
    many0(parse)(input)
}

/// Negotiates next token from several parsers.
fn parse(input: &str) -> IResult<&str, Token> {
    // Parser is in order of priority.
    alt((
        parse_linebreak,
        parse_tag_close,
        parse_tag_open,
        parse_url,
        parse_text,
        parse_text_literally,
    ))(input)
}

/// Anticipates a line ending, returns Token::Linebreak.
fn parse_linebreak(input: &str) -> IResult<&str, Token> {
    map(line_ending, Token::Linebreak)(input)
}

/// Anticipates a closing tag, returns Token::TagClose.
fn parse_tag_close(input: &str) -> IResult<&str, Token> {
    map(
        consumed(delimited(tag("[/"), alpha1, tag("]"))),
        |(raw, tag): (&str, &str)| (Token::TagClose(raw, tag)),
    )(input)
}

/// Anticipates an opening tag, returns Token::Tag.
fn parse_tag_open(input: &str) -> IResult<&str, Token> {
    let (input, (raw, between)) = consumed(delimited(tag("["), tag_and_argument, tag("]")))(input)?;

    // Token generated
    if let Ok((_, (_, (tag, arg)))) = token_from_argument(between) {
        Ok((input, Token::Tag(raw, tag, arg)))
    }
    // No token generated
    else {
        // Attempt to match leftover
        Ok((input, token_from_text(raw)))
    }
}

/// Returns text until the next terminator is seen, returns Token::Text or Token::Null.
fn parse_text(input: &str) -> IResult<&str, Token> {
    // Consume garbage text.
    let (leftover, garbage) = many0(char('\r'))(input)?;

    // Pull until end of line or next token.
    // Returns if there is no text.
    let (_, mut between) = parse_text_until_terminator(leftover)?;

    // Do this repeatedly untl we have found the earliest instance.
    while let Ok((_, betweener)) = parse_text_until_terminator(between) {
        if between != betweener {
            between = betweener;
        } else {
            break;
        }
    }

    map(take(garbage.len() + between.chars().count()), |s| {
        token_from_text(s)
    })(input)
}

fn parse_text_literally(input: &str) -> IResult<&str, Token> {
    map(parse_text_and_take_one_bracket, |s: &str| {
        token_from_text(s)
    })(input)
}

fn parse_text_and_take_one_bracket(input: &str) -> IResult<&str, &str> {
    let (_, (char, str)) = permutation((char('['), recognize(many0(none_of("\r\n[")))))(input)?;
    take(char.len_utf8() + str.len())(input)
}

fn parse_text_until_terminator(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(take_until1("http")),
        recognize(many1(none_of("\r\n["))),
    ))(input)
}

fn parse_url(input: &str) -> IResult<&str, Token> {
    peek(tag("http"))(input)?;
    let (input, url) = recognize(many1(none_of(" \r\n[>,")))(input)?;

    match Url::parse(url) {
        Ok(_) => Ok((input, Token::Url(url))),
        Err(_) => Ok((input, Token::Text(url))),
    }
}

fn tag_and_argument(input: &str) -> IResult<&str, &str> {
    recognize(many1(none_of("\r\n[]")))(input)
}

fn token_from_argument(input: &str) -> IResult<&str, (&str, (&str, Option<&str>))> {
    // returns (raw, (tag, args))
    alt((
        // Matches `url=x` and `url a=1`
        map(
            consumed(tuple((alpha1, rest))),
            |(raw, (tag, arg)): (&str, (&str, &str))| {
                (raw, (tag, if arg.len() > 0 { Some(arg) } else { None }))
            },
        ),
        // Matches `url` only
        map(alpha1, |tag: &str| (tag, (tag, None))),
    ))(input)

    //Token::Tag(tag, )
    //Token::Tag(s, None)
}

fn token_from_text(input: &str) -> Token {
    if input.len() > 0 {
        Token::Text(input)
    } else {
        Token::Null
    }
}

mod tests {
    #[test]
    fn empty() {
        use super::tokenize;

        let input = "";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn linebreaks() {
        use super::{tokenize, Token};

        let inputn = "Hello, world!\nWorld, hello!";
        let inputrn = "Hello, world!\r\nWorld, hello!";

        let tokens = tokenize(inputn).unwrap().1;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Text("Hello, world!"));
        assert_eq!(tokens[1], Token::Linebreak("\n"));
        assert_eq!(tokens[2], Token::Text("World, hello!"));

        let tokens = tokenize(inputrn).unwrap().1;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Text("Hello, world!"));
        assert_eq!(tokens[1], Token::Linebreak("\r\n"));
        assert_eq!(tokens[2], Token::Text("World, hello!"));
    }

    #[test]
    fn links() {
        use super::{tokenize, Token};

        let input = "Hello, this is a link: https://google.com, but it's [i]before[/i] a tag!";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::Text("Hello, this is a link: "));
        assert_eq!(tokens[1], Token::Url("https://google.com"));
        assert_eq!(tokens[2], Token::Text(", but it's "));
        assert_eq!(tokens[3], Token::Tag("[i]", "i", None));
        assert_eq!(tokens[4], Token::Text("before"));
        assert_eq!(tokens[5], Token::TagClose("[/i]", "i"));
        assert_eq!(tokens[6], Token::Text(" a tag!"));

        let input = "Hello, this is a [u]tag[/u], but it's before this https://google.com link!";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::Text("Hello, this is a "));
        assert_eq!(tokens[1], Token::Tag("[u]", "u", None));
        assert_eq!(tokens[2], Token::Text("tag"));
        assert_eq!(tokens[3], Token::TagClose("[/u]", "u"));
        assert_eq!(tokens[4], Token::Text(", but it's before this "));
        assert_eq!(tokens[5], Token::Url("https://google.com"));
        assert_eq!(tokens[6], Token::Text(" link!"));
    }

    #[test]
    fn tag_basic() {
        use super::{tokenize, Token};

        let input = "[b]Bold[/b]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Tag("[b]", "b", None));
        assert_eq!(tokens[1], Token::Text("Bold"));
        assert_eq!(tokens[2], Token::TagClose("[/b]", "b"));

        let input = "私は猫[i]です[/i]。";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Text("私は猫"));
        assert_eq!(tokens[1], Token::Tag("[i]", "i", None));
        assert_eq!(tokens[2], Token::Text("です"));
        assert_eq!(tokens[3], Token::TagClose("[/i]", "i"));
        assert_eq!(tokens[4], Token::Text("。"));
    }

    #[test]
    fn tag_double() {
        use super::{tokenize, Token};

        let input = "[b][i]Bold and Italic[/b]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Tag("[b]", "b", None));
        assert_eq!(tokens[1], Token::Tag("[i]", "i", None));
        assert_eq!(tokens[2], Token::Text("Bold and Italic"));
        assert_eq!(tokens[3], Token::TagClose("[/b]", "b"));
    }

    #[test]
    fn tag_with_arg() {
        use super::{tokenize, Token};

        let input = "[url=https://zombo.com]ZOMBO[/url]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens[0],
            Token::Tag("[url=https://zombo.com]", "url", Some("=https://zombo.com"))
        );
        assert_eq!(tokens[1], Token::Text("ZOMBO"));
        assert_eq!(tokens[2], Token::TagClose("[/url]", "url"));
    }

    #[test]
    fn tag_with_empty_arg() {
        use super::{tokenize, Token};

        let input = "[b ]";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Tag(input, "b", Some(" ")));

        let input = "[b=]";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Tag(input, "b", Some("=")));

        let input = "[b= ]";
        let tokens = tokenize(input).unwrap().1;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Tag(input, "b", Some("= ")));
    }

    #[test]
    fn tag_with_false_open() {
        use super::{tokenize, Token};

        let input = "[[b[b]Bold[/b]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Text("["));
        assert_eq!(tokens[1], Token::Text("[b"));
        assert_eq!(tokens[2], Token::Tag("[b]", "b", None));
        assert_eq!(tokens[3], Token::Text("Bold"));
        assert_eq!(tokens[4], Token::TagClose("[/b]", "b"));
    }

    #[test]
    fn tag_with_quoted_arg() {
        use super::{tokenize, Token};

        let input = "[url=\"https://zombo.com\"]ZOMBO[/url]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens[0],
            Token::Tag(
                "[url=\"https://zombo.com\"]",
                "url",
                Some("=\"https://zombo.com\"")
            )
        );
        assert_eq!(tokens[1], Token::Text("ZOMBO"));
        assert_eq!(tokens[2], Token::TagClose("[/url]", "url"));
    }

    #[test]
    fn tag_with_multi_arg() {
        use super::{tokenize, Token};

        let input = "[tag abc=123 xyz=\"000\"]Text[/tag]";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens[0],
            Token::Tag(
                "[tag abc=123 xyz=\"000\"]",
                "tag",
                Some(" abc=123 xyz=\"000\"")
            )
        );
        assert_eq!(tokens[1], Token::Text("Text"));
        assert_eq!(tokens[2], Token::TagClose("[/tag]", "tag"));
    }

    #[test]
    fn text() {
        use super::{tokenize, Token};

        let input = "Hello, world!";
        let tokens = tokenize(input).unwrap().1;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Text(input));
    }
}
