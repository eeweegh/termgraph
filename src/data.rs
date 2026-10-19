#[derive(Debug, Default, PartialEq)]
pub struct Dataset {
    pub categories: Vec<String>,
    pub labels: Vec<String>,
    pub rows: Vec<Vec<f64>>,
}

impl Dataset {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn series(&self) -> usize {
        self.rows.first().map_or(0, Vec::len)
    }

    pub fn min(&self) -> f64 {
        self.rows.iter().flatten().copied().fold(f64::INFINITY, f64::min)
    }

    /// Scales rows so the largest row sum spans `width`; negatives are lifted so the minimum becomes zero.
    pub fn normalised(&self, width: u16) -> Vec<Vec<f64>> {
        let shift = (-self.min()).max(0.0);
        let shifted: Vec<Vec<f64>> =
            self.rows.iter().map(|row| row.iter().map(|v| v + shift).collect()).collect();
        let max = shifted.iter().map(|row| row.iter().sum::<f64>()).fold(0.0, f64::max);
        if max <= 0.0 {
            return shifted;
        }
        let factor = f64::from(width) / max;
        shifted.into_iter().map(|row| row.into_iter().map(|v| v * factor).collect()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset(rows: &[&[f64]]) -> Dataset {
        Dataset { rows: rows.iter().map(|r| r.to_vec()).collect(), ..Dataset::default() }
    }

    #[test]
    fn scales_largest_row_sum_to_width() {
        let data = dataset(&[&[1.0, 1.0], &[2.0, 2.0]]);
        assert_eq!(data.normalised(8), [[2.0, 2.0], [4.0, 4.0]]);
    }

    #[test]
    fn negatives_are_lifted_to_zero() {
        let data = dataset(&[&[-5.0], &[5.0]]);
        assert_eq!(data.normalised(10), [[0.0], [10.0]]);
    }

    #[test]
    fn all_zero_is_left_alone() {
        let data = dataset(&[&[0.0], &[0.0]]);
        assert_eq!(data.normalised(10), [[0.0], [0.0]]);
    }

    #[test]
    fn series_counts_values_of_first_row() {
        assert_eq!(dataset(&[&[1.0, 2.0, 3.0]]).series(), 3);
        assert_eq!(Dataset::default().series(), 0);
    }
}
