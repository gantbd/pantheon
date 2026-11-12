# Workbench TUI

`workbench-tui` is an unpublished composition proof for independent Chronos and
Vellum Ratatui peers. One executable owns the terminal, split layout, focus and
global exit routing. Each peer renders only inside its assigned rectangle and
retains its own input and service authority.

The current binary is deliberately an unattached qualification shell. It does
not connect either service, orchestrate agents, merge engine state, infer task
completion from edits, or claim a production workbench. Its tests prove that
both presentation libraries coexist in one frame and that the host routes an
event to only the focused peer.

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
