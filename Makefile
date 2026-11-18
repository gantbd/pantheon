.PHONY: check fmt lint test doc run tui-demo qualify-emacs qualify-moira-participation qualify-task-conversation

PANTHEON_RUN_ROOT ?= /private/tmp/pantheon-run
PANTHEON_RUN_DIRECTORY ?= $(PANTHEON_RUN_ROOT)/chronos
PANTHEON_RUN_VELLUM_SOCKET ?= $(PANTHEON_RUN_ROOT)/vellum.sock
PANTHEON_RUN_VELLUM_ROOT ?= $(CURDIR)/README.md

run:
	mkdir -p $(PANTHEON_RUN_DIRECTORY) $(PANTHEON_RUN_ROOT)
	chmod 700 $(PANTHEON_RUN_DIRECTORY) $(PANTHEON_RUN_ROOT)
	cargo run -p pantheon-cli --bin pantheon --locked --offline -- interactive $(PANTHEON_RUN_DIRECTORY) \
		--host-chronos --chronos-allow-writes --chronos-watch \
		--vellum-service $(PANTHEON_RUN_VELLUM_SOCKET) --host-vellum \
		--vellum-root $(PANTHEON_RUN_VELLUM_ROOT) --vellum-allow-edit \
		--vellum-allow-save --vellum-allow-add-file

tui-demo:
	./tests/live/tui_breadth.sh
	./tests/live/emacs_keybindings.sh
	./tests/live/tui_operational_depth.sh

qualify-emacs:
	./tests/live/emacs_keybindings.sh

qualify-moira-participation:
	./qualification/moira-participation/qualify.sh

qualify-task-conversation:
	./tests/live/task_conversation.sh

check: fmt lint test doc

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked --offline

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline
