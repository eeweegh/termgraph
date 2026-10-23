use crate::data::Dataset;
use crate::error::Error;
use std::fs::File;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

/// Whether the first line names the categories.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Header {
    /// A line starting with `@`, or one with a non-numeric value cell.
    #[default]
    Auto,
    Always,
    Never,
}

/// Reads from `path`, or from stdin for `None` and `-`. A terminal on stdin counts as no input.
pub fn read(path: Option<&Path>, delim: u8, header: Header) -> Result<Dataset, Error> {
    match path.filter(|p| *p != Path::new("-")) {
        Some(path) => {
            let file = File::open(path).map_err(|source| Error::ReadInput { path: path.into(), source })?;
            parse(file, delim, header)
        }
        None if io::stdin().is_terminal() => Ok(Dataset::default()),
        None => parse(io::stdin().lock(), delim, header),
    }
}

/// Parses csv: `#` lines and blank lines are skipped, a first line may name the categories, and every
/// other line is a label followed by one number per category.
pub fn parse(reader: impl Read, delim: u8, header: Header) -> Result<Dataset, Error> {
    let mut records = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delim)
        .comment(Some(b'#'))
        .trim(csv::Trim::All)
        .from_reader(reader);
    let mut data = Dataset::default();
    let mut record = csv::StringRecord::new();
    let mut values = Vec::new();
    let mut first = true;
    while records.read_record(&mut record)? {
        if record.len() == 1 && record[0].is_empty() {
            continue;
        }
        if std::mem::take(&mut first) && is_header(&record, header) {
            data = Dataset::new(record.iter().skip(1).map(String::from).collect());
            continue;
        }
        values.clear();
        for col in record.iter().skip(1) {
            values.push(value(col).ok_or_else(|| invalid(col, &record, delim))?);
        }
        data.push(&record[0], &values)?;
    }
    Ok(data)
}

// The name of the first column is ignored.
fn is_header(record: &csv::StringRecord, header: Header) -> bool {
    match header {
        Header::Always => true,
        Header::Never => false,
        Header::Auto => {
            record[0].starts_with('@') || record.iter().skip(1).any(|c| !c.is_empty() && number(c).is_none())
        }
    }
}

// Finite numbers only: `nan` and `inf` parse as f64 but cannot be drawn.
fn number(col: &str) -> Option<f64> {
    col.parse().ok().filter(|v: &f64| v.is_finite())
}

// An empty cell counts as zero.
fn value(col: &str) -> Option<f64> {
    if col.is_empty() { Some(0.0) } else { number(col) }
}

