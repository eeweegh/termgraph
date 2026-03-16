# Makefile for termgraph

.PHONY: ignore clean install update

ignore:
	@echo "This is a Makefile for termgraph. Use 'make install' to install the tool, 'make update' to update it, and 'make clean' to uninstall it."

clean:
	uv tool uninstall termgraph
	rm -f ${HOME}/.local/bin/tg

install:
	uv tool install .
	install -v -D --mode=755 --target-directory=${HOME}/.local/bin bin/tg

update:
	uv tool upgrade termgraph
	install -v -D --mode=755 --target-directory=${HOME}/.local/bin bin/tg

