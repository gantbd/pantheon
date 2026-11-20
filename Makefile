.PHONY: check hygiene fmt lint test doc run attach tui-demo qualify-emacs qualify-j1 qualify-j1-result qualify-rune-author-review qualify-rune-author qualify-rune-blueprint qualify-rune-packet-publication qualify-bounded-agent-development qualify-moira-participation qualify-moira-deliberation qualify-moira-local-ensemble qualify-task-conversation qualify-agora-shared-context qualify-hestia-artifact qualify-hestia-real-model qualify-hestia-real-model-diagnostic qualify-hestia-real-model-daimon qualify-hestia-model-bench

PANTHEON_RUN_ROOT ?= /private/tmp/pantheon-run
PANTHEON_RUN_DIRECTORY ?= $(PANTHEON_RUN_ROOT)/chronos
PANTHEON_RUN_VELLUM_SOCKET ?= $(PANTHEON_RUN_ROOT)/vellum.sock
PANTHEON_RUN_VELLUM_ROOT ?= $(CURDIR)/README.md
PANTHEON_TMUX_SESSION ?= pantheon-j1-next

run:
	mkdir -p $(PANTHEON_RUN_DIRECTORY) $(PANTHEON_RUN_ROOT)
	chmod 700 $(PANTHEON_RUN_DIRECTORY) $(PANTHEON_RUN_ROOT)
	cargo run -p pantheon-cli --bin pantheon --locked --offline -- interactive $(PANTHEON_RUN_DIRECTORY) \
		--host-chronos --chronos-allow-writes --chronos-watch \
		--vellum-service $(PANTHEON_RUN_VELLUM_SOCKET) --host-vellum \
		--vellum-root $(PANTHEON_RUN_VELLUM_ROOT) --vellum-allow-edit \
		--vellum-allow-save --vellum-allow-add-file

attach:
	@tmux has-session -t "$(PANTHEON_TMUX_SESSION)" 2>/dev/null || { \
		echo "Pantheon tmux session '$(PANTHEON_TMUX_SESSION)' is not running." >&2; \
		echo "Select an exact session with PANTHEON_TMUX_SESSION=<name>." >&2; \
		exit 1; \
	}
	@if [ -n "$$TMUX" ]; then \
		tmux switch-client -t "$(PANTHEON_TMUX_SESSION)"; \
	else \
		exec tmux attach-session -t "$(PANTHEON_TMUX_SESSION)"; \
	fi

tui-demo:
	./tests/live/tui_breadth.sh
	./tests/live/emacs_keybindings.sh
	./tests/live/tui_operational_depth.sh

qualify-emacs:
	./tests/live/emacs_keybindings.sh

qualify-j1:
	./tests/live/work_item_agent_journey.sh

qualify-j1-result:
	./tests/live/j1_agent_result_observe.sh

qualify-rune-author-review:
	./tests/live/rune_author_capability_review.sh

qualify-rune-author:
	./tests/live/rune_author_capability.sh

qualify-rune-blueprint:
	./tests/live/rune_blueprint.sh

qualify-rune-packet-publication:
	./tests/live/rune_packet_publication.sh

qualify-bounded-agent-development:
	./qualification/bounded-agent-development/qualify.sh

qualify-moira-participation:
	./qualification/moira-participation/qualify.sh

qualify-moira-deliberation:
	sh tests/live/moira_deliberation.sh

qualify-moira-local-ensemble:
	PANTHEON_LOCAL_MODEL_ENSEMBLE=1 ./tests/live/moira_local_ensemble.sh

qualify-task-conversation:
	./tests/live/task_conversation.sh

qualify-agora-shared-context:
	./tests/live/agora_shared_agent_context.sh

qualify-hestia-artifact:
	./tests/live/hestia_artifact_acquisition.sh

qualify-hestia-real-model:
	PANTHEON_LIVE_MODEL=1 ./tests/live/hestia_real_model.sh

qualify-hestia-real-model-diagnostic:
	PANTHEON_LIVE_MODEL=1 PANTHEON_REAL_MODEL_DIAGNOSTIC=1 ./tests/live/hestia_real_model.sh

qualify-hestia-real-model-daimon:
	PANTHEON_LIVE_MODEL=1 PANTHEON_REAL_MODEL_DAIMON=1 ./tests/live/hestia_real_model.sh

qualify-hestia-model-bench:
	PANTHEON_LOCAL_MODEL_BENCH=1 ./tests/live/hestia_model_bench.sh

check: hygiene fmt lint test doc

hygiene:
	./scripts/check-repository-hygiene.sh

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings

test:
	cargo test --workspace --all-targets --all-features --locked --offline

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline
