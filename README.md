# Workbench TUI

`workbench-tui` is an unpublished composition root for independent Chronos and
Vellum Ratatui peers. One executable owns the terminal, split layout, focus and
global exit routing. Each peer renders only inside its assigned rectangle and
retains its own input and service authority.

The binary attaches one existing Chronos local service directory and one Vellum
Unix service socket. It does not orchestrate agents, merge engine state or infer
task completion from edits. The host preserves Vellum's dirty/indeterminate exit
laws and never retries an uncertain effect. Chronos writes remain disabled
unless explicitly enabled; Vellum authority remains entirely service-owned.

```sh
cargo run --locked --offline -- \
  /private/tmp/chronos-service-directory \
  --vellum-service /private/tmp/vellum.sock
```

Add `--chronos-allow-writes` only when the Chronos service was composed for
writes. Add `--vellum-credential PATH` for Vellum's optional local credential
handshake. Tab changes focus and Ctrl-Q requests whole-workbench exit. Add
`--snapshot` for a bounded headless attachment probe against both services.

Local coordinated source layout:

```text
Workspace/
  local/
    vellum/
    vellum-tui/
    workbench-tui/
  github.com/gantbd/
    chronos/
    chronos-tui/
```

Run `cargo test --all-targets --locked --offline`. Nothing is published to
crates.io and no remote, release, signing or activation action is performed.
