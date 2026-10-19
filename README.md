# termgraph

Draw basic bar graphs in the terminal. Rust rewrite of a trimmed-down fork of [mkaz/termgraph](https://github.com/mkaz/termgraph).

```
printf '@,Boys,Girls\n2007,183,202\n2008,231,199\n' | termgraph --stacked
termgraph data.csv --width 40 --color blue red
```

Input is a file or stdin: an optional `@,Cat1,Cat2` header, then `label,value[,value...]` rows; `#` lines and blank lines are skipped. Columns split on the delimiter (outside quotes), else on whitespace.
Empty input, or a bare invocation on a terminal, exits silently with status 0. Errors go to stderr as `termgraph: error: ...` with status 1.

`termgraph --help` lists every option. `--format` takes one printf conversion (`%5.2f`, `%.1f%%`, `%d`), not a Python format spec.

`bin/tg` is a zsh driver that reads CSV with a header through [xan](https://github.com/medialab/xan) and picks single or stacked output. It is embedded in the binary; `just install` writes it to `~/.local/bin`, or from a plain cargo install:

```
cargo install --git https://github.com/eeweegh/termgraph
termgraph --tg-script > ~/.local/bin/tg && chmod 755 ~/.local/bin/tg
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

The version lives in `Cargo.toml`. Shell completions and the man page are generated: `termgraph --completions zsh`, `termgraph --man`.