fn invalid(col: &str, record: &csv::StringRecord, delim: u8) -> Error {
    let line = record.iter().collect::<Vec<_>>().join(&char::from(delim).to_string());
    Error::InvalidDataPoint { value: col.into(), line }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(text: &str) -> Result<Dataset, Error> {
        parse(text.as_bytes(), b',', Header::Auto)
    }

    fn parse_with(text: &str, header: Header) -> Result<Dataset, Error> {
        parse(text.as_bytes(), b',', header)
    }

    #[test]
    fn parses_header_labels_and_values() {
        let data = parse_str("@,Boys,Girls\n2007,183,202\n2008,231,199\n").unwrap();
        assert_eq!(data.categories(), ["Boys", "Girls"]);
        assert_eq!(data.labels(), ["2007", "2008"]);
        assert_eq!(data.rows().collect::<Vec<_>>(), [[183.0, 202.0], [231.0, 199.0]]);
    }

    #[test]
    fn plain_csv_header_names_the_categories() {
        let data = parse_str("year,Boys,Girls\n2007,183,202\n").unwrap();
        assert_eq!(data.categories(), ["Boys", "Girls"]);
        assert_eq!(data.labels(), ["2007"]);
    }

    #[test]
    fn headerless_input_has_no_categories() {
        assert!(parse_str("a,1\nb,2\n").unwrap().categories().is_empty());
    }

    #[test]
    fn only_the_first_line_can_be_a_header() {
        let err = parse_str("name,count\na,1\nb,x\n").unwrap_err();
        assert_eq!(err.to_string(), "invalid data point [x] in line [b,x]");
    }

    #[test]
    fn numeric_header_needs_the_flag() {
        let text = "x,2020,2021\na,1,2\n";
        assert!(parse_str(text).unwrap().categories().is_empty());
        let data = parse_with(text, Header::Always).unwrap();
        assert_eq!(data.categories(), ["2020", "2021"]);
        assert_eq!(data.labels(), ["a"]);
    }

    #[test]
    fn never_reads_the_first_line_as_data() {
        let err = parse_with("x,a\n", Header::Never).unwrap_err();
        assert_eq!(err.to_string(), "invalid data point [a] in line [x,a]");
    }

    #[test]
    fn single_column_header_is_consumed_once() {
        let data = parse_with("name\na,1\nb,2\n", Header::Always).unwrap();
        assert_eq!(data.labels(), ["a", "b"]);
    }

    #[test]
    fn quoted_header_names() {
        let data = parse_str("name,\"a, b\",c\nx,1,2\n").unwrap();
        assert_eq!(data.categories(), ["a, b", "c"]);
    }

    #[test]
    fn apostrophes_are_not_quotes() {
        let data = parse_str("year,Men's,Women's\n2007,1,2\n").unwrap();
        assert_eq!(data.categories(), ["Men's", "Women's"]);
    }

    #[test]
    fn non_finite_values_are_rejected() {
        for text in ["a,1\nb,nan\n", "a,1\nb,inf\n", "a,1\nb,-Infinity\n"] {
            assert!(matches!(parse_str(text), Err(Error::InvalidDataPoint { .. })), "{text:?}");
        }
    }

    #[test]
    fn nan_in_the_first_line_makes_it_a_header() {
        assert_eq!(parse_str("x,nan\na,1\n").unwrap().categories(), ["nan"]);
    }

    #[test]
    fn header_must_match_the_values() {
        let err = parse_str("x,a,b,c\nr,1,2\n").unwrap_err();
        assert_eq!(err.to_string(), "header has 3 names, rows have 2 values");
    }

    #[test]
    fn byte_order_mark_is_dropped() {
        assert_eq!(parse_str("\u{feff}a,1\n").unwrap().labels(), ["a"]);
        assert_eq!(parse_str("\u{feff}x,Boys\na,1\n").unwrap().categories(), ["Boys"]);
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let data = parse_str("# note\n\na,1\n  \nb,2\n").unwrap();
        assert_eq!(data.labels(), ["a", "b"]);
    }

    #[test]
    fn quoted_delimiters_and_escapes_do_not_split() {
        let data = parse_str("\"a, \"\"b\"\"\",1\n").unwrap();
        assert_eq!(data.labels(), ["a, \"b\""]);
        assert_eq!(data.rows().collect::<Vec<_>>(), [[1.0]]);
    }

    #[test]
    fn quoted_fields_may_span_lines() {
        assert_eq!(parse_str("\"a\nb\",1\n").unwrap().labels(), ["a\nb"]);
    }

    #[test]
    fn custom_delimiter() {
        let data = parse("a;1;2\n".as_bytes(), b';', Header::Auto).unwrap();
        assert_eq!(data.rows().collect::<Vec<_>>(), [[1.0, 2.0]]);
    }

    #[test]
    fn empty_cell_is_zero() {
        let data = parse_str("a,1,,3\n").unwrap();
        assert_eq!(data.rows().collect::<Vec<_>>(), [[1.0, 0.0, 3.0]]);
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

    #[test]
    fn invalid_utf8_is_a_read_error() {
        assert!(matches!(parse(&b"a,1\n\xff,2\n"[..], b',', Header::Auto), Err(Error::Read(_))));
    }
}
