use clap::Parser;
use std::io::{BufWriter, stdout};
use std::process::ExitCode;
use termgraph::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Some(shell) = cli.completions {
        termgraph::generate::completions(shell, &mut stdout());
        return ExitCode::SUCCESS;
    }
    if cli.man {
        return match termgraph::generate::man_page(&mut stdout()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail(&e),
        };
    }
    match termgraph::run(&cli, &mut BufWriter::new(stdout().lock())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&e),
    }
}

fn fail(e: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("termgraph: error: {e}");
    ExitCode::FAILURE
}
