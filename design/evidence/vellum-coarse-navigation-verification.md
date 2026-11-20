# Vellum coarse-navigation verification

Status: ordinary verification passed; independent adversarial challenge remains required

Date: 2026-11-20

## Exact lineage

- Delivery Rune: `30f52576156a5513ef0d0eaf6a9e870a706d9b3fb5fb8451fae45ab8d2251788`
- Delivery typed IR: `553be17e568a3f3ac93dd87e58024668725e2672965529ead7649181d118897a`
- Smith executable: `72a45a8348f82106e9d401090b44b98e38173284da3231b7ff497f3cf125dc77`
- Buffered delivery Rust: `9b5b37a2120322d75cfa82a51225f5bb649c09445e2e60931c6e459fc748abfe`
- Test-machine Rune: `da58b76a854f1fe4fd4ab6d492981520e09e01b8056123a1149054ae351e5dfa`
- Test-machine typed IR: `5e5b7d81b1d30e928c296832a1a30a3012a9f85b0cb00bce1e2afc3ec5eb07fa`
- Generated Rust test controller: `3bf0c65d8d698f397cbc920c473bedd00cae8dd2eafe5e832845210d30796d6d`

Scry accepted both machines. Smith reported complete Rust compatibility for the test machine with no unsupported semantics. An independent `rustc` invocation compiled the generated controller. Its private harness advanced through all seven compelled transitions, evaluated all fourteen pre/post obligations, reached `Complete` without a chooser, and refused a deliberately denied first precondition.

## Candidate identities

| Owner | File | SHA-256 |
| --- | --- | --- |
| Vellum | `host/vellum-editor/src/app.rs` | `8440bbb49a88e7df2c94300bd738b41894b38de30a4fb542a2dae4bd73d2cc72` |
| Vellum TUI | `src/service.rs` | `f7c6ba71b81bdb31fa47e9fb2a9b9a7d2379cccd24ca673e5679179613c5f4f6` |
| Vellum TUI | `src/terminal.rs` | `06aa934d7b4600dd729f62129081591ca6c4b169a1a92409b66b80291d245914` |
| Vellum TUI | `cli/src/main.rs` | `a4431ad1f3d7b3376a37f977def9f52e61f16264ee9dad077786af1ede7582b9` |
| Pantheon | `src/lib.rs` | `ac5d57137ff05936c8ab60c334b6bcbcb085ff1a29a340dd36275dbcdda5f2fb` |
| Pantheon | `cli/src/terminal.rs` | `5694587e2cb404dc209fcfc7a08b6dbdf4a88f2be0ecb5e507643822aa7f3774` |
| Pantheon | `cli/src/terminal/coarse_navigation_tests.rs` | `5c251480ad6501fdb2fb6ab2c936c786a8d473664650d8be4e5d8c93084064dd` |

## Deterministic evidence

- `cargo test -p vellum-editor --lib`: passed; the crate currently has no owner-local unit tests, so this is compilation evidence only.
- Vellum TUI `cargo test --lib`: 19 passed.
- Pantheon `cargo test -p pantheon-cli --lib forged_coarse_navigation_contract_reaches_complete_only_on_actual_evidence`: 1 passed, 129 filtered.
- Pantheon `cargo test -p pantheon-cli --lib`: 130 passed when permitted to create its private Unix sockets. The first sandboxed run passed 125 and denied five socket-owning tests; it is retained as a sandbox-policy observation, not a product failure.
- The forged adapter discharges all fourteen generated obligations against actual owner movement, physical and Emacs translation, modal/focus routing, viewport geometry, protected-state preservation, exact one-dispatch evidence, and the telemetry field allow-list. Adversarial inspection found that its first revision exercised only the `moved` evidence path despite the broader generated obligation. The corrected adapter requires separate, exactly-once traces for `moved`, `boundary`, semantic `refused`, and access `failed`; the forged controller refuses completion if any mapping or allow-list check fails.

