use crate::format::Format;
use clap::Parser;
use clap_complete::Shell;
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Input (file or stdin; empty input exits silently with status 0):
  year,Boys,Girls  optional csv header: names the categories (first column name ignored)
  2007,183,202     label followed by one value per category
  # comment        comment; blank lines are skipped
  Columns split on the delimiter (outside quotes), else on whitespace.
  The first line is a header if it has a non-numeric value cell, or starts with @.
  The legend is shown for more than one category (see --legend, --no-legend).

Format (--format): one printf conversion, f or d, with flags - + space 0, width and precision,
and optional literal text, e.g. %5.2f, %.1f%%, %d. Values are scaled to K/M/B/T before formatting
(not in --stacked, which formats the row total).

The tg driver script (zsh) is embedded: termgraph --tg-script > ~/.local/bin/tg && chmod 755 ~/.local/bin/tg

Examples:
  termgraph data.csv --width 40 --color blue red
  printf 'year,Boys,Girls\\n2007,183,202\\n2008,231,199\\n' | termgraph --stacked";

#[derive(Debug, Parser)]
#[command(name = "termgraph", version, about = "Draw basic bar graphs in the terminal.", after_help = AFTER_HELP)]
pub struct Cli {
    /// Data file (comma or space separated) [default: stdin]
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
    /// Bar color per category, by name or 8-bit number (see --colors)
    #[arg(long, num_args = 1.., value_name = "COLOR")]
    pub color: Vec<String>,
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
    /// Custom delimiter [default: , or whitespace]
    #[arg(long)]
    pub delim: Option<String>,
    /// Show the value before the bar
    #[arg(long)]
    pub label_before: bool,
    /// Show the color table and exit
    #[arg(long)]
    pub colors: bool,
    /// Print the tg driver script (zsh) and exit
    #[arg(long)]
    pub tg_script: bool,
    /// Print a shell completion script and exit
    #[arg(long, value_name = "SHELL", hide = true)]
    pub completions: Option<Shell>,
    /// Print the man page (roff) and exit
    #[arg(long, hide = true)]
    pub man: bool,
}
