# termgraph

Draw basic bar graphs in the terminal. Rust rewrite of a trimmed-down fork of [mkaz/termgraph](https://github.com/mkaz/termgraph).

```
printf 'year,Boys,Girls\n2007,183,202\n2008,231,199\n' | termgraph plot --stacked
termgraph plot data.csv --width 40 --color blue red
```

Input is a file or stdin: plain CSV with an optional header (`year,Cat1,Cat2`; the first column name is ignored, a leading `@` also works; `--header` / `--no-header` override the detection, needed for numeric names like `2020,2021`), then `label,value[,value...]` rows; `#` lines and blank lines are skipped. It is plain csv: fields may be `"quoted"`, and `--delim` sets another single-character delimiter (`\t` for tab).
Values must be finite numbers, and the header must name as many categories as the rows have values. Colors are only emitted on a terminal and without `NO_COLOR` (`--colorize auto|always|never`). Empty input exits silently with status 0; a subcommand is always required (bare `termgraph` shows help). Errors go to stderr as `termgraph: error: ...` with status 1.

`termgraph --help` lists the subcommands, `termgraph plot --help` every option. `--format` takes one printf conversion (`%5.2f`, `%.1f%%`, `%d`), not a Python format spec.

`bin/tg` is a small zsh driver that sets the look (width from the terminal, stacked bars; `TGNOSTACK` for plain) and passes everything else to termgraph. It is embedded in the binary; `just install` writes it to `~/.local/bin`, or from a plain cargo install:

```
cargo install --git https://github.com/eeweegh/termgraph
termgraph tg-script > ~/.local/bin/tg && chmod 755 ~/.local/bin/tg
```

## Layout of the code

| Module | Role |
| --- | --- |
| `cli` | clap definitions |
| `input`, `data` | parsing into a `Dataset`; normalisation (negatives lifted, scaled to the width) |
| `color`, `format` | color names and escapes; printf-style value format and K/M/B/T scaling |
| `render` | legend, bar rows and stacked rows |
| `error`, `generate` | typed errors; completions and man page |

## Development

```
just check      # fmt, clippy -D warnings, tests
```

Tests: unit tests per module, `tests/cli.rs` (exit codes, stderr, empty input) and `tests/render.rs` (insta snapshots of the binary's output, escapes made visible).

`termgraph plot`, `colors`, `tg-script`, `completions <shell>` and `man` are the subcommands; one is always required (plotting used to be the default, and `colors` etc. were flags).

The version lives in `Cargo.toml`. Shell completions and the man page are generated: `termgraph completions zsh`, `termgraph man`.
