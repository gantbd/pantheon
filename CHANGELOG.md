# Changelog

## Unreleased

- Added the bounded Chronos/Vellum Ratatui composition root.
- Kept terminal lifecycle, focus, layout, and global exit routing host-owned.
- Repaired coordinated sibling-source paths after relocation under
  `github.com/gantbd`.
- Added a bounded versioned workspace profile with relative endpoints, exact
  Chronos peer bindings, explicit write enablement at launch, and a headless
  validation mode.
- Added closed Overview, Workspace, Agora, Moira, Daimon, Grounding, Aegis and
  Hermes navigation. Surfaces without a composable presentation peer expose an
  explicit gap and receive no input.
- Qualified profile navigation, optional attachment, focus isolation and Agora
  gap rendering against real Chronos and Vellum services in tmux.
- Composed a host-supplied independent `agora-tui` peer as the first live
  surface beyond Workspace while leaving standalone transport and credential
  selection with the deployment host.
- Added profile-driven Agora attachment through the separately packaged private
  Unix client. The client validates endpoint ownership/mode, negotiates bounded
  framing, and distinguishes not-sent requests from outcome-unknown replies.
- Qualified host-supplied and profile-driven Agora rendering in tmux, including
  a real private Unix handshake and request/response exchange with a bounded
  protocol fixture.
- Replaced the dual-service-only presentation snapshot with a versioned
  headless report over any configured subset of Agora, Chronos workstreams, and
  Vellum. Reports preserve per-peer capability, truncation, refusal, access
  uncertainty, and partial aggregate failure without constructing TUI state.
- Advanced the Vellum presentation input to the separately packaged standalone
  CLI boundary while retaining the reusable root presentation dependency.
- Extracted Workbench's executable, profile parser, terminal lifecycle, and
  hosted-service helpers into `workbench-tui-cli`; the root package is now the
  reusable shell and peer-presentation composition boundary.
- Advanced the Chronos peer input to its explicit, versioned,
  multi-workstream standalone headless contract.
- Added explicit `interactive` and `headless` Workbench commands while retaining
  the historical headless flags as compatibility aliases.
- Advanced the Agora and Vellum peer inputs to their versioned standalone
  `rooms.read.v2` and `workspace.inventory.v1` headless contracts.
- Recorded the qualified core-peer parity matrix separately from the still-open
  hosted service topology and immutable engine-source closure.
- Added mode-specific capability negotiation from bounded profile requirements
  to peer-owned Agora, Chronos, and Vellum presentation manifests before any
  service connection.
- Added the hosted-topology lifecycle core: concrete typed clients remain with
  peer integrations while a bounded registry owns reverse-order shutdown and
  explicit clean, refused, or outcome-unknown receipts.
- Advanced the Agora and Vellum standalone headless inputs to versioned failure
  envelopes that preserve refusal, not-sent, outcome-unknown, invalid reply or
  projection, and uncertain disconnect states instead of collapsing them into
  unstructured stderr.
- Added guarded hosted-topology assembly that rolls back already-started peers
  in reverse order and returns exact clean, refused, or outcome-unknown receipts
  when a later factory refuses startup.
- Added profile- and launch-selected native Chronos hosting for interactive and
  headless Workbench runs, using the real service owner, ordinary local client,
  native drain signal, joined shutdown, and exact lifecycle receipts.
- Added profile- and launch-selected native Vellum hosting with persistent
  identity, bounded workspace discovery, optional principal authentication,
  explicit edit/save ceilings, native shutdown control, and joined drain.
- Delayed hosted headless aggregate success until native shutdown completes and
  included reverse-order per-peer lifecycle receipts in the versioned report.
- Advanced the Chronos peer input to a manifest-derived versioned failure
  envelope for malformed headless configuration as well as service uncertainty.
- Advanced Agora and Vellum so selecting headless mode always yields one
  versioned refusal on malformed configuration and never falls through to an
  interactive presentation path.
- Made explicit Workbench headless profile, capability, and syntax refusals use
  the aggregate versioned envelope before any service or presentation startup.
- Added a typed deployment-owned Agora factory seam and qualified its lifecycle
  registry against the real local daemon and public Unix client under an
  explicitly test-only injected authority composition.
- Qualified a single interactive and headless mixed topology with attached
  Agora plus Workbench-owned Chronos and Vellum.
- Qualified one process and lock closure owning real Chronos, Vellum, and Agora
  services, with reverse clean drain, using a derived local source-identity
  patch and explicitly test-only Agora authority inputs.
- Advanced the exact Agora TUI source receipt to its standalone real-daemon
  interactive and headless qualification revision.