The initial missing-API check produced 24 expected compiler errors, but it used a temporary hand-authored test and was withdrawn when the holofractal test rule was recognized. The forged test controller was compiled only after implementation was visible to the build, so this record does **not** claim a clean forged RED transcript.

## Author adversarial challenge

The implementing agent challenged the exact candidate before requesting independent review and found two real defects in its own evidence and composition boundary:

1. The first evidence adapter proved only `moved`, despite claiming the complete settlement vocabulary. The corrected adapter requires four distinct, exactly-once traces: `owner_moved`, `owner_boundary`, `owner_refused`, and `owner_failed`, each with the same bounded allow-list.
2. Non-peer surfaces retained the last Workspace focus. Consequently, a surface-local PageDown on Overview could reach Vellum when the stale focus was `Vellum`. `RoutedInput::Suppress` now closes that path: only the standalone Vellum surface or Vellum-focused Workspace can produce a Vellum route. The forged obligation attacks Overview with deliberately stale Vellum focus and requires suppression.

The oversized-viewport oracle was also strengthened. Its first form used a five-line fixture and therefore could not distinguish the required 512-line clamp from unbounded movement ending at the document boundary. The corrected owner test uses 600 logical lines, supplies `u16::MAX`, and requires the cursor to land exactly 511 transitions away.

After these corrections the forged controller and all 130 Pantheon CLI library tests pass. This author challenge is useful falsification evidence but does not impersonate the Rune's independent verifier requirement.

## Compiled terminal evidence

An isolated `pantheon-page-live` tmux run used a 40-line fixture at SHA-256 `f6fa4e3ae79098a1bbd492df25506c178a2d95038699c15ff448abc7a5b7dbad`.

- Initial document viewport: lines 00 through 23.
- Two physical `PageDown` inputs reached the final page; the third reported `Vellum document boundary`.
- Two physical `PageUp` inputs returned to lines 00 through 23; the third reported `Vellum document boundary`.
- Source stayed `Clean` and byte-identical.
- One live failure exposed incorrect host geometry: movement used 28 rows while the composed Vellum document body exposed 24. The candidate now derives the exact owner document viewport from shell, workspace, and Vellum chrome; the forged test fixes the 32-row expectations at 24 lines for Vellum and 22 for Workspace.
- The owner implementation was subsequently tightened from a full-document line-index allocation to a current-line plus directional scan of at most 511 transitions.

## Instrumentation and authority result

Pantheon emits one correlated, content-free pathway with normalized action identity and one of `owner_moved`, `boundary`, `accepted`, `refused`, or `failed`. The forged test rejects document text, paths, key/search text, cursor facts, viewport contents, or fields outside the allow-list. Telemetry is not consulted by navigation behavior. Page actions grant no edit, save, file, shell, provider, approval, or rollout authority.

The Activity overlay currently presents receipts rather than the underlying bounded trace. This is a presentation gap, not missing trace production, and remains outside this slice.

## Publication friction and adjacent work

Publishing the 915-line generated controller through the interactive Vellum TUI exposed two gaps. Normal paste exhausted the editor history; bracketed paste avoided that limit but changed LF bytes to CR bytes. A bounded headless Vellum editor publication preserved the exact generated hash and safely refused the first sandbox-denied write without partial bytes.

Vellum should therefore own a revision-bound change-set operation: preview a bounded unified diff against an exact document identity, revision, and base digest; apply all hunks atomically; refuse stale, out-of-root, malformed, oversized, partial, or authority-expanding changes; and return a content-minimal receipt. It must not become generic filesystem or shell authority. Destination creation needs its own explicit owner admission rather than being smuggled through patch application.

This gap is adjacent backlog, not a reason to weaken or delay the verified coarse-navigation contract.

## Remaining gate

The exact candidate still needs the Rune's independent adversarial challenge: focus spoofing, modal bypass, state tampering, false movement claims, evidence disclosure, action amplification, oversized viewport input, and authority escalation. Review readiness and any rollout decision remain human-gated after that evidence is conclusive.
