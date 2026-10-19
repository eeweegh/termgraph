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
    cargo run --quiet -- --completions {{shell}}

man:
    cargo run --quiet -- --man

install:
    cargo install --path . --locked
    install -v -D --mode=755 --target-directory=${HOME}/.local/bin bin/tg

uninstall:
    cargo uninstall termgraph
    rm -f ${HOME}/.local/bin/tg
