use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Conversion {
    Float,
    Integer,
}

/// A single printf conversion (`%[-+ 0][width][.precision]{f,d}`) with optional literal text around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    prefix: String,
    suffix: String,
    left: bool,
    plus: bool,
    space: bool,
    zero: bool,
    width: usize,
    precision: usize,
    conversion: Conversion,
}

impl Format {
    pub fn apply(&self, value: f64) -> String {
        let sign = if value.is_sign_negative() && !value.is_nan() {
            "-"
        } else if self.plus {
            "+"
        } else if self.space {
            " "
        } else {
            ""
        };
        let precision = match self.conversion {
            Conversion::Float => self.precision,
            Conversion::Integer => 0,
        };
        let digits = format!("{:.precision$}", value.abs());
        let pad = self.width.saturating_sub(sign.len() + digits.chars().count());
        let (prefix, suffix) = (&self.prefix, &self.suffix);
        if self.left {
            format!("{prefix}{sign}{digits}{}{suffix}", " ".repeat(pad))
        } else if self.zero {
            format!("{prefix}{sign}{}{digits}{suffix}", "0".repeat(pad))
        } else {
            format!("{prefix}{}{sign}{digits}{suffix}", " ".repeat(pad))
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        "%-5.2f".parse().expect("default format is valid")
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(spec: &str) -> Result<Self, String> {
        let (prefix, rest) = split_literal(spec);
        let rest = rest.ok_or_else(|| format!("'{spec}' has no conversion, expected e.g. %5.2f"))?;
        let (flags, rest) = take_while(rest, |c| "-+ 0".contains(c));
        let (width, rest) = take_while(rest, |c| c.is_ascii_digit());
        let (precision, rest) = match rest.strip_prefix('.') {
            Some(after) => take_while(after, |c| c.is_ascii_digit()),
            None => ("", rest),
        };
        let mut chars = rest.chars();
        let conversion = match chars.next() {
            Some('f') => Conversion::Float,
            Some('d') => Conversion::Integer,
            Some(c) => return Err(format!("unsupported conversion '%{c}', use f or d")),
            None => return Err(format!("'{spec}' ends inside a conversion")),
        };
        let (suffix, extra) = split_literal(chars.as_str());
        if extra.is_some() {
            return Err(format!("'{spec}' has more than one conversion"));
        }
        let number = |digits: &str, default| {
            if digits.is_empty() {
                Ok(default)
            } else {
                digits.parse().map_err(|e| format!("bad number in '{spec}': {e}"))
            }
        };
        Ok(Self {
            prefix,
            suffix,
            left: flags.contains('-'),
            plus: flags.contains('+'),
            space: flags.contains(' '),
            zero: flags.contains('0'),
            width: number(width, 0)?,
            precision: number(precision, 6)?,
            conversion,
        })
    }
}

fn take_while(s: &str, keep: impl Fn(char) -> bool) -> (&str, &str) {
    let end = s.find(|c| !keep(c)).unwrap_or(s.len());
    s.split_at(end)
}

// Returns the literal text up to the first '%' conversion (with "%%" unescaped) and what follows the '%'.
fn split_literal(s: &str) -> (String, Option<&str>) {
    let mut literal = String::new();
    let mut rest = s;
    while let Some(pos) = rest.find('%') {
        literal.push_str(&rest[..pos]);
        rest = &rest[pos + 1..];
        match rest.strip_prefix('%') {
            Some(after) => {
                literal.push('%');
                rest = after;
            }
            None => return (literal, Some(rest)),
        }
    }
    literal.push_str(rest);
    (literal, None)
}

/// Scales to K/M/B/T, rounded to three decimals, so 12550 becomes (12.55, "K").
pub fn readable(value: f64) -> (f64, &'static str) {
    const UNITS: [&str; 5] = ["", "K", "M", "B", "T"];
    let round = |v: f64| (v * 1000.0).round() / 1000.0;
    let (mut scaled, mut index) = (value.abs(), 0);
    // Rounding first, so 999.9996 becomes 1 K rather than 1000.
    while index < UNITS.len() - 1 && round(scaled) >= 1000.0 {
        scaled /= 1000.0;
        index += 1;
    }
    let rounded = round(scaled);
    (if value < 0.0 { -rounded } else { rounded }, UNITS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(spec: &str, value: f64) -> String {
        spec.parse::<Format>().unwrap().apply(value)
    }

    #[test]
    fn default_pads_to_five_columns() {
        assert_eq!(Format::default().apply(3.0), "3.00 ");
        assert_eq!(Format::default().apply(385.0), "385.00");
    }

    #[test]
    fn flags_width_and_precision() {
        assert_eq!(fmt("%8.1f", 3.15159), "     3.2");
        assert_eq!(fmt("%-8.1f|", 3.15159), "3.2     |");
        assert_eq!(fmt("%08.2f", -3.15159), "-0003.15");
        assert_eq!(fmt("%+.0f", 2.5), "+2");
    }

    #[test]
    fn integer_conversion_rounds() {
        assert_eq!(fmt("%d", 41.6), "42");
        assert_eq!(fmt("[%5d]", 7.0), "[    7]");
    }

    #[test]
    fn literal_text_and_escaped_percent() {
        assert_eq!(fmt("%.1f%%", 12.34), "12.3%");
        assert_eq!(fmt("~%.0f", 12.34), "~12");
    }

    #[test]
    fn invalid_specs_are_rejected() {
        for spec in ["", "plain", "%s", "%5", "%f%f", "%99999999999999999999f"] {
            assert!(spec.parse::<Format>().is_err(), "{spec}");
        }
    }

    #[test]
    fn readable_scales_units() {
        assert_eq!(readable(12550.0), (12.55, "K"));
        assert_eq!(readable(19_561_100.0), (19.561, "M"));
        assert_eq!(readable(1_000_000.0), (1.0, "M"));
        assert_eq!(readable(-2500.0), (-2.5, "K"));
        assert_eq!(readable(0.0), (0.0, ""));
        assert_eq!(readable(0.5), (0.5, ""));
        assert_eq!(readable(5e15), (5000.0, "T"));
        assert_eq!(readable(999.9996), (1.0, "K"));
        assert_eq!(readable(999_999.9), (1.0, "M"));
    }
}
