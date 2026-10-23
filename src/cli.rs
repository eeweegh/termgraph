use crate::format::Format;
use crate::input::Header;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Options: see termgraph plot --help.

Input (file or stdin; empty input exits silently with status 0):
  year,Boys,Girls  optional csv header: names the categories (first column name ignored)
  2007,183,202     label followed by one value per category
  # comment        comment; blank lines are skipped
  Plain csv: fields may be \"quoted\"; --delim sets another single-character delimiter.
  The first line is a header if it has a non-numeric value cell, or starts with @;
  force it either way with --header / --no-header (e.g. for numeric names like 2020,2021).
  The legend is shown for more than one category (see --legend, --no-legend).

Format (--format): one printf conversion, f or d, with flags - + space 0, width and precision,
and optional literal text, e.g. %5.2f, %.1f%%, %d. Values are scaled to K/M/B/T before formatting
(not in --stacked, which formats the row total).

The tg driver script (zsh) is embedded: termgraph tg-script > ~/.local/bin/tg && chmod 755 ~/.local/bin/tg

Examples:
  termgraph plot data.csv --width 40 --color blue red
  printf 'year,Boys,Girls\\n2007,183,202\\n2008,231,199\\n' | termgraph plot --stacked";

/// When to emit ANSI color escapes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Colorize {
    /// On a terminal, unless `NO_COLOR` is set
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(
    name = "termgraph",
    version,
    about = "Draw basic bar graphs in the terminal.",
    after_help = AFTER_HELP,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Draw a bar graph from csv (file or stdin)
    Plot(Box<ChartArgs>),
    /// Show the color table
    Colors,
    /// Print the tg driver script (zsh)
    #[command(name = "tg-script")]
    TgScript,
    /// Print a shell completion script
    #[command(hide = true)]
    Completions { shell: Shell },
    /// Print the man page (roff)
    #[command(hide = true)]
    Man,
}

#[derive(Debug, clap::Args)]
pub struct ChartArgs {
    /// Data file (csv) [default: stdin]
    pub file: Option<PathBuf>,
    /// Title printed above the graph
    #[arg(long)]
    pub title: Option<String>,
    /// Width of the graph in characters
    #[arg(long, default_value_t = 50)]
    pub width: u16,
    /// printf-style format for values
    #[arg(long, default_value = "%-5.2f", value_name = "FORMAT")]
    pub format: Format,
    /// Text appended to every value
    #[arg(long, default_value = "")]
    pub suffix: String,
    /// Do not print the label column
    #[arg(long)]
    pub no_labels: bool,
    /// Do not print the values at the end
    #[arg(long)]
    pub no_values: bool,
    /// Print an empty line between rows
    #[arg(long)]
    pub space_between: bool,
    /// Bar color per category, by name or 8-bit number (see the colors subcommand)
    #[arg(long, num_args = 1.., value_name = "COLOR")]
    pub color: Vec<String>,
    /// Treat the first line as a header (names the categories)
    #[arg(long, conflicts_with = "no_header")]
    pub header: bool,
    /// Treat the first line as data
    #[arg(long)]
    pub no_header: bool,
    /// When to use ANSI colors in the output
    #[arg(long, value_enum, default_value_t)]
    pub colorize: Colorize,
    /// Always show the legend, even for a single category
    #[arg(long, conflicts_with = "no_legend")]
    pub legend: bool,
    /// Never show the legend
    #[arg(long)]
    pub no_legend: bool,
    /// Draw one stacked bar per row
    #[arg(long)]
    pub stacked: bool,
    /// Custom tick character, emoji approved
    #[arg(long, value_name = "TICK")]
    pub custom_tick: Option<String>,
    /// Field delimiter: one ASCII character, or \t for a tab
    #[arg(long, default_value = ",", value_parser = parse_delim)]
    pub delim: u8,
    /// Show the value before the bar
    #[arg(long)]
    pub label_before: bool,
}

impl ChartArgs {
    pub fn header(&self) -> Header {
        match (self.header, self.no_header) {
            (true, _) => Header::Always,
            (_, true) => Header::Never,
            _ => Header::Auto,
        }
    }

    /// `None` leaves it to the number of categories.
    pub fn legend(&self) -> Option<bool> {
        match (self.legend, self.no_legend) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        }
    }
}

impl Colorize {
    /// `terminal`: stdout is a terminal. `no_color`: `NO_COLOR` is set to a non-empty value (<https://no-color.org>).
    pub fn enabled(self, terminal: bool, no_color: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => terminal && !no_color,
        }
    }
}

fn parse_delim(s: &str) -> Result<u8, String> {
    match s.as_bytes() {
        b"\\t" => Ok(b'\t'),
        [byte] if byte.is_ascii() => Ok(*byte),
        _ => Err(format!("'{s}' is not a single ASCII character")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiters() {
        assert_eq!(parse_delim(";"), Ok(b';'));
        assert_eq!(parse_delim("\\t"), Ok(b'\t'));
        assert_eq!(parse_delim("\t"), Ok(b'\t'));
        assert!(parse_delim("").is_err() && parse_delim(";;").is_err() && parse_delim("é").is_err());
    }

    #[test]
    fn colorize_modes() {
        assert!(Colorize::Auto.enabled(true, false));
        assert!(!Colorize::Auto.enabled(false, false));
        assert!(!Colorize::Auto.enabled(true, true));
        assert!(Colorize::Always.enabled(false, true));
        assert!(!Colorize::Never.enabled(true, false));
    }
}
