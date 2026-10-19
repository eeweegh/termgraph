use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::Shell;
use std::io::{self, Write};

pub fn completions(shell: Shell, out: &mut impl Write) {
    clap_complete::generate(shell, &mut Cli::command(), "termgraph", out);
}

pub fn man_page(out: &mut impl Write) -> io::Result<()> {
    clap_mangen::Man::new(Cli::command()).render(out)
}
