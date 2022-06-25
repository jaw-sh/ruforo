use super::Element;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::alpha1,
    combinator::{all_consuming, verify},
    IResult,
};
use std::cell::RefMut;

impl super::Tag {
    pub fn open_color_tag(el: RefMut<Element>) -> String {
        if let Some(arg) = el.get_argument() {
            if let Ok((_, color)) = color_from(arg) {
                return format!(
                    "<span class=\"bbCode tagColor\" style=\"color: {}\">",
                    color
                );
            }
        }

        Self::open_broken_tag(el)
    }
}

//
// Lexer logic
//

fn color_from(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("=")(input)?;

    match color_hex(input) {
        Ok(result) => Ok(result),
        Err(_) => color_websafe(input),
    }
}

fn color_hex(input: &str) -> IResult<&str, &str> {
    let (color, _) = tag("#")(input)?;
    let (_, _) = all_consuming(take_while_m_n(6, 6, is_hex_digit))(color)?;

    Ok(("", input))
}

fn color_websafe(input: &str) -> IResult<&str, &str> {
    all_consuming(verify(alpha1, |s: &str| WEBSAFE_COLORS.contains(&s)))(input)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

const WEBSAFE_COLORS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgrey",
    "darkgreen",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "grey",
    "green",
    "greenyellow",
    "honeydew",
    "hotpink",
    "indianred ",
    "indigo ",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgrey",
    "lightgreen",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "transparant",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];
