use crate::data::Dataset;
use crate::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};
use std::path::Path;

/// Reads from `path`, or from stdin for `None` and `-`. A terminal on stdin counts as no input.
pub fn read(path: Option<&Path>, delim: &str) -> Result<Dataset, Error> {
    match path.filter(|p| *p != Path::new("-")) {
        Some(path) => {
            let file = File::open(path).map_err(|source| Error::ReadInput { path: path.into(), source })?;
            parse(BufReader::new(file), delim)
        }
        None if io::stdin().is_terminal() => Ok(Dataset::default()),
        None => parse(io::stdin().lock(), delim),
    }
}

pub fn parse(reader: impl BufRead, delim: &str) -> Result<Dataset, Error> {
    let mut data = Dataset::default();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols = split(line, delim);
        if is_header(&cols, &data) {
            data.categories = cols[1..].iter().map(|c| unquote(c).to_string()).collect();
            continue;
        }
        let values = cols[1..].iter().map(|col| value(col, line)).collect::<Result<_, _>>()?;
        data.labels.push(unquote(cols[0]).to_string());
        data.rows.push(values);
    }
    check_rows(&data)?;
    Ok(data)
}

// The header is the first line when it has a non-numeric value cell (as in any csv with a header),
// or a line starting with `@`. The name of the first column is ignored.
fn is_header(cols: &[&str], data: &Dataset) -> bool {
    if !data.rows.is_empty() || !data.categories.is_empty() {
        return false;
    }
    cols[0].starts_with('@')
        || cols[1..].iter().any(|c| !c.trim().is_empty() && c.trim().parse::<f64>().is_err())
}

fn check_rows(data: &Dataset) -> Result<(), Error> {
    let expected = data.series();
    for (label, row) in data.labels.iter().zip(&data.rows) {
        if row.is_empty() {
            return Err(Error::NoValues { label: label.clone() });
        }
        if row.len() != expected {
            return Err(Error::RowLength { label: label.clone(), found: row.len(), expected });
        }
    }
    Ok(())
}

fn value(col: &str, line: &str) -> Result<f64, Error> {
    let col = col.trim();
    if col.is_empty() {
        return Ok(0.0);
    }
    col.parse().map_err(|_| Error::InvalidDataPoint { value: col.into(), line: line.into() })
}

fn unquote(col: &str) -> &str {
    col.trim_matches([' ', '"'])
}

// Splits on the delimiter outside quotes; a line without one (after column 0) falls back to whitespace.
fn split<'a>(line: &'a str, delim: &str) -> Vec<&'a str> {
    if line.find(delim).is_none_or(|i| i == 0) {
        return line.split_whitespace().collect();
    }
    let mut cols = Vec::new();
    let mut start = 0;
    for (i, _) in line.match_indices(delim) {
        if balanced_quotes(&line[i + delim.len()..]) {
            cols.push(&line[start..i]);
            start = i + delim.len();
        }
    }
    cols.push(&line[start..]);
    cols
}

fn balanced_quotes(mut s: &str) -> bool {
    while let Some(c) = s.chars().next() {
        if c == '\'' || c == '"' {
            match s[1..].find(c) {
                Some(end) => s = &s[end + 2..],
                None => return false,
            }
        } else {
            s = &s[c.len_utf8()..];
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn parse_str(text: &str) -> Result<Dataset, Error> {
        parse(Cursor::new(text), ",")
    }

    #[test]
    fn parses_header_labels_and_values() {
        let data = parse_str("@,Boys,Girls\n2007,183,202\n2008,231,199\n").unwrap();
        assert_eq!(data.categories, ["Boys", "Girls"]);
        assert_eq!(data.labels, ["2007", "2008"]);
        assert_eq!(data.rows, [[183.0, 202.0], [231.0, 199.0]]);
    }

    #[test]
    fn plain_csv_header_names_the_categories() {
        let data = parse_str("year,Boys,Girls\n2007,183,202\n").unwrap();
        assert_eq!(data.categories, ["Boys", "Girls"]);
        assert_eq!(data.labels, ["2007"]);
    }

    #[test]
    fn headerless_input_has_no_categories() {
        assert!(parse_str("a,1\nb,2\n").unwrap().categories.is_empty());
    }

    #[test]
    fn only_the_first_line_can_be_a_header() {
        let err = parse_str("name,count\na,1\nb,x\n").unwrap_err();
        assert_eq!(err.to_string(), "invalid data point [x] in line [b,x]");
    }

    #[test]
    fn quoted_header_names() {
        let data = parse_str("name,\"a, b\",c\nx,1,2\n").unwrap();
        assert_eq!(data.categories, ["a, b", "c"]);
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let data = parse_str("# note\n\na,1\n  \nb,2\n").unwrap();
        assert_eq!(data.labels, ["a", "b"]);
    }

    #[test]
    fn falls_back_to_whitespace() {
        let data = parse_str("a 1 2\n").unwrap();
        assert_eq!(data.rows, [[1.0, 2.0]]);
    }

    #[test]
    fn quoted_delimiters_do_not_split() {
        let data = parse_str("\"a, b\",1\n").unwrap();
        assert_eq!(data.labels, ["a, b"]);
        assert_eq!(data.rows, [[1.0]]);
    }

    #[test]
    fn custom_delimiter() {
        let data = parse(Cursor::new("a;1;2\n"), ";").unwrap();
        assert_eq!(data.rows, [[1.0, 2.0]]);
    }

    #[test]
    fn empty_cell_is_zero() {
        assert_eq!(parse_str("a,1,,3\n").unwrap().rows, [[1.0, 0.0, 3.0]]);
    }

    #[test]
    fn empty_input_is_an_empty_dataset() {
        assert!(parse_str("").unwrap().is_empty());
        assert!(parse_str("@,Boys\n").unwrap().is_empty());
    }

    #[test]
    fn invalid_number_is_reported_with_its_line() {
        let err = parse_str("a,1\nb,x\n").unwrap_err();
        assert_eq!(err.to_string(), "invalid data point [x] in line [b,x]");
    }

    #[test]
    fn ragged_rows_are_rejected() {
        let err = parse_str("a,1,2\nb,3\n").unwrap_err();
        assert_eq!(err.to_string(), "label [b] has 1 values, expected 2");
    }

    #[test]
    fn row_without_values_is_rejected() {
        assert!(matches!(parse_str("a\n"), Err(Error::NoValues { .. })));
    }
}
