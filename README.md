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
component through `workbench_tui_cli::run_hosted_terminal_with_agora`. Moira,
Daimon, Hermes, Aegis, and Grounding profiles attach their peer-owned read-only
components. Grounding uses Dendrite's typed presentation endpoint rather than
MCP tool envelopes or text. `[` and `]` switch
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
Chronos and Vellum profiles may select `"deployment":"hosted"`. Workbench then
owns their real local service lifecycles while the same peer clients and TUI
components are used. Hosted Vellum requires a bounded non-empty `roots` list;
attached profiles must not include roots. Edit and save authority remain
launch-only through `--vellum-allow-edit` and `--vellum-allow-save`.
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
handshake. In hosted mode the same private record configures both the daemon
verifier and client claim provider without placing credential material in the
profile or lifecycle receipt. Tab changes focus and Ctrl-Q requests
whole-workbench exit.
Use the explicit `headless` command for a bounded, versioned JSON observation of every configured
Agora, Chronos, Vellum, Moira, Daimon, Hermes, Aegis, and Grounding peer. Each row names its peer-owned capability and
truthful status. Healthy rows remain visible when another peer is unavailable,
but the command refuses aggregate success. `--headless` and `--snapshot` remain
compatibility aliases for invocations without an explicit command.

A Vellum-owned integration host can also call `workbench_tui_cli::run_hosted_terminal`
with a Chronos `Hub` and a factory for the Vellum service and handoff handler.
Editor commands and handoff requests share one service owner. The host resolves
the document association and authorizes capture, publication, and review.
The standalone socket CLI starts with handoff controls disabled.

The Workbench executable can natively host Chronos and Vellum together in
interactive or headless mode. Services bind before clients are constructed;
startup refusal rolls back earlier owners, and shutdown drains in reverse order
with explicit clean, refused, or outcome-unknown receipts. Agora remains an
attached or host-injected peer until a deployment supplies its required Aegis
authentication, policy, clock, and durable-host composition.

The workspace-wide standalone/Workbench pattern, composition taxonomy, current
parity matrix and Axon/Engram prior-art routing live in
[`docs/composition/`](docs/composition/README.md). Peer roadmaps continue to own
their behavior; this repository owns product composition and its evidence. The
closed machine-readable parity matrix is [`composition-status.json`](composition-status.json).

Local coordinated source layout:

```text
gantbd/
  chronos/
  chronos-tui/
  daimon/
  daimon-tui/
  hermes/
  hermes-tui/
  aegis/
  aegis-tui/
  dendrite/
  grounding-tui/
  moira/
  moira-tui/
  vellum/
  vellum-tui/
  workbench-tui/
```

Run `cargo test --workspace --all-targets --locked --offline`. Nothing is published to
crates.io and no remote, release, signing or activation action is performed.
The machine source receipt records the current dirty coordinated closure as
unqualified; immutable-revision CI is intentionally deferred until those
inputs are committed and remotely reachable.

The source receipt now pins the committed Aegis, Agora, Chronos, Daimon, Grounding, Hermes, Moira, and Vellum TUI
boundaries used by this composition. Their reusable default presentations keep
standalone CLI/application concerns outside Workbench's dependency graph.
Remaining owner working trees and remote reachability remain unqualified.

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
It first builds and attaches temporary Agora, Chronos, and Vellum services,
reopens a saved profile with relative service paths, and uses an isolated tmux
session to check workstream switching, offline recovery, focus routing, all
three peer panes, and exit. It then restarts Chronos and Vellum under Workbench
ownership while attaching Agora, qualifies the mixed topology's headless
snapshot and interactive panes, and proves reverse shutdown removes every
Workbench-owned data and control endpoint.

Run `sh qualification/agora-host/qualify.sh` to qualify one Workbench process
owning Chronos, Vellum, and the real Agora local daemon while serving Agora's
public Unix client. The same process observes all three peers through the
headless aggregation API used by the standalone Workbench CLI. The nested lock
closure uses an explicitly test-only Agora authority composition. Its command derives a scoped Cargo source patch because
Chronos standalone and Agora resolve the compatible checkpoint/commit subset of
`keystone-storage` from different local identities. It qualifies the all-three lifecycle graph,
but not the production Agora authority factory or a normal unpatched release
build.

Run `sh tests/live/aegis_attach.sh` to qualify the independently packaged
read-only Aegis peer through the real durable daemon in both Workbench headless
and interactive attachment, including bounded grant/decision page navigation.
The gate exercises `authority.observe.v1` plus the peer-owned
`authority.revoke-epoch.v1` and `authority.attenuate-authorize-once.v1` guards.
Workbench gains no ambient authority: separately invoked commands require the
exact current epoch and literal confirmation, attempt each mutation once, and
never retry an uncertain result. The bounded authority journey keeps its parent
and child handles inside one authenticated connection and returns only redacted
authorization facts. Reusable issue and attenuation management remain unavailable.

Run `sh tests/live/grounding_attach.sh` to qualify the independently packaged
Grounding peer through Dendrite's typed authenticated observation endpoint in
both Workbench modes. This uses `grounding.observe.v1`, not Dendrite's agent-facing
MCP boundary.

Run `sh tests/live/moira_attach.sh` to qualify Moira's content-free census in
headless mode and the same reusable component's bounded selected-saga evidence
and operator receipts interactively. The gate uses only `SagaReadOnlyClient`;
it grants Workbench no reconciliation authority.

Run `sh tests/live/hermes_attach.sh` to qualify configured stream health and one
configured consumer's content-free cursor/backlog through Hermes's read-only
facade. Exact opaque selectors are profile inputs and are never catalogued or
echoed. The same gate separately qualifies `ledger.events.disclose.v1` with an
exact stream/start/limit confirmation, a 16-event ceiling, and the negotiated
frame bound. Routine observation remains content-free; Workbench receives no
append or compaction authority. The gate also qualifies
`ledger.acknowledge.v1`: standalone and Workbench reuse one peer-owned guard
that observes the expected cursor and attempts one exact-confirmed
acknowledgement on the same connection. Consumer authority remains outside the
reusable component and hosted read-only daemon.

Run `sh tests/live/hermes_hosted.sh` to qualify Workbench-owned lifecycle for
Hermes's standalone read-only daemon. The profile supplies explicit
create/recover mode, daemon path, durable directory, socket, OS peer identity and
subject. Workbench waits for the owner socket, observes through the same public
client used for attachment in headless and interactive modes, requests signal
drain, verifies clean shutdown, and reopens the same ledger after each path. It
links no Hermes engine/service or Keystone crate and does not acquire append,
acknowledge or compaction authority.

Run `sh tests/live/operational_attach.sh` to qualify the real Moira, Daimon,
Hermes, and Aegis daemons plus Dendrite's typed Grounding service in one
Workbench headless invocation. This coexistence gate exercises only the five
peer-owned observation capabilities; it does not claim native hosting or
mutation authority for those peers.
