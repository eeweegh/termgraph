use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read {}: {source}", path.display())]
    ReadInput { path: PathBuf, source: std::io::Error },
    #[error("could not read input: {0}")]
    Read(#[from] std::io::Error),
    #[error("invalid data point [{value}] in line [{line}]")]
    InvalidDataPoint { value: String, line: String },
    #[error("label [{label}] has no values")]
    NoValues { label: String },
    #[error("label [{label}] has {found} values, expected {expected}")]
    RowLength { label: String, found: usize, expected: usize },
    #[error("color ({given}) and category ({categories}) array sizes don't match")]
    ColorCount { given: usize, categories: usize },
    #[error("invalid color {name}, choose from {valid}")]
    InvalidColor { name: String, valid: String },
}
