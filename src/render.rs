use crate::color::{Fg, RESET};
use crate::data::Dataset;
use crate::format::{Format, readable};
use std::io::{self, Write};

#[derive(Debug)]
pub struct Options {
    pub width: u16,
    pub tick: String,
    pub small_tick: String,
    pub format: Format,
    pub suffix: String,
    pub title: Option<String>,
    pub stacked: bool,
    pub no_labels: bool,
    pub no_values: bool,
    pub space_between: bool,
    pub label_before: bool,
}

pub fn chart(out: &mut impl Write, data: &Dataset, colors: &[u8], opts: &Options) -> io::Result<()> {
    if let Some(title) = &opts.title {
        write!(out, "# {title}\n\n")?;
    }
    let label_width = data.labels.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let mut chart =
        Chart { out, data, colors, opts, normal: data.normalised(opts.width), min: data.min(), label_width };
    if !data.categories.is_empty() {
        chart.legend()?;
    }
    if opts.stacked { chart.stacked() } else { chart.rows() }
}

struct Chart<'a, W> {
    out: &'a mut W,
    data: &'a Dataset,
    colors: &'a [u8],
    opts: &'a Options,
    normal: Vec<Vec<f64>>,
    min: f64,
    label_width: usize,
}

impl<W: Write> Chart<'_, W> {
    fn color(&self, index: usize) -> u8 {
        self.colors[index % self.colors.len()]
    }

    // Wraps once the names pass the graph width.
    fn legend(&mut self) -> io::Result<()> {
        let mut used = 0;
        for (i, name) in self.data.categories.iter().enumerate() {
            write!(self.out, "{}{} {name} {RESET}", Fg(self.color(i)), self.opts.tick)?;
            used += name.chars().count() + 2;
            if used > usize::from(self.opts.width) {
                writeln!(self.out)?;
                used = 0;
            }
        }
        write!(self.out, "\n\n")
    }

    fn rows(&mut self) -> io::Result<()> {
        let opts = self.opts;
        write!(self.out, "{RESET}")?;
        for (i, (label, values)) in self.data.labels.iter().zip(&self.data.rows).enumerate() {
            if opts.space_between && i != 0 {
                writeln!(self.out)?;
            }
            let width = self.label_width;
            let mut label = match (opts.no_labels, opts.label_before) {
                (true, _) => String::new(),
                (false, true) => format!("{label:<width$}"),
                (false, false) => format!("{label:<width$} "),
            };
            for (j, &value) in values.iter().enumerate() {
                // Only the first series of a row carries the label.
                if j > 0 {
                    label = " ".repeat(label.chars().count());
                }
                let shown = if opts.no_values { String::new() } else { self.shown(value) };
                let blocks = self.normal[i][j] as usize;
                let color = self.color(j);
                if opts.label_before {
                    write!(self.out, "{RESET}{label} {shown}{}  ", opts.suffix)?;
                    self.bar(value, blocks, color)?;
                    writeln!(self.out)?;
                } else {
                    write!(self.out, "{label}{RESET}")?;
                    self.bar(value, blocks, color)?;
                    writeln!(self.out, "{shown}{}", opts.suffix)?;
                }
            }
        }
        writeln!(self.out)
    }

    fn stacked(&mut self) -> io::Result<()> {
        let opts = self.opts;
        for (i, (label, values)) in self.data.labels.iter().zip(&self.data.rows).enumerate() {
            if opts.space_between && i != 0 {
                writeln!(self.out)?;
            }
            if !opts.no_labels {
                write!(self.out, "{label:<width$} ", width = self.label_width)?;
            }
            for (j, &value) in values.iter().enumerate() {
                write!(self.out, "{RESET}")?;
                self.bar(value, self.normal[i][j] as usize, self.color(j))?;
            }
            let total = if opts.no_values {
                String::new()
            } else {
                format!(" {}", opts.format.apply(values.iter().sum()))
            };
            writeln!(self.out, "{total}{}", opts.suffix)?;
        }
        Ok(())
    }

    fn shown(&self, value: f64) -> String {
        let (scaled, unit) = readable(value);
        let lead = if self.opts.label_before { "" } else { " " };
        format!("{lead}{}{unit}", self.opts.format.apply(scaled))
    }

    // Black (code 0) is drawn in the terminal's own color; a positive value under one block gets the small tick.
    fn bar(&mut self, value: f64, blocks: usize, color: u8) -> io::Result<()> {
        let opts = self.opts;
        let tiny = blocks == 0 && (value > self.min || value > 0.0);
        let bar = if tiny || (opts.label_before && value == 0.0) {
            opts.small_tick.clone()
        } else {
            opts.tick.repeat(blocks)
        };
        if color == 0 { write!(self.out, "{bar}") } else { write!(self.out, "{}{bar}{RESET}", Fg(color)) }
    }
}
