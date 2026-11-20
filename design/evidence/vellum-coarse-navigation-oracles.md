# Vellum coarse-navigation oracle set

Status: verified; pending independent adversarial challenge and human review

## Lineage

- Rune source SHA-256: `30f52576156a5513ef0d0eaf6a9e870a706d9b3fb5fb8451fae45ab8d2251788`
- Typed IR SHA-256: `553be17e568a3f3ac93dd87e58024668725e2672965529ead7649181d118897a`
- Smith executable SHA-256: `72a45a8348f82106e9d401090b44b98e38173284da3231b7ff497f3cf125dc77`
- Buffered Rust projection SHA-256: `9b5b37a2120322d75cfa82a51225f5bb649c09445e2e60931c6e459fc748abfe`
- Projection result: Rust-compatible, provenance-bound, compiled by Smith and an independent `rustc` invocation. The delivery projection remains buffered; its separately forged test-machine projection was admitted through Vellum and exercised against the candidate.

The test-machine Rune SHA-256 is `da58b76a854f1fe4fd4ab6d492981520e09e01b8056123a1149054ae351e5dfa`, its typed-IR SHA-256 is `5e5b7d81b1d30e928c296832a1a30a3012a9f85b0cb00bce1e2afc3ec5eb07fa`, and its generated Rust SHA-256 is `3bf0c65d8d698f397cbc920c473bedd00cae8dd2eafe5e832845210d30796d6d`.

## Owner and behavior oracles

1. `PageUp` and `PageDown` are payload-free Vellum service actions. The host cannot choose a movement distance.
2. Vellum retains the latest observed viewport line count. It defaults to one and clamps every observation to `1..=512`.
3. One owner action moves at most `viewport_lines - 1` logical lines and never performs repeated host dispatch.
4. Movement clamps at the first and final logical lines and returns the content-free outcome `boundary` when the cursor does not move.
5. A successful displacement returns `moved`; the result does not contain cursor, line, path, or document data.
6. The preferred logical character column is retained across vertical coarse movement and used where the destination line permits it. Short destination lines clamp without erasing that preference.
7. Navigation preserves source bytes, revision/persistence projection, dirty state, document identity, selection membership, and save/history authority.
8. Terminal `PageUp` and `PageDown` translate to the corresponding owner actions. Emacs `M-v` and `C-v` translate to the same actions.
9. Pantheon's behavior facade grants surface-local routing only when Vellum owns the active input surface. Existing modal and global precedence remains authoritative.

## Threat and pathway oracles

| Threat | Falsifiable oracle |
| --- | --- |
| Spoofing | A page key while an overlay owns input is consumed by that overlay and cannot produce a Vellum action. |
| Tampering | Source and owner persistence witnesses are identical before and after both movement and boundary outcomes. |
| Repudiation | A dispatched page intent produces exactly one settled `moved`, `boundary`, `refused`, or `failed` pathway outcome. Missing or duplicate settlement is inconclusive, never success. |
| Information disclosure | Evidence is rejected if it contains document text, path, search/key text, cursor offset, cursor line/column, viewport contents, or unbounded error text. |
| Denial of service | An observed extent of zero yields the safe one-line default; an extent above 512 is clamped; one input produces one owner invocation. |
| Elevation of privilege | Page navigation issues no edit, save, file, shell, provider, rollout, or other service command. |

Allowed evidence fields are: schema/version, component, normalized action identity, outcome class, duration, and correlation identity. Telemetry is never consulted by behavior.

## Expected RED evidence

- Vellum tests initially fail to compile because `ServiceAction::PageUp`, `ServiceAction::PageDown`, viewport observation, and content-free owner outcomes do not exist.
- Vellum terminal translation tests initially fail because physical page keys and Emacs coarse-navigation chords do not map to semantic actions.
- Pantheon composition tests initially fail because its Vellum action classifier and profile translator are not exhaustive over the new owner variants.
- Existing modal-precedence tests are expected to remain green and are recorded as retained protection, not new behavior evidence.

End-state assertions remain independent of pathway evidence: a telemetry record cannot prove cursor movement or source immutability.
