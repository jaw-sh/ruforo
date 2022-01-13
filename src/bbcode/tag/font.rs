use crate::bbcode::ast::GroupType;
use crate::bbcode::lexer::Lexer;
use phf::phf_set;

impl Lexer {
    pub(crate) fn cmd_color_open(&mut self, arg: &str) {
        if (arg.starts_with('#') && arg.len() == 7
            || arg.len() == 4
                && arg
                    .trim_start_matches('#')
                    .chars()
                    .all(|c| c.is_ascii_hexdigit()))
            || WEB_COLOURS.contains(arg)
        {
            self.new_group(GroupType::Colour);
            self.current_node.borrow_mut().set_arg(arg);
        } else {
            self.new_group(GroupType::Kaput(Box::new(GroupType::Colour), "color"));
            self.current_node.borrow_mut().set_arg(arg);
        }
    }

    pub(crate) fn cmd_color_bare_open(&mut self) {
        self.new_group(GroupType::Kaput(Box::new(GroupType::Colour), "color"));
    }

    pub(crate) fn cmd_color_close(&mut self) {
        self.end_group(GroupType::Colour);
    }

    pub(crate) fn cmd_opacity_open(&mut self, arg: &str) {
        let mut divisor = 1.0;
        let arg_string;
        if arg.ends_with('%') {
            arg_string = arg.trim_end_matches('%');
            divisor = 100.0;
        } else {
            arg_string = arg;
        }
        match arg_string.parse::<f32>() {
            Ok(mut val) => {
                val /= divisor;
                if val < 0.0 {
                    val = 0.0;
                } else if val > 1.0 {
                    val = 1.0;
                }
                self.new_group(GroupType::Opacity);
                self.current_node.borrow_mut().set_arg(&val.to_string());
            }
            Err(_) => {
                self.new_group(GroupType::Kaput(Box::new(GroupType::Opacity), "opacity"));
                self.current_node.borrow_mut().set_arg(arg);
            }
        }
    }
    pub(crate) fn cmd_opacity_bare_open(&mut self) {
        self.new_group(GroupType::Kaput(Box::new(GroupType::Opacity), "opacity"));
    }
    pub(crate) fn cmd_opacity_close(&mut self) {
        self.end_group(GroupType::Opacity);
    }

    pub(crate) fn cmd_size_open(&mut self, arg: &str) {
        let mut divisor = 1.0;
        let arg_string;
        if arg.ends_with("em") {
            arg_string = arg.trim_end_matches("em");
        } else {
            arg_string = arg;
            divisor = 16.0;
        }
        match arg_string.parse::<f32>() {
            Ok(mut val) => {
                val /= divisor;
                if val < 0.5 {
                    val = 0.5;
                } else if val > 2.0 {
                    val = 2.0;
                }
                self.new_group(GroupType::Size);
                self.current_node.borrow_mut().set_arg(&val.to_string());
            }
            Err(_) => {
                self.new_group(GroupType::Kaput(Box::new(GroupType::Size), "size"));
                self.current_node.borrow_mut().set_arg(arg);
            }
        }
    }
    pub(crate) fn cmd_size_bare_open(&mut self) {
        self.new_group(GroupType::Kaput(Box::new(GroupType::Size), "size"));
    }
    pub(crate) fn cmd_size_close(&mut self) {
        self.end_group(GroupType::Size);
    }
}

/// Static compile-time set of valid HTML web colours.
static WEB_COLOURS: phf::Set<&'static str> = phf_set! {
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
};
