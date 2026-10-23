use crate::error::Error;
use std::fmt;
use std::io::{self, Write};

pub const RESET: &str = "\x1b[0m";

// 8-bit ANSI codes; the order is the default order for uncolored categories.
const COLORS: [(&str, u8); 32] = [
    ("green", 2),
    ("red", 1),
    ("yellow", 3),
    ("blue", 4),
    ("magenta", 5),
    ("cyan", 6),
    ("grey", 7),
    ("black", 0),
    ("brightblack", 8),
    ("brightred", 9),
    ("brightgreen", 10),
    ("brightyellow", 11),
    ("brightblue", 12),
    ("brightmagenta", 13),
    ("brightcyan", 14),
    ("brightwhite", 15),
    ("24", 24),
    ("25", 25),
    ("26", 26),
    ("27", 27),
    ("28", 28),
    ("29", 29),
    ("30", 30),
    ("31", 31),
    ("32", 32),
    ("33", 33),
    ("34", 34),
    ("35", 35),
    ("36", 36),
    ("37", 37),
    ("38", 38),
    ("39", 39),
];

#[derive(Debug, Clone, Copy)]
pub struct Fg(pub u8);

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[38;5;{}m", self.0)
    }
}

pub fn resolve(names: &[String], categories: usize) -> Result<Vec<u8>, Error> {
    if names.is_empty() {
        return Ok(COLORS.iter().take(categories).map(|&(_, code)| code).collect());
    }
    if names.len() != categories {
        return Err(Error::ColorCount { given: names.len(), categories });
    }
    names
        .iter()
        .map(|name| {
            COLORS.iter().find(|(n, _)| n == name).map(|&(_, code)| code).ok_or_else(|| Error::InvalidColor {
                name: name.clone(),
                valid: COLORS.map(|(n, _)| n).join(", "),
            })
        })
        .collect()
}

pub fn write_table(out: &mut impl Write) -> io::Result<()> {
    for row in 0..32u8 {
        for col in 0..8u8 {
            let code = row * 8 + col;
            write!(out, "{}{code:>3} ▇▇▇▇▇{RESET}  ", Fg(code))?;
        }
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn defaults_follow_table_order() {
        assert_eq!(resolve(&[], 3).unwrap(), [2, 1, 3]);
    }

    #[test]
    fn named_colors_map_to_codes() {
        assert_eq!(resolve(&names(&["blue", "28"]), 2).unwrap(), [4, 28]);
    }

    #[test]
    fn count_mismatch_is_rejected() {
        assert!(matches!(resolve(&names(&["red"]), 2), Err(Error::ColorCount { given: 1, categories: 2 })));
    }

    #[test]
    fn unknown_color_lists_valid_names() {
        let Err(Error::InvalidColor { name, valid }) = resolve(&names(&["pink"]), 1) else {
            panic!("expected InvalidColor");
        };
        assert_eq!(name, "pink");
        assert!(valid.starts_with("green, red") && valid.ends_with("39"));
    }
}
