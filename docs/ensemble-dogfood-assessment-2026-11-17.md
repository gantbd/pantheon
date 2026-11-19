# Stakeholder-SME ensemble dogfood assessment — 2026-11-17

## Decision

**No-go for claiming an operationally mature stakeholder ensemble.** The slice
is a useful, technically healthy stakeholder-truth foundation and may continue
as an explicitly experimental contract, but it is not ready to support a human
rollout decision or to represent that a room contained the right stakeholders.

The strongest result was not consensus. The human requester challenged the
initial framing twice—first correcting “personas” to stakeholders, then making
equity and “Do we have the right ensemble?” primary. Those interventions exposed
material omissions that the agent/model path had not found independently.

## Exact slice exercised

- Moira now retains a member principal, constituency/domain digest,
  representation kind (`actual_stakeholder`, `authorized_representative`,
  `agent_modeled_perspective`, or `technical_sme`), accountable principal,
  mandate and conflict evidence separately from the optional persona lens.
- A plan retains affected-population, stakeholder-map, coverage, equity and
  assessor evidence plus a categorical `yes`, `no`, or `contested` answer to
  “Do we have the right ensemble?”
- `no` and `contested` plans remain inspectable but refuse substantive reports;
  they can terminate honestly as `WrongEnsemble`.
- Agent-modeled and technical-SME members cannot publish a stakeholder mandate.
- The durable checkpoint identity advanced from version 2 / `MOIRADL2` to
  version 3 / `MOIRADL3`, so prior bytes refuse instead of being reinterpreted.

Qualification evidence:

- Full Moira workspace tests passed, including the new admission/standing
  refusals, canonical persistence, wire, service, SQLite, recovery and the live
  Dendrite grounding test. Existing explicitly opt-in external/soak tests
  remained ignored.
- `grounded_workflow_ensemble_crosses_actual_live_provider` passed in 8.66s:
  Dendrite grounded four members, Daimon made four real LM Studio
  `qwen/qwen3-coder-next` invocations, Moira retained four attributed reports,
  and the plan became ready to resolve without auto-selection.
- `authorized_live_provider_workflow_crosses_distinct_aegis_agora_and_vellum`
  passed in 11.11s and crossed the distinct Aegis, Agora and Vellum owners.
- The exercised cohort was explicitly two **agent-modeled stakeholder
  perspectives** and two **technical SMEs**. It was not four actual
  stakeholders, and the run must not be described that way.
- The follow-on owner slice now computes content-free cohort correlation before
  dispatch, retains it with the governed harness result and projects it from
  `moira-methodical-local`. Shared principal/profile identity is explicitly
  `correlated`; distinct identities remain `not_established` and never become a
  claim of cognitive independence. Public tests cover both dispositions.

## Categorical assessment

| Category | Assessment | Evidence and implication |
| --- | --- | --- |
| Stakeholder truth | **Pass at retained core; partial end to end** | Persona can no longer stand in for stakeholder identity. Actual and authorized representation have stricter invariants. The runtime prompt/participation contracts do not yet carry the retained standing automatically. |
| “Right ensemble?” gate | **Partial / no-go** | `yes`, `no`, and `contested` are retained and non-admitted cohorts cannot report. However, `yes` is still supplied by configuration and validated only for nonzero evidence identities; no selected owner or authority proves who may assess or change cohort fitness. |
| Equity and coverage | **Partial / no-go** | Exact digests retain population, map, coverage and equity artifacts. There is no owner-native schema/projection for required, present, missing, excluded, unavailable, conflicted and overrepresented constituencies, accessible participation or burden/benefit distribution. A digest alone is not an equitable process. |
| Actual stakeholder participation | **Fail for this run** | The human requester materially corrected the design, but no typed Agora/Moira member report retained that human contribution. The two stakeholder members were agent-modeled and correctly labeled; neither claimed lived experience, consent or mandate. |
| Technical grounding | **Pass for the bounded slice** | Dendrite resolved exact Rune method state, fold/surface identities, concern roots and orthogonality evidence for all four members. Stakeholder constituency and equity are not yet inputs to Dendrite routing, so technical orthogonality must not be mistaken for representative coverage. |
| Independence and correlation | **Partial / no-go** | The original four executions used the same Qwen model, provider, principal and harness; different lenses/standing labels still provide no cognitive or organizational independence. Moira now retains and projects shared principal/profile/context-manifest facts and conservatively labels shared execution identity `correlated`, while distinct identities remain `not_established`. Provider/model identity, organizational control, training lineage and substantive cross-challenge are not yet included, and the original live cohort has not yet been rerun through this projection. |
| Context quality | **Partial / harness leak** | The live qualifier injected stakeholder standing through a hard-coded context-to-instruction mapping. The normal Moira→Daimon participation/prompt contract still binds persona and grounding but not stakeholder representation, constituency, mandate or conflicts. |
| Challenge quality | **Weak / unassessed model capability** | The provider was instructed to return exact schema content. This proves composition and attribution, not steelmanning, bidirectional challenge, disconfirming evidence, or model judgment. Model quality cannot be inferred from this run. |
| Actionability and dissent | **Structural pass; semantic partial** | Reports, evidence and dissent identities survive durable owner paths. The forced outputs did not produce meaningful findings, dispositions, rebuttals or a contested synthesis. |
| Agora collaboration | **Fail for room maturity** | The authorized path publishes only the selected member’s proposal. All member first passes, challenges and replies do not yet appear as attributed messages in one Agora room, so the run is not an ensemble working together in Agora. |
| Human gates and authority | **Fail / not exercised** | Aegis authorized the selected bounded effect, but no `HumanGateRequestV1`, presentation receipt or named-human response was exercised. Ensemble agreement still cannot authorize rollout, and this assessment is a human-facing no-go rather than an automated decision. |
| Vellum context/workspace | **Partial** | The selected result can cross the Vellum owner and recover exactly. Vellum is not yet used as the cohort’s shared, revision-pinned context workspace or as the retained home for the stakeholder map/equity evidence. |
| Operational usability | **Partial** | The executable path is real and fast, but it remains a qualification command with hard-coded identities. Provider/model, cohort, standing, evidence, room and gate configuration are not yet manageable through Pantheon’s operator UI. |
| Observability | **Partial / Moira gap narrowed** | Pass/fail, call count, owner receipts, durable state and content-free execution-cohort correlation are observable. Per-member latency/token/cost, provider/model lineage, cohort changes, admission rationale, cross-component correlation and traces are still not projected as the events/logs/metrics/traces needed for operational diagnosis. |
| Outcome usefulness | **High for architecture discovery; low for decision quality** | Dogfooding exposed stakeholder-standing, equity, prompt-boundary, Agora-breadth and protocol-version defects. It did not produce decision-grade stakeholder review content. |

