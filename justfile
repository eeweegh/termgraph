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
    cargo install --path . --locked --root ${HOME}/.local
    mkdir -p ${HOME}/.local/bin
    ${CARGO_HOME:-$HOME/.cargo}/bin/termgraph tg-script > ${HOME}/.local/bin/tg
    chmod 755 ${HOME}/.local/bin/tg

uninstall:
    cargo uninstall termgraph --root ${HOME}/.local
    rm -f ${HOME}/.local/bin/tg

demo:
    examples/demo.sh
