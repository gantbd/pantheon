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

## Pause state

The requested review pause ended when the human explicitly said to proceed.
The no-go remains authoritative for mature stakeholder-ensemble rollout, while
bounded implementation and qualification may continue. Any final rollout or
other reasonably contestable material decision still requires the named-human
gate described in the governing journey plan.