## Failure and surprise classification

| Finding | Primary category | Secondary category |
| --- | --- | --- |
| Stakeholder standing did not reach the normal model prompt/participation binding | Harness/boundary | Context |
| Cohort fitness `yes` is caller-asserted with opaque evidence digests | Owner/authority contract | Harness |
| Equity and missing/excluded/overrepresented coverage have no typed owner artifact | Owner/domain contract | Presentation |
| Same provider/model/principal/harness served all four members; the original run predated the new correlation projection | Cohort design | Model correlation |
| Exact-output instruction prevents assessing reviewer reasoning quality | Harness/evaluation | Model assessment |
| Only the selected member publishes to Agora | Composition/harness | User experience |
| Human corrections are not retained as an attributed ensemble contribution | Composition/context | User experience |
| Deliberation public wire shape changed while the outer handshake still advertises protocol version 1 | Protocol boundary | Compatibility/release |
| Persistence version was correctly advanced and rejects old bytes | Positive owner behavior | Recovery |
| Real local-provider and cross-owner runs were fast and reliable | Positive provider/runtime behavior | Operations |

## Required next slice

1. Select and version an owner-native stakeholder-map/equity/coverage artifact;
   make the cohort admission assessor and allowed transitions explicit. Do not
   let configuration alone attest that the ensemble is right.
2. Introduce a versioned deliberation protocol/wire evolution rather than
   changing the V1 frame under a V1 handshake.
3. Carry stakeholder standing, representation limitations, mandate/conflicts and
   the exact cohort question through the normal Moira→Daimon participation and
   prompt contract; remove the live qualifier’s hard-coded mapping.
4. Publish every admitted first pass and cross-review as attributed Agora room
   events. Preserve model/provider/common-context correlation and prevent room
   chatter from becoming prompts without explicit admission.
5. Use Vellum for the revision-pinned question, stakeholder map, evidence and
   cohort rationale; let Chronos surface missing-human and contested-cohort
   items on the operator agenda.
6. Exercise at least one actual human stakeholder contribution and one modeled
   stakeholder perspective without conflating them, then exercise a supported
   human-advocate gate and named-human response.
7. Extend the now-present execution-cohort correlation with provider/model and
   organizational lineage, then add per-member and cross-owner Moira
   observations for latency, usage, refusals, cohort changes, gate state and
   outcome follow-up.
8. Repeat this categorical assessment with substantive, non-forced reviewer
   outputs and owner dispositions. Model reasoning quality remains unassessed
   until that run.

## 2026-11-18 heterogeneous-provider update

The substantive rerun now exists and narrows the no-go without reversing it.
Moira invoked four distinct LM Studio model/profile identities—Qwen3 Coder
Next, GPT-OSS 20B, Qwen3.6 35B-A3B and GPT-OSS 120B—against one exact question
and a read-only Vellum projection carrying the complete source SHA-256 and
selected line extents. Dendrite grounded the method/concern cohort, Daimon
invoked each provider independently, every run had zero tool budget, and Moira
accepted four structurally valid attributed reports in 161.86 seconds while
leaving resolution absent. The retained raw-response artifact has SHA-256
`c4544a30f1a5997212407899a602724b3711d5a63f3e7ca1c0579056b3909ff9`.

