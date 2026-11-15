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
