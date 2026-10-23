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

# bump 1.6.2: set the version, commit "Bump version to 1.6.2", tag v1.6.2 and push both (needs a clean tree)
bump version:
    #!/usr/bin/env bash
    set -euo pipefail
    [[ "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must look like 1.2.3" >&2; exit 1; }
    git diff --quiet && git diff --cached --quiet || { echo "working tree is not clean" >&2; exit 1; }
    perl -0pi -e 's/^version = "[^"]*"/version = "{{version}}"/m' Cargo.toml
    cargo update -p termgraph --offline --quiet
    git commit --quiet -am "Bump version to {{version}}"
    git tag -a "v{{version}}" -m "termgraph {{version}}"
    git push origin HEAD "v{{version}}"