The reports are useful challenges, not verified findings. Qwen3 Coder Next's
symlink/path-traversal claim overlooks `symlink_metadata(...).is_file()`
rejecting symlinks. GPT-OSS 20B's duplicate-worker concern overlooks the single
worker slot per record. Qwen3.6 correctly identifies weak operator telemetry but
treats explicit private-directory and bounded-concurrency policy as defects.
GPT-OSS 120B identifies useful fault-injection work but recasts fail-closed
worker disconnect behavior as a retry defect without an authored retry policy.
The run therefore demonstrates a working independently attributed technical
discussion cohort and meaningful model diversity, while also demonstrating why
cross-review, falsification, stakeholder standing and human disposition remain
mandatory.

The Hestia small-model comparison is a sharper role-fit no-go: only SmolLM3's
response satisfied the member schema, and its substance was contradictory.
OLMo omitted required evidence fields, Granite appended transport punctuation
and alleged absent checks that were present, and Qwen2.5 fenced its JSON and
gave a generic conclusion. The raw-response artifact has SHA-256
`fae3e15432b1b54921fd0197ad30fda174908af128f4db2e4d8fe2f8ba4c2a0a`.
These models may still qualify for narrower verifier/triage/ideation tasks; they
are not interchangeable stakeholder or technical-review seats.

The first question remains “Do we have the right ensemble?” For final rollout,
the answer remains **no**: the cohort is still statically configured, actual
affected stakeholders and a named human advocate are absent, Agora does not yet
carry all first passes/rebuttals, and no human gate response exists. The next
slice should use this working cohort to generate bounded proposals while Moira
retains dissent and forces evidence-backed owner or human dispositions.

## Pause state

The requested review pause ended when the human explicitly said to proceed.
The no-go remains authoritative for mature stakeholder-ensemble rollout, while
bounded implementation and qualification may continue. Any final rollout or
other reasonably contestable material decision still requires the named-human
gate described in the governing journey plan.

## 2026-11-19 Hestia structured-output and testing update

The current Hestia-owned four-model cohort passed its locked graph, formatting,
and all-target compile preflight before any model started. Hestia admitted and
served SmolLM3 3B, OLMo2 1B, Granite 4 H Tiny, and Qwen2.5 Coder 1.5B, then
cleaned up every endpoint after the run. With Daimon plain-text responses,
Moira received four independently attributed raw outputs but admitted zero:
SmolLM split required claim fields across objects and contradicted itself,
OLMo emitted malformed JSON without required provenance, Granite appended
trailing punctuation, and Qwen wrapped an otherwise parseable object in
forbidden Markdown. The retained raw-response artifact has SHA-256
`b31c0a2859107be02b47b1de296bf4445e8d46da816b2897c7db6681904c7af8`.

A controlled rerun changed only those providers to Daimon's existing strict
JSON-schema response mode. All four runtimes again became Ready, but the first
provider returned `OutcomeUnknown` before Moira could retain a report. Daimon's
host contract currently exposes only `Receipt` or `OutcomeUnknown`, while the
OpenAI-compatible adapter erases transport, HTTP-status, schema, and parse
causes into the same outcome. The experiment was reverted rather than leaving
a failing qualification or weakening Moira's parser. This is evidence for the
existing `daimon-provider-uncertain-diagnostic-receipt` and
`daimon-structured-output-profile-negotiation` tasks, not evidence that every
provider supports one generic structured-output profile.

Daimon commit `ac00c51` then added a content-free diagnostic receipt without
changing `OutcomeUnknown`. An opt-in one-model Hestia gate replayed the
structured profile against the exact Qwen2.5 Coder 1.5B artifact: ordinary
completion succeeded, while the structured qualification was classified as
`invalid_structured_output` and retained only a response digest. Hestia drained
cleanly. This falsifies a blanket HTTP-schema-rejection explanation for Qwen;
it does not retroactively identify the original first-provider/SmolLM failure.

Categorically, model hosting, cleanup, locked graph resolution, and compilation
passed. Plain-text serialization failed for all four small models; the raw
substance also showed contradiction, missing grounding, or overconfidence.
The original first-provider structured failure remains unattributed, while the
subsequent Qwen run proves one invalid structured payload after HTTP success.
No report is decision evidence, no model is qualified for these stakeholder
seats, and no rollout disposition changed.

Chronos revision 59 now also admits
`grounded-instrumented-tdd-evidence-loop` with Tellus-grounding,
Dendrite-context, Oculus-telemetry, and test-evidence capabilities. Its intended
boundary is explicit: deterministic tests remain authoritative; Tellus retains
sources, assumptions, freshness, and the falsifiable invariant; Dendrite
supplies bounded context; Oculus correlates red, green, regression, and
surprise evidence. Grounding may explain why a test exists and what could
falsify it, but a model interpretation cannot become the assertion.
