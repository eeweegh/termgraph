//! termgraph: draw basic bar graphs in the terminal.

mod cli;
mod color;
mod data;
mod error;
mod format;
mod generate;
mod input;
mod render;

use clap::Parser;
use cli::{ChartArgs, Cli, Command};
use error::Error;
use render::Options;
use std::io::{self, BufWriter, ErrorKind, IsTerminal, Write, stdout};
use std::process::ExitCode;

const TICK: &str = "⣿";
const SMALL_TICK: &str = "⡇";

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut out = BufWriter::new(stdout().lock());
    match run(&cli, &mut out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("termgraph: error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli, out: &mut impl Write) -> Result<(), Error> {
    let result = match &cli.command {
        Command::Plot(args) => return chart(args, out),
        Command::Colors => color::write_table(out),
        Command::TgScript => generate::tg_script(out),
        Command::Completions { shell } => {
            generate::completions(*shell, out);
            Ok(())
        }
        Command::Man => generate::man_page(out),
    };
    finish(result.and_then(|()| out.flush()))
}

fn chart(args: &ChartArgs, out: &mut impl Write) -> Result<(), Error> {
    let data = input::read(args.file.as_deref(), args.delim, args.header())?;
    if data.is_empty() {
        return Ok(());
    }
    let colors = color::resolve(&args.color, data.series())?;
    finish(render::chart(out, &data, &colors, &options(args)).and_then(|()| out.flush()))
}

fn options(args: &ChartArgs) -> Options<'_> {
    let custom_tick = args.custom_tick.as_deref().filter(|t| !t.is_empty());
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    Options {
        width: args.width,
        tick: custom_tick.unwrap_or(TICK),
        small_tick: if custom_tick.is_some() { "" } else { SMALL_TICK },
        format: &args.format,
        suffix: &args.suffix,
        title: args.title.as_deref().filter(|t| !t.is_empty()),
        legend: args.legend(),
        color: args.colorize.enabled(io::stdout().is_terminal(), no_color),
        stacked: args.stacked,
        no_labels: args.no_labels,
        no_values: args.no_values,
        space_between: args.space_between,
        label_before: args.label_before,
    }
}

// A closed pipe (e.g. `| head`) is a normal way for the reader to stop.
fn finish(result: io::Result<()>) -> Result<(), Error> {
    match result {
        Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
        other => other.map_err(Error::Write),
    }
}
