# Makefile for termgraph

.PHONY: ignore clean install uninstall update test

ignore:
	@echo "This is a Makefile for termgraph. Use 'make install' to install the tool, 'make update' to update it, and 'make clean' to uninstall it."

clean:
	rm -rf .ruff_cache

uninstall:
	uv tool uninstall termgraph
	rm -f ${HOME}/.local/bin/tg

install:
	touch pyproject.toml
	uv tool install .
	install -v -D --mode=755 --target-directory=${HOME}/.local/bin bin/tg

update:
	touch pyproject.toml
	uv tool upgrade termgraph
	install -v -D --mode=755 --target-directory=${HOME}/.local/bin bin/tg

test:
	touch pyproject.toml
	uv tool run termgraph --version
	uv tool run termgraph --help
	uv tool run termgraph --colors
	printf "@,Boys\n2007,183\n2008,231\n" | uv tool run termgraph
	printf "@,Boys\n2007,183\n2008,231\n" | uv tool run termgraph --color red
	printf "@,Boys,Girls\n2007,183,202\n2008,231,199\n" | uv tool run termgraph --color blue red
	printf "@,Boys,Girls\n2007,183,202\n2008,231,199\n" | uv tool run termgraph --stacked

