# Workbench TUI

`workbench-tui` is an unpublished reusable terminal presentation host for
independent workspace peers. The sibling `workbench-tui-cli` package owns the
standalone executable, profile parsing, terminal lifecycle, and process exit;
the root package owns navigation, layout, focus, and global input routing. Each
attached peer renders only inside its assigned rectangle and retains its own
input and service authority.

The binary attaches a Chronos Hub and one Vellum Unix service socket. Add
`--chronos-workstream NAME DIRECTORY` for each extra Chronos service. With
Chronos focused, **w** switches workstreams and **/** searches observed tasks. It does not orchestrate agents, merge engine state or infer
task completion from edits. The host preserves Vellum's dirty/indeterminate exit
laws and never retries an uncertain effect. Chronos writes remain disabled
unless explicitly enabled; Vellum authority remains entirely service-owned.

The versioned workspace shell also names Overview, Agora, Moira, Daimon,
Grounding, Aegis and Hermes surfaces. Overview and the Chronos/Vellum Workspace
are live in the standalone binary. A workspace profile may attach Agora through
the selected private Unix client by supplying its socket and participant claim;
the daemon authenticates the connected OS peer and remains the authority owner.
A trusted host can instead supply an independently constructed `agora-tui`
component through `workbench_tui_cli::run_hosted_terminal_with_agora`. Other configured surfaces
render their exact integration gap and receive no input. `[` and `]` switch
surfaces; Tab switches focus only inside Workspace. See
[the gap ledger](WORKSPACE_GAPS.md) and [delivery roadmap](ROADMAP.md).

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  /private/tmp/chronos-service-directory \
  --vellum-service /private/tmp/vellum.sock
```

To load a bounded workspace profile instead, copy
`config/workspace.example.json`, adjust its relative service paths, and run:

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  --workspace-profile config/workspace.json
```

Validate profile structure and path resolution without contacting a service:

```sh
cargo run -p workbench-tui-cli --bin workbench-tui --locked --offline -- interactive \
  --workspace-profile config/workspace.json --check-profile
```

Profiles contain connection configuration, presentation selection and explicit
cross-service scope mappings. They cannot enable Chronos writes or carry grant
handles. Continue to pass `--chronos-allow-writes` at launch when intended.
Each configured peer may also carry a bounded `requires` list. Workbench checks
those exact identifiers against the peer-owned TUI manifest for the selected
interactive or headless mode before connecting. A match describes composition
support only; the service still owns authorization and may refuse the request.
The Agora participant value is an identity claim checked against the daemon's
authenticated peer context; it is neither a credential nor authority.
Unknown fields, versions, duplicate surfaces, control characters, excessive
workstreams/bindings and files larger than 64 KiB are refused.

Use `--chronos-profile PATH` in place of the initial service directory to
reopen a workspace saved by `chronos-tui --save-profile`. Relative service
paths resolve from the profile directory. Peer flags use the exact names in
the profile, and write access remains an explicit launch option.

Add `--chronos-allow-writes` only when the Chronos service was composed for
writes. `--chronos-watch` enables feed recovery on services started with
`--atomic-state --observer-history`. `--chronos-scheduling` exposes scheduling
controls. `--chronos-planning PATH` loads the same read-only Gantt source as
the standalone Chronos Hub. Press **7** to view the plan.
`--chronos-cancel-receipts` selects V3 cancellation on a writable
atomic service. Add `--vellum-credential PATH` for Vellum's optional local credential
handshake. Tab changes focus and Ctrl-Q requests whole-workbench exit. Add
Use the explicit `headless` command for a bounded, versioned JSON observation of every configured
Agora, Chronos, and Vellum peer. Each row names its peer-owned capability and
truthful status. Healthy rows remain visible when another peer is unavailable,
but the command refuses aggregate success. `--headless` and `--snapshot` remain
compatibility aliases for invocations without an explicit command.

A Vellum-owned integration host can call `workbench_tui_cli::run_hosted_terminal`
with a Chronos `Hub` and a factory for the Vellum service and handoff handler.
Editor commands and handoff requests share one service owner. The host resolves
the document association and authorizes capture, publication, and review.
The standalone socket CLI starts with handoff controls disabled.

Local coordinated source layout:

```text
gantbd/
  chronos/
  chronos-tui/
  vellum/
  vellum-tui/
  workbench-tui/
```

Run `cargo test --workspace --all-targets --locked --offline`. Nothing is published to
crates.io and no remote, release, signing or activation action is performed.
The machine source receipt records the current dirty coordinated closure as
unqualified; immutable-revision CI is intentionally deferred until those
inputs are committed and remotely reachable.

The source receipt now pins the committed Agora, Chronos, and Vellum TUI
boundaries used by this composition. Their reusable default presentations keep
standalone CLI/application concerns outside Workbench's dependency graph.
The owning engine working trees and remote reachability remain unqualified.

Run `python3 scripts/qualify-source.py WORKBENCH_COMMIT --live` to do both
from temporary archives of the candidate commits. The command reads the peer
pins from the source receipts and accepts full candidate commit IDs for
Chronos, Vellum, and Vellum TUI while evaluating an updated set. It emits the
tested revisions after success and leaves the sibling checkouts untouched.

The local `qualification/chronos-workbench` branches record the passing
immutable candidate, with Workbench at `693b68d9a29b59ed313b09589864e5b0da864c58`.
Its locked tests, dual-service tmux gate, and the Chronos capture integration's
hosted handoff tmux journey passed on macOS.

For attempt inspection, pass `--chronos-moira WORKSTREAM SOCKET SCOPE`
and optionally `--chronos-daimon WORKSTREAM SOCKET`. Use `Chronos` for the
primary workstream or the exact name supplied with `--chronos-workstream`.
With Chronos focused, press **5**, select an attempt, and press **x**.
Moira authorizes its receipt disclosure. Daimon authenticates and authorizes
a separate inspection when Moira supplies a recorded execution ID.

Run `sh tests/live/dual_service.sh` for local composition qualification.
It builds and starts temporary Chronos and Vellum services, reopens a saved
profile with a relative service path, and uses an isolated tmux session to
check workstream switching, offline recovery, focus routing, and exit.
