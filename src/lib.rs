//! termgraph: draw basic bar graphs in the terminal.

pub mod cli;
pub mod color;
pub mod data;
pub mod error;
pub mod format;
pub mod generate;
pub mod input;
pub mod render;

use cli::Cli;
use render::Options;
use std::io::{self, ErrorKind, Write};

const TICK: &str = "⣿";
const SMALL_TICK: &str = "⡇";

pub fn run(cli: &Cli, out: &mut impl Write) -> anyhow::Result<()> {
    if cli.colors {
        return finish(color::write_table(out).and_then(|()| out.flush()));
    }
    let delim = cli.delim.as_deref().filter(|d| !d.is_empty()).unwrap_or(",");
    let data = input::read(cli.file.as_deref(), delim)?;
    if data.is_empty() {
        return Ok(());
    }
    let colors = color::resolve(&cli.color, data.series())?;
    finish(render::chart(out, &data, &colors, &options(cli)).and_then(|()| out.flush()))
}

fn options(cli: &Cli) -> Options {
    let custom_tick = cli.custom_tick.as_deref().filter(|t| !t.is_empty());
    Options {
        width: cli.width,
        tick: custom_tick.unwrap_or(TICK).into(),
        small_tick: if custom_tick.is_some() { String::new() } else { SMALL_TICK.into() },
        format: cli.format.clone(),
        suffix: cli.suffix.clone(),
        title: cli.title.clone().filter(|t| !t.is_empty()),
        stacked: cli.stacked,
        no_labels: cli.no_labels,
        no_values: cli.no_values,
        space_between: cli.space_between,
        label_before: cli.label_before,
    }
}

// A closed pipe (e.g. `| head`) is a normal way for the reader to stop.
fn finish(result: io::Result<()>) -> anyhow::Result<()> {
    match result {
        Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
        other => Ok(other?),
    }
}
