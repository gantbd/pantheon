# Qwen Vellum-context review: single reviewer versus bounded ensemble

## Accountable conclusion

The bounded Qwen ensemble produced a better **inventory of concerns** than the
single Qwen review, especially around missing human stakeholders, human
go/no-go, actionable context-refusal feedback, common correlation, and
observability. It did not produce a more trustworthy decision. Repetition by
the same model amplified invented UUID, HTTP, JWT, cache, normalization,
signature, timestamp, and receipt-schema mechanisms. Cross-review removed most
of them only after the accountable integrator supplied authoritative contract
facts.

The ensemble is useful for implementation discovery after independent contract
verification. It is **no-go for rollout advice or go/no-go authority**. No
actual operator, affected user, accountable sponsor, security owner, or named
human approver participated.

## Cohort and standing

The first question was “Do we have the right ensemble?” The honest answer was
`contested` for bounded implementation review and `no` for rollout:

| Member | Standing | Limitation |
| --- | --- | --- |
| Operator advocate | Agent-modeled stakeholder perspective | No lived-experience mandate or actual operator validation |
| Product/sponsor advocate | Agent-modeled stakeholder perspective | No executive authority or sponsorship mandate |
| Component-boundary reviewer | Technical SME | Same Qwen provider/model and common prompt |
| Reliability/observability reviewer | Technical SME | Same Qwen provider/model and common prompt |

All seven invocations—four first passes, two peer cross-reviews, and one
synthesis—used LM Studio `qwen/qwen3-coder-next` through Daimon's bounded
OpenAI-compatible adapter. Provider execution was healthy and attributed, but
the cohort had complete model/provider correlation.

The four first passes were sequential but isolated at the request contract:
each was a fresh Daimon invocation with `continuation: None`, its role prompt
and the common question only. No member received another member's output until
the two explicitly labeled cross-reviews. Sequential ordering is still weaker
cohort design than randomized concurrent execution, but it is not evidence of
an information channel by itself.

An additional concurrent canary control launched an operator prompt containing
only `amber-orchid-731` and a boundary-SME prompt containing only
`cobalt-lantern-284`. Each response repeated its own canary and neither emitted
the other's. Tightening the operator role to forbid implementation mechanisms
also stopped the UUID/string-revision invention, supporting weak role fidelity
and familiar-API completion priors rather than observed cross-request leakage.
The boundary member returned only its canary both concurrently and when rerun
alone; this rules out concurrency as the cause but exposes a different model or
prompt-compliance failure: a nonempty provider completion is not necessarily a
substantive review. The control detected no leakage; it does not prove LM
Studio internals are state-free.

This was not Pantheon's mature stakeholder-ensemble journey. Moira's current
live ensemble qualifier forces predetermined output and could not honestly
evaluate this arbitrary review. The review also did not publish first passes
and rebuttals into one Agora room. Those are harness/product gaps, not reasons
to describe independent provider calls as a completed operational ensemble.

## Comparison

| Dimension | Single reviewer | Bounded ensemble | Accountable assessment |
| --- | --- | --- | --- |
| Cohort fitness | Not assessed | Every first pass answered `contested` or `no` | Useful addition; actual humans still missing |
| Human gate | Not surfaced | Product/operator members required human go/no-go | Accepted; model agreement cannot authorize rollout |
| Exact-byte identity | Proposed normalization despite exact-byte prompt | Repeated normalization twice, then corrected in cross-review | Model/common-correlation failure; exact bytes remain authoritative |
| Owner boundary | Suggested cache and Vellum-specific generic receipt | Repeated cache/UUID/signed-receipt ideas, then rejected | Same-model overreach; no cache/fallback or generic-receipt specialization |
| Failure UX | Requested structured failures generally | Converged on actionable revision/unavailable orientation | Accepted product gap, but details must remain content-free |
| Verification | Recommended live drift test | Added explicit revision-zero and mismatch emphasis | Accepted; success gate and mismatch unit proof now exist |
| Observability | Limited | Added correlation and refusal diagnostics | Accepted direction; no invented timestamps/receipt hash |
| Decision quality | One concise but flawed review | Broader, noisier, internally corrective | Better discovery, not stronger authority |

## Accepted, rejected, and deferred findings

Accepted:

- strict offline schema/path/identity validation without peer contact;
- fresh authenticated Vellum reads with no static fallback or source cache;
- SHA-256 over exact `snapshot.text.as_bytes()` matching Vellum headless;
- exact revision comparison, including valid revision `0`;
- owner-neutral durable Daimon context receipts and redacted binding metadata in
  the provider catalog;
- headless and Ratatui projections sourced from the same receipt protocol;
- actionable, content-free operator orientation for unavailable or invalid
  context;
- explicit human go/no-go and actual stakeholder participation before rollout.

Rejected:

- whitespace/encoding normalization before hashing;
- cached context or an offline fallback mode;
- UUID, HTTP/JWT, signed-snapshot, string-revision, timestamp, or receipt-hash
  mechanisms absent from the owner contracts;
- Vellum-specific fields in Daimon's generic durable context receipt;
- the synthesis claim that the ensemble was fit to make a rollout go/no-go.

Deferred:

- whether Daimon's public refusal vocabulary should distinguish revision drift
  from other invalid evidence. The current owner-neutral public classes are
  `Unavailable`, `InvalidEvidence`, and `Limit`; any protocol expansion needs a
  compatibility and information-disclosure review rather than an improvised
  enum addition.

## Failure classification

| Finding | Primary category | Secondary category |
| --- | --- | --- |
| UUID/string-revision/HTTP/JWT inventions | Model overreach | Missing owner-contract context |
| Normalization and cache repeatedly resurfaced | Common-model correlation | Model overreach |
| Components mistaken for ensemble members | Prompt/model comprehension | Cohort-design confusion |
| Cross-review still claimed offline validation could detect runtime drift | Model reasoning | Boundary context |
| Final synthesis declared rollout go/no-go fitness without humans | Human-gate violation | Model overreach |
| Arbitrary question cannot enter governed Moira ensemble path | Harness/product gap | User experience |
| First passes/rebuttals were not retained in Agora | Composition gap | Observability |
| No actual operator or sponsor participated | Cohort-design gap | Human gate |
| Daimon provider calls were bounded and attributed | Positive harness behavior | Operations |

## Evidence and next slice

The implementation gate now proves authenticated Vellum owner read, exact
Vellum/Daimon digest equality, durable context receipt, Pantheon headless and
Ratatui inspection, and clean cross-owner drain. The focused unit contract
accepts revision `0` and refuses exact revision drift.

The next ensemble slice should make an arbitrary Vellum-pinned question a
first-class Moira plan, retain cohort fitness and common-model correlation,
publish each attributed first pass and cross-review into an Agora room, and
place the final recommendation on Chronos's human agenda. An actual operator
and accountable sponsor must then respond; the model ensemble cannot approve
its own rollout.
