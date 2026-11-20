# Center-out delivery v2 compiler rejection

Date: 2026-11-20

Status: retained semantic-gap evidence; not an admitted Rune, executable
projection, Chronos task, owner implementation, acceptance, or rollout approval

## Intent

The withdrawn v1 delivery Rune described holofractal recursion in prose but
encoded a linear additive sequence. The v2 candidate corrected the topology by
making one exact child computation explicit on the
`frontier-retained → child-returned` edge:

```lisp
(invoke pantheon-center-out-delivery-v2)
```

The candidate also separated external Rosetta-preservation review from Scry
admission; qualified frontier projection from Chronos retention; zero, multiple
and single eligibility; non-executable discoveries from executable packets; an
irreducible base leaf from a recursive child; singular evidence bind from
additive completion; pending human disposition from accepted, refused and
revision-required outcomes; and child qualification from root acceptance.

## Exact evidence

Rejected candidate:
`pantheon-center-out-delivery-v2.rejected.rune.txt`

- candidate SHA-256:
  `7355979f495fe75aba57e41e2fe89208da97c899ccb12bffe8ff7537cd0f7c69`
- Scry SHA-256:
  `cac27f87a01181d0b0075d752cc95cbecb8597b5091dfc0323a6345151df46b4`
- Smith SHA-256:
  `3288271a2dcae7bd833ddfea15a16562c5fb4335b16e0373e34c45bfa998504c`

The exact command was:

```text
../rune/src/ocaml/_build/default/scry.exe design/runes/pantheon-center-out-delivery-v2.machine.rune
```

Scry returned exit code `3` and the closed-grammar refusal:

```text
scry: 1:1: node rune: edge frontier-retained→child-returned: unknown clause (invoke …) — the grammar is closed (skip not)
```

The earlier `.candidate.rune` filename was also correctly refused because
`candidate` is not an admitted Rune shape. Candidate status therefore lives in
Rosetta/evidence, while compilable source uses a recognized shape.

## Interpretation

This is not permission to delete invocation, aggregate a list of receipts,
hand-select a child in a wrapper, or call a linear Rust projection semantically
complete. Smith's typed IR and compatibility analysis know `invoke` and classify
invoked-machine evaluation as unsupported for Rust, but the current canonical
machine-dialect source grammar cannot express it on an edge. The stronger
source-to-IR path is unavailable for this machine.

The source audit confirms the boundary rather than inferring it from one error:

- `rune/src/ocaml/scry.ml:930-1005` parses machine edges and closes the allowed
  clause set without `invoke`;
- `rune/src/ocaml/scry.ml:1007-1024` describes recursive relations as a type-
  graph declaration whose closure proof is staged, not invoked execution;
- `rune/src/rust/shared/src/ir.rs:1121-1178` parses `invoke` on the lower typed
  transition IR;
- `rune/src/rust/smith/src/semantic_compatibility.rs:44-73` preserves invoked-
  machine evaluation only for Gleam and reports it unsupported elsewhere; and
- `rune/src/rust/smith/src/codegen_gleam.rs:63-106` gates a transition on a
  caller-provided invoked-machine accepting-halt predicate. That is useful prior
  art, but it is not yet a machine-dialect source path or a typed child receipt.

The smallest honest next substrate work is a Rune/Scry design decision and
qualification that either:

1. adds an explicit machine-edge invocation clause with closed grammar,
   canonical round-trip, typed-IR preservation, recursive-cycle/bound policy,
   mandatory-path semantics and negative tests; or
2. identifies an existing stronger canonical Rune construct that expresses the
   same singular child computation and typed return without semantic loss.

Only after Scry admits the invoked structure should Smith compatibility run. A
Rust refusal for `invoked-machine-evaluation` is then expected and remains
visible until Rust enforces it. Gleam support is prior art, not evidence that
Rust or the machine dialect is complete.

## Planning classification

This is a non-executable discovery candidate bound to the candidate governing
intent and the `chronos-rune-machine-planning-projection` bootstrap item. It is
not yet an independently admitted Chronos dependency because the qualified
frontier projector and governing-intent owner identity do not exist. The first
machine-backed planning cycle must review its parent relationship and either
admit it as a critical prerequisite, link it to an existing Rune semantic-
parity item, or retain an attributable rejection.
