use crate::error::Error;

/// Labelled rows of equally many values, optionally with a name per value column.
///
/// Rows can only be added through [`Dataset::push`], which keeps them rectangular and in line with the
/// category names, so the accessors never see a ragged set.
#[derive(Debug, Default, PartialEq)]
pub struct Dataset {
    categories: Vec<String>,
    labels: Vec<String>,
    // Row-major, `series` values per label.
    values: Vec<f64>,
    series: usize,
}

impl Dataset {
    pub fn new(categories: Vec<String>) -> Self {
        Self { categories, ..Self::default() }
    }

    /// Adds a row; it needs at least one value, and as many as the rows before it and the category names.
    pub fn push(&mut self, label: &str, values: &[f64]) -> Result<(), Error> {
        if values.is_empty() {
            return Err(Error::NoValues { label: label.into() });
        }
        if self.labels.is_empty() {
            if !self.categories.is_empty() && self.categories.len() != values.len() {
                return Err(Error::HeaderLength { found: self.categories.len(), expected: values.len() });
            }
            self.series = values.len();
        } else if values.len() != self.series {
            return Err(Error::RowLength { label: label.into(), found: values.len(), expected: self.series });
        }
        self.labels.push(label.into());
        self.values.extend_from_slice(values);
        Ok(())
    }

    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Number of values per row; 0 while there are no rows.
    pub fn series(&self) -> usize {
        self.series
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    pub fn rows(&self) -> impl Iterator<Item = &[f64]> {
        self.values.chunks_exact(self.series.max(1))
    }

    pub fn min(&self) -> f64 {
        self.values.iter().copied().fold(f64::INFINITY, f64::min)
    }

    /// Scales values so the largest row sum spans `width`; negatives are lifted so the minimum becomes zero.
    /// The result has the layout of the values: `series` entries per row.
    pub fn normalised(&self, width: u16) -> Vec<f64> {
        let shift = (-self.min()).max(0.0);
        let lifted = |row: &[f64]| row.iter().map(|v| v + shift).sum::<f64>();
        let max = self.rows().map(lifted).fold(0.0, f64::max);
        let factor = if max > 0.0 { f64::from(width) / max } else { 1.0 };
        self.values.iter().map(|v| (v + shift) * factor).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset(rows: &[&[f64]]) -> Dataset {
        let mut data = Dataset::default();
        for (i, row) in rows.iter().enumerate() {
            data.push(&i.to_string(), row).unwrap();
        }
        data
    }

    #[test]
    fn scales_largest_row_sum_to_width() {
        let data = dataset(&[&[1.0, 1.0], &[2.0, 2.0]]);
        assert_eq!(data.normalised(8), [2.0, 2.0, 4.0, 4.0]);
    }

    #[test]
    fn negatives_are_lifted_to_zero() {
        let data = dataset(&[&[-5.0], &[5.0]]);
        assert_eq!(data.normalised(10), [0.0, 10.0]);
    }

    #[test]
    fn all_zero_is_left_alone() {
        let data = dataset(&[&[0.0], &[0.0]]);
        assert_eq!(data.normalised(10), [0.0, 0.0]);
    }

    #[test]
    fn rows_come_back_in_order() {
        let data = dataset(&[&[1.0, 2.0], &[3.0, 4.0]]);
        assert_eq!(data.rows().collect::<Vec<_>>(), [[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(data.labels(), ["0", "1"]);
        assert_eq!(Dataset::default().rows().count(), 0);
    }

    #[test]
    fn series_is_the_row_width() {
        assert_eq!(dataset(&[&[1.0, 2.0, 3.0]]).series(), 3);
        assert_eq!(Dataset::default().series(), 0);
    }

    #[test]
    fn rows_must_agree_in_width() {
        let mut data = dataset(&[&[1.0, 2.0]]);
        let err = data.push("b", &[3.0]).unwrap_err();
        assert_eq!(err.to_string(), "label [b] has 1 values, expected 2");
        assert!(matches!(data.push("c", &[]), Err(Error::NoValues { .. })));
        assert_eq!(data.labels().len(), 1);
    }

    #[test]
    fn rows_must_match_the_category_names() {
        let mut data = Dataset::new(vec!["a".into(), "b".into(), "c".into()]);
        let err = data.push("r", &[1.0, 2.0]).unwrap_err();
        assert_eq!(err.to_string(), "header has 3 names, rows have 2 values");
        assert!(data.push("r", &[1.0, 2.0, 3.0]).is_ok());
    }
}
