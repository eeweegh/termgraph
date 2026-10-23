use crate::color::{Fg, RESET};
use crate::data::Dataset;
use crate::format::{Format, readable};
use std::borrow::Cow;
use std::fmt;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Options<'a> {
    pub width: u16,
    pub tick: &'a str,
    pub small_tick: &'a str,
    pub format: &'a Format,
    pub suffix: &'a str,
    pub title: Option<&'a str>,
    /// `None` shows the legend for more than one category.
    pub legend: Option<bool>,
    pub color: bool,
    pub stacked: bool,
    pub no_labels: bool,
    pub no_values: bool,
    pub space_between: bool,
    pub label_before: bool,
}

pub fn chart(out: &mut impl Write, data: &Dataset, colors: &[u8], opts: &Options) -> io::Result<()> {
    if let Some(title) = opts.title {
        write!(out, "# {title}\n\n")?;
    }
    let normal = data.normalised(opts.width);
    let label_width = data.labels().iter().map(|l| l.width()).max().unwrap_or(0);
    let mut chart = Chart { out, data, colors, opts, normal: &normal, min: data.min(), label_width };
    if !data.categories().is_empty() && opts.legend.unwrap_or(data.categories().len() > 1) {
        chart.legend()?;
    }
    if opts.stacked { chart.stacked() } else { chart.rows() }
}

// Prints its content only when color is on.
struct Esc<T>(bool, T);

impl<T: fmt::Display> fmt::Display for Esc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 { self.1.fmt(f) } else { Ok(()) }
    }
}

fn pad(s: &str, width: usize) -> String {
    format!("{s}{}", " ".repeat(width.saturating_sub(s.width())))
}

// Whole blocks of a bar. The input is never negative: `Dataset::normalised` lifts the minimum to zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn blocks(normalised: f64) -> usize {
    normalised as usize
}

// Each label with its values and their scaled bars.
fn lines<'a>(data: &'a Dataset, normal: &'a [f64]) -> impl Iterator<Item = (&'a str, &'a [f64], &'a [f64])> {
    let labels = data.labels().iter().map(String::as_str);
    labels.zip(data.rows()).zip(normal.chunks_exact(data.series().max(1))).map(|((l, v), n)| (l, v, n))
}

struct Chart<'a, W> {
    out: &'a mut W,
    data: &'a Dataset,
    colors: &'a [u8],
    opts: &'a Options<'a>,
    // `data.rows()` scaled to the graph width, in the same layout.
    normal: &'a [f64],
    min: f64,
    label_width: usize,
}

impl<W: Write> Chart<'_, W> {
    fn color(&self, index: usize) -> u8 {
        self.colors[index % self.colors.len()]
    }

    fn fg(&self, code: u8) -> Esc<Fg> {
        Esc(self.opts.color, Fg(code))
    }

    fn reset(&self) -> Esc<&'static str> {
        Esc(self.opts.color, RESET)
    }

    // Wraps once the names pass the graph width.
    fn legend(&mut self) -> io::Result<()> {
        let mut used = 0;
        for (i, name) in self.data.categories().iter().enumerate() {
            write!(self.out, "{}{} {name} {}", self.fg(self.color(i)), self.opts.tick, self.reset())?;
            used += name.width() + 2;
            if used > usize::from(self.opts.width) {
                writeln!(self.out)?;
                used = 0;
            }
        }
        write!(self.out, "\n\n")
    }

    fn rows(&mut self) -> io::Result<()> {
        let opts = self.opts;
        write!(self.out, "{}", self.reset())?;
        for (i, (label, values, normal)) in lines(self.data, self.normal).enumerate() {
            if opts.space_between && i != 0 {
                writeln!(self.out)?;
            }
            let mut label = match (opts.no_labels, opts.label_before) {
                (true, _) => String::new(),
                (false, true) => pad(label, self.label_width),
                (false, false) => pad(label, self.label_width) + " ",
            };
            for (j, (&value, &scaled)) in values.iter().zip(normal).enumerate() {
                // Only the first series of a row carries the label.
                if j > 0 {
                    label = " ".repeat(label.width());
                }
                let shown = if opts.no_values { String::new() } else { self.shown(value) };
                let color = self.color(j);
                if opts.label_before {
                    write!(self.out, "{}{label} {shown}{}  ", self.reset(), opts.suffix)?;
                    self.bar(value, blocks(scaled), color)?;
                    writeln!(self.out)?;
                } else {
                    write!(self.out, "{label}{}", self.reset())?;
                    self.bar(value, blocks(scaled), color)?;
                    writeln!(self.out, "{shown}{}", opts.suffix)?;
                }
            }
        }
        writeln!(self.out)
    }

    fn stacked(&mut self) -> io::Result<()> {
        let opts = self.opts;
        for (i, (label, values, normal)) in lines(self.data, self.normal).enumerate() {
            if opts.space_between && i != 0 {
                writeln!(self.out)?;
            }
            if !opts.no_labels {
                write!(self.out, "{} ", pad(label, self.label_width))?;
            }
            for (j, (&value, &scaled)) in values.iter().zip(normal).enumerate() {
                write!(self.out, "{}", self.reset())?;
                self.bar(value, blocks(scaled), self.color(j))?;
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
            Cow::Borrowed(opts.small_tick)
        } else {
            Cow::Owned(opts.tick.repeat(blocks))
        };
        if color == 0 {
            write!(self.out, "{bar}")
        } else {
            write!(self.out, "{}{bar}{}", self.fg(color), self.reset())
        }
    }
}
