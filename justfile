root := env("HOME") + "/.local"

default: check

check: fmt-check lint test

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets --locked -- -D warnings

test:
    cargo test --locked

run *args:
    cargo run --release -- {{args}}

build:
    cargo build --release --locked

completions shell="zsh":
    cargo run --quiet -- completions {{shell}}

man:
    cargo run --quiet -- man

install:
    cargo install --path . --locked --root {{root}}
    mkdir -p {{root}}/bin
    {{root}}/bin/termgraph tg-script > {{root}}/bin/tg
    chmod 755 {{root}}/bin/tg

uninstall:
    cargo uninstall termgraph --root {{root}}
    rm -f {{root}}/bin/tg

demo:
    examples/demo.sh
