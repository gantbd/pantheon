# Pantheon use cases and user journeys

Status: research blueprint
Research date: 2026-11-17
Scope: Pantheon, its configured peer surfaces, their aggregate journeys, and the
shared Keystone laws that constrain composition.

## Purpose

This document defines the product journeys Pantheon should support and the
capabilities required to support them without collapsing the authority of its
sovereign peers. It combines the current repository contracts with primary
standards, foundational research, and current industry precedents.

It is a product and architecture blueprint, not an assertion that every listed
capability is implemented. The claim vocabulary is:

- **Qualified**: present in the current Pantheon composition status with cited
  live evidence.
- **Peer-qualified**: present in the owning peer but not yet composed by
  Pantheon.
- **Required**: necessary for a coherent target journey.
- **Later**: valuable only after the owner, deployment, or authority model is
  selected.
- **Non-goal**: a tempting coupling that would violate component ownership or
  make evidence dishonest.

The current implementation baseline is
[`composition-status.json`](../composition-status.json); detailed peer behavior
remains owned by the sibling repositories. The architectural boundary is
[`COMPONENT_ARCHITECTURE.md`](../COMPONENT_ARCHITECTURE.md), and current
composition delivery is tracked in
[`docs/composition/backlog.md`](composition/backlog.md).

## Executive conclusions

1. **Pantheon is a workbench and control plane, not a universal engine.** Its
   unique value is a coherent human experience over separately authoritative
   conversation, work, workflow, execution, documents, events, authority, and
   grounding.
2. **The primary aggregate object is a journey, not a shared record.** A ticket,
   room, saga, agent session, document, and event retain different identities.
   Pantheon needs typed links and a journey projection, not one mutable
   cross-component object.
3. **Evidence must remain typed by domain.** A chat message is not task
   admission; an agent response is not a completed effect; a task cancellation
   does not prove an already-published message was withdrawn; telemetry is not a
   receipt; and model consensus is neither truth nor authority.
4. **The cornerstone journey is work-to-agent-to-room-to-artifact.** A human
   admits work, authorizes a bounded attempt, observes an agent session, steers
   it through a task-linked room, reviews grounded artifacts, and accepts or
   rejects independently receipted effects.
5. **Recovery is part of every journey.** Every mutating step needs stable
   operation identity, exact-input replay, outcome-unknown fencing, historical
   lookup where supported, and an operator-visible next safe action.
6. **The largest present product gap is not another engine feature.** It is the
   aggregate projection of typed relationships and Moira's peer-qualified agent
   participation/session/event/prompt/permission surfaces, which are currently
   absent from Pantheon.
7. **Human interaction needs three planes kept visually distinct:** observation
   (what is known), control (what the user is asking an owner to do), and
   management (which processes/topology Pantheon owns). Authorization is an
   independent peer decision, not a fourth UI synonym for confirmation.
8. **Personas and work modes are independent axes.** A reviewer may brainstorm,
   model architecture, test a prototype, investigate an incident, or extract
   insight. Pantheon must preserve context and evidence across those transitions
   without forcing every kind of work into a delivery pipeline.
9. **Federation is explicit topology, not transparent remoting.** Components may
   reside on different hosts and multiple instances may cooperate, but each
   logical authority needs a selected home, shard, consensus group, replica law,
   or worker-pool contract. A network address, discovered endpoint, or healthy
   replica is never by itself semantic authority.
10. **Analysis and ambient expertise are first-class, inspectable capabilities.**
    Users and agents need bounded data/query/graph exploration plus Dendrite-
    routed SME guidance at ambient, activated and reference depth. Queries,
    visualizations and expert recommendations remain derived evidence: their
    source, version, assumptions, execution and influence must be reviewable.
11. **Presentation lifetime is independent of work lifetime.** TUI and CLI
    clients must attach to, observe, control and detach from existing authorized
    component resources and long-running sessions. Detach ends a client
    attachment only; cancel, close, leave, acknowledge, shutdown and drain remain
    separate owner commands. Every interactive capability needs bounded headless
    semantics, not necessarily an identical presentation.
12. **Events, logs, metrics, traces, alerts and domain receipts are different
    evidence.** Pantheon needs a correlated operational inquiry experience over
    all of them, but each signal retains its source, population, sampling,
    retention and semantic limits. Telemetry may explain or direct attention; it
    cannot prove authorization, durable effects, workflow completion or absence.
13. **Human-agent parity is participation parity, not authority equality.** Every
    work mode admitted now or later—not merely the illustrative catalogue in
    this document—must expose semantically equivalent human and agent
    participation. An agent remains a non-human principal with explicit
    delegation, budgets, provenance and an accountable human or institution;
    policy may reserve approval, risk acceptance, redress and trust-root actions.
14. **Pantheon must be able to improve Pantheon without trusting candidate code
    to govern itself.** An agent may develop, package and propose a replacement,
    then observe it in sandbox, shadow, canary or bounded production operation.
    Promotion, traffic switching, telemetry, safety enforcement and rollback
    remain independently controlled, fail closed on uncertainty, and preserve a
    known-good recovery path outside the component being changed. This remains
    architectural direction—not an implementable profile—until sovereign build,
    deployment, observation and recovery owners are selected and qualified.
15. **Material work receives productively adversarial stakeholder review.**
    Dendrite grounds and checks the concern coverage and orthogonality of the SME
    lenses; Moira frames and retains the bounded deliberation, independent
    reports, evidence, dissent and resolution. The purpose is to expose blind
    spots and improve the work, not manufacture consensus, votes or approval.
16. **Final rollout is a human go/no-go, and credible human-advocacy concerns
    can require human gating elsewhere.** No candidate becomes the authoritative
    or default production version without presentation to and an explicit
    decision from a named human principal. For any other material decision, an
    ensemble's supported, non-frivolous human-advocate finding can require the
    exact decision to pause for human adjudication. The advocate finding triggers
    presentation; it is not itself approval, denial, authority or an agent veto.

## The product model

### Persona and participation model

Pantheon should describe a person's current participation in a journey rather
than assign one permanent persona. One person may request an outcome, build the
automation, approve a sensitive effect, and later operate it, although policy
may require those roles to be separated. The UI must show which role and
authority are active at each consequential action.

| Persona | Primary intent | Characteristic capabilities |
| --- | --- | --- |
| Requester / stakeholder | Express a desired outcome and accept or reject the result | Structured intake, clarification, progress and decision visibility, acceptance criteria, outcome feedback |
| Individual contributor | Perform and deliver the work | Personal queue, rooms, documents, agent collaboration, local control, evidence and handoff |
| Analyst / decision scientist | Turn data and relationships into testable findings and decision evidence | Source discovery, profiling, query, graph traversal, statistics, visualization, reproducible analysis, uncertainty and lineage |
| Work coordinator | Turn demand into tractable, ordered work | Triage, decomposition, assignment, dependencies, capacity, escalation, cross-workstream linking |
| Team / portfolio owner | Align work with objectives, constraints, and capacity | Objectives, scenarios, prioritization, dependency and risk views, outcome and investment review |
| Subject-matter expert | Supply and validate domain knowledge | Scoped participation, source contribution, claim review, contradiction resolution, provenance |
| Reviewer | Evaluate quality and fitness without owning the effect | Review packages, comparisons, comments, requested changes, independent test and evidence inspection |
| Approver / accountable owner | Authorize a risk, expense, publication, or transition | Exact-effect preview, policy context, separation of duties, approve/deny/expire, signed rationale |
| Agent and workflow builder | Design reusable automated behavior | Definitions, tools, prompts, budgets, workflow composition, simulation, evaluation, versioning and promotion |
| Knowledge curator | Maintain trustworthy grounding surfaces | Source admission, classification, freshness, lineage, conflict handling, coverage and retirement |
| Automation owner | Remain accountable for a running automation | SLOs, budget, quality, drift, incidents, change control, pause and retirement |
| Operator / incident responder | Keep the system safe and available | Health, alerts, diagnosis, containment, recovery, reconciliation, handoff and post-incident review |
| Observability engineer / telemetry steward | Make system behavior measurable without corrupting semantics, privacy or cost | Instrument catalogue, schemas, pipelines, logs/metrics/traces/events, SLOs, alerts, cardinality, sampling, retention and quality |
| Platform, security, or data administrator | Govern deployment and bounded authority | Topology, membership, policy, credentials, classification, retention, residency, backup and upgrade |
| Auditor / investigator | Reconstruct what was known, decided, authorized, and effected | Immutable references, manifests, gaps/nonclaims, policy versions, export and verification |
| External collaborator / recipient | Contribute to or receive only selected work | Invitation, scoped room/task/artifact access, review or acceptance, expiry and revocation |
| Affected person | Understand, challenge, or correct an automated action affecting them | Notice, explanation, evidence access, correction, appeal, human review and redress |

Agent teams, scheduled automations, peer services, identity providers, webhook
producers, and hostile or compromised actors belong in threat and journey actor
models, but they are not human personas. Agents are nevertheless first-class,
attributable journey participants as defined below; no agent may impersonate a
person or inherit human accountability merely by selecting a role label.

### Human-agent role and goal parity

Every work mode the system can admit—including organization-defined, domain-
specific and future modes absent from this document—must be usable by an
authorized human, an authorized agent, or a mixed team through the same owner-
native records, typed links, evidence vocabulary, headless contracts and
recovery states. The parity test is: given the same declared role, scope and
authority, do both participants have a semantically equivalent way to propose,
inspect, execute, explain, challenge, hand off and recover the work?
Presentation may differ; meaning may not.

Role selection describes the participant's current responsibility. It never
grants authority. An agent role activation binds agent/session identity, model
and tool profile, objective, work mode, exact resource scope, allowed actions,
budgets, grounding digest, time bound, accountable owner and policy epoch.
Changing roles creates a recorded transition with rationale and re-evaluation;
it cannot amplify the existing grant. A creative agent can later become an
architect or operator only after the new role's constraints are resolved. The
activation and transition history is a durable owner record with optimistic
revision/replay semantics, never reconstructed from current UI state, room text,
agent prose, organizational title or an authority grant.

| Participation class | Human and agent capability | Accountability boundary |
| --- | --- | --- |
| Symmetric contribution | Ideate, analyze, query, model, draft, prototype, implement, test, review evidence, document, monitor, diagnose and recommend | Every contribution identifies its actual human or agent author, sources, transformations, uncertainty and current role |
| Delegated coordination | Triage, decompose, schedule, facilitate, route, summarize, request review, run bounded workflows and operate within a runbook | The delegation names resources, actions, budget, duration, escalation and accountable owner; organizational seniority is not synthesized for an agent |
| Qualified consequential action | Package, sign through a separate service, publish, deploy, alter traffic, contain, restart or roll back when the selected risk policy permits | Current Aegis and owner checks, exact-effect preview, separation of duties and durable receipts apply equally; the agent cannot approve its own requested effect |
| Human/institution-reserved accountability | Accept residual risk, make legal or fiduciary attestations, decide affected-person redress, exercise designated break-glass authority, or change the independent trust/recovery root | Agents may assemble evidence and recommend; the named accountable person or institution makes the decision unless a deployment profile explicitly and lawfully assigns it elsewhere |

Required uniformity:

- Human and agent participants can enter any work mode, branch or loop, attach
  to the same task/room/workflow/session/artifact, and hand work to one another.
- The UI, TUI and CLI show active role, principal type, delegation, accountable
  owner, authority/budget horizon and whether output is proposal, observation,
  decision, approval or effect.
- Rooms support human steering, questions, pause, interruption, correction,
  takeover and handback throughout agent work. An agent cannot hide a live
  consequential action behind an offline summary.
- Agent-created agents, workflows, policies, tools or code inherit no creator
  authority. Each is a new versioned subject/artifact admitted under its own
  policy and provenance.
- Evaluation applies comparable task and outcome criteria without pretending
  humans and agents have the same cognition, privacy rights, obligations or
  failure modes. Private chain-of-thought is not required; inspectable plans,
  decisions, sources, tool calls, state transitions and concise rationale are.
- A new/custom work mode declares its semantic owner, purpose, inputs, outputs,
  lifecycle/transitions, evidence and uncertainty vocabulary, authority/effect
  boundaries, stakeholder-review triggers, recovery, retention/disclosure and
  interactive/headless projections. Registration extends the catalogue; it does
  not require Pantheon to reinterpret the mode as one of the examples below.

### Organizational altitude and external participation

Pantheon must serve people from individual execution through executive
governance and outside participation using projections of the same underlying
journeys. Organizational altitude changes scope, time horizon, decision rights,
language, and evidence density; it must not create a parallel status truth.

| Altitude / participant | Questions Pantheon must answer | Appropriate experience and decisions |
| --- | --- | --- |
| Individual contributor | What should I do next? What context, tools, constraints and help do I have? What changed? | Personal queue, focused room/session/artifact, exact dependencies, safe actions, local feedback, handoff and learning |
| Senior IC / technical lead | Are the approach, interfaces, quality and risks coherent across contributors? Where is intervention needed? | Design and review queues, architectural context, dependency/risk hotspots, standards, evidence, mentoring and technical decisions |
| Team lead / manager | Is the team pursuing the right outcomes at sustainable flow? What is blocked, overloaded or at risk? | Outcome-linked work, capacity/WIP, aging, dependencies, quality, incidents, staffing constraints, escalation and coaching signals |
| Program / product / department leader | Are coordinated initiatives delivering intended outcomes? Which tradeoffs and cross-team dependencies need decisions? | Initiative and outcome maps, scenario plans, milestones, investment/capacity, dependency chains, risk, customer evidence and decision agenda |
| Executive | Are strategy, investment, execution, exposure and outcomes aligned? What changed materially and what decision is required? | Concise exception- and decision-oriented briefs, trends and scenarios, strategic assumptions, material risks, commitments, outcomes and accountable drill-down |
| Board / owner / investor | Is governance effective and is the organization operating within agreed strategy, risk and stewardship boundaries? | Curated period reports, material decisions/events, assurance, risk posture, outcome trends and independently verifiable evidence under strict disclosure policy |
| Customer / end-user representative | Is the need understood, what commitment was made, what is available, and how can I provide feedback or seek remedy? | Scoped roadmap/commitment, delivery and service status, acceptance, feedback, incident communication, explanation, appeal and redress |
| Partner / supplier / contractor | What must we exchange, by when, under which interface, quality, security and acceptance contract? | Shared milestones, specifications, rooms, deliverables, dependencies, attestations, acceptance and bounded incident coordination |
| Regulator / assessor / auditor | What controls and decisions applied, what evidence supports them, and where are the gaps or exceptions? | Time/resource-scoped evidence manifests, policy/control mapping, provenance, sampling, exceptions, remediation and explicit nonclaims |
| Public / community / affected party | What happened or is proposed, how might it affect me, and how can I question, correct or challenge it? | Accessible notices, appropriate transparency, consultation, explanation, correction, appeal and outcome communication without internal-data exposure |

The same person may operate at several altitudes. “Executive” is therefore a
decision context, not a permission bundle, and seniority never implies automatic
access to sensitive content. Aegis evaluates purpose, resource, relationship,
classification, jurisdiction, time and action independently of organizational
rank.

Required altitude behavior:

- **Progressive summarization with accountable drill-down:** every aggregate
  claim identifies its population, time window, owner, freshness, computation,
  exclusions and supporting journey references.
- **Decision-oriented compression:** higher-altitude views emphasize material
  changes, options, assumptions, exposure, confidence, requested decisions and
  accountable owners—not activity volume or generated prose.
- **Bidirectional traceability:** an objective or commitment traces downward to
  admitted work and evidence; an operational observation or stakeholder signal
  traces upward to the affected commitment, risk or decision.
- **Scope-correct metrics:** individual signals are not repurposed for ranking;
  team flow is not summed into portfolio value; local utilization is not a proxy
  for outcomes; missing or incomparable populations remain explicit.
- **Selective disclosure:** external views are purpose-bound projections with
  field-, relationship-, artifact- and time-scoped access, redaction evidence,
  expiry, revocation and export controls. Internal deep links never bypass them.
- **Commitment semantics:** aspiration, forecast, target, approved plan,
  contractual commitment and delivered outcome remain distinct and visibly
  attributed.
- **Stakeholder interaction:** authorized outsiders can ask questions, supply
  evidence, review, accept, dispute, appeal or receive communications without
  being granted internal operator capabilities.
- **Communication integrity:** material status includes confidence, uncertainty,
  dissent, known gaps and last verified time. Summaries link to evidence and may
  be challenged or corrected; AI-generated executive synthesis is labeled and
  reviewed before consequential use.
- **Representation and consent:** affected groups can be consulted without one
  convenient stakeholder being treated as representative of everyone; consent,
  mandate and conflicts of interest are recorded when relevant.
- **Separation of duties:** creating a report, attesting its evidence, accepting
  a risk and approving an effect can require different people regardless of
  hierarchy.

### Goal and work-mode coverage

The following catalogue demonstrates divergent, convergent, delivery and
learning work; it is deliberately non-exhaustive. Pantheon must also admit
custom, emergent and domain-specific modes through the same extensible work-mode
contract and human-agent parity test above. Modes are not a mandatory waterfall:
participants may enter anywhere, branch, loop, suspend, compare alternatives,
compose modes, or stop after learning. A transition creates typed links and
explicit evidence; it does not silently convert one owner's record into another
owner's truth.

| Goal / work mode | User outcome and characteristic outputs | Necessary Pantheon composition | Evidence of success |
| --- | --- | --- | --- |
| Creativity and idea storming | Generate, combine, challenge, cluster, and preserve ideas without premature commitment; divergent notes, questions, alternatives, sketches | Agora rooms and facilitation; Vellum canvases/documents; optional Daimon ideation agents; Dendrite source context; Moira only when a structured method is requested | Authorship and provenance retained; alternatives remain distinct; clustering/ranking rationale visible; selected ideas link forward while rejected ideas remain recoverable |
| Strategy | Frame objectives, assumptions, constraints, stakeholders, scenarios, options, tradeoffs, measures, and decision horizons | Vellum strategy artifacts; Agora deliberation; Dendrite evidence; Moira multi-perspective evaluation; Chronos objectives/initiatives; Aegis for sensitive inputs | Decision record names evidence, assumptions, dissent, owner, horizon, measures, review trigger, and chosen/non-chosen options |
| Architecture | Model context, boundaries, interfaces, quality attributes, data/control flows, failure modes, alternatives, and decisions | Vellum diagrams/specifications/ADRs; Dendrite standards and repository grounding; Moira tradeoff review; Chronos architecture work; Agora review room | Versioned decision and views trace requirements to components, risks, alternatives, verification plans, and superseding decisions |
| Security | Identify assets, trust boundaries, threats, abuse cases, controls, residual risk, and response obligations | Aegis policy and authority; Vellum threat/control models; Dendrite control evidence; Moira risk workflow; Daimon constrained analysis; Chronos remediation; Hermes audit delivery | Threats and controls are traceable; authority is least-privilege; residual risk has an accountable acceptance; tests and monitoring validate controls |
| Prototyping | Test the riskiest assumptions quickly with disposable or promotable experiments | Chronos hypothesis/task; Vellum prototype artifacts; Daimon build/test session; isolated tool authority from Aegis; Agora feedback; Moira experiment method | Hypothesis, time/resource box, environment, observations, limitations, result, and discard/promote decision are explicit; prototype status cannot masquerade as production readiness |
| Engineering | Turn accepted intent into maintainable implementation while preserving design and operational constraints | Chronos decomposition/dependencies; Moira delivery workflow; Daimon execution; Vellum source editing; Dendrite code/standard grounding; Agora collaboration; selected VCS owner | Requirements-to-change trace, reviewable diffs, reproducible build evidence, decisions, ownership, and durable peer receipts |
| Testing and evaluation | Establish fitness through deterministic tests, exploratory inquiry, simulations, adversarial evaluation, and human judgment | Chronos test obligations; Moira test/evaluation matrix; Daimon runners/evaluators; Vellum plans/results; Dendrite fixtures/benchmarks; Hermes result events | Scope, environment, version, oracle, result, coverage and limitations are recorded; flaky, inconclusive, skipped, and failed remain distinct |
| Packaging | Produce identifiable, reproducible, verifiable artifacts with dependencies, metadata, provenance, and policy evidence | Selected build/package owner; Vellum manifests; Moira release workflow; Aegis signing authority; Dendrite/SLSA-style provenance; Hermes publication event | Immutable digest, version, dependency inventory, build provenance, signatures/attestations, compatibility and installation evidence |
| Delivery | Promote or communicate an approved outcome to a destination with rollout and recovery controls | Chronos release work; Moira staged/compensating workflow; Aegis approvals; selected deployment/publication owner; Hermes events; Agora coordination | Destination receipt, exact artifact/configuration, approvers, rollout state, verification, rollback/withdrawal disposition and user communication |
| Nominal operations | Sustain expected service, quality, cost, capacity, security, and user support | Operations surface; Chronos recurring work; Hermes events; Dendrite runbooks; Daimon bounded automation; Aegis operational authority; Moira maintenance workflows | SLO and budget posture, change history, capacity forecast, maintenance evidence, known degradation and accountable ownership are visible |
| Incidental operations | Detect, triage, contain, diagnose, recover, reconcile, and learn from unplanned events | Operations/Overview triage; Hermes alert/event evidence; Agora incident room; Chronos incident tasks; Moira response workflow; Daimon investigation; Aegis break-glass; Vellum timeline | Impact and uncertainty are explicit; containment and authority are attributable; recovery is verified; lost/duplicate effects are reconciled; follow-up work is admitted |
| Observability | Ask what is happening and why across behavior, dependency, work, agent, cost, and user-outcome signals | Owner-native observations correlated by journey links; Hermes event posture; trace context; Overview; per-owner metrics/logs/traces/profiles | Source, time, revision, sampling, gaps and cardinality are visible; signals link to journeys without being treated as semantic completion receipts |
| Event, log and metric inquiry | Detect, follow, search, aggregate, correlate and explain discrete occurrences, diagnostic records and measured populations in nominal or incident operation | Owner event feeds and Hermes domain streams; selected log/metric/trace backends; Pantheon telemetry explorer; Dendrite observability/SRE SMEs; Chronos/Moira incident workflow | Event/schema identity, log source/severity, metric type/unit/temporality/population, time quality, sampling, retention and gaps remain explicit; alert and telemetry evidence link to but do not settle incidents |
| Retrospection and introspection | Examine outcomes, decisions, process, agent behavior, collaboration, assumptions, and cognitive or organizational patterns | Vellum retrospective/decision artifacts; Agora facilitated review; Chronos flow history; Moira decision/effect history; Daimon attributed traces; Dendrite evidence | Blameless observations separate facts, interpretations and counterfactuals; insights name confidence; improvements have owners and follow-up checks |
| Data analysis and inquiry | Discover, profile, clean, join, aggregate, model, compare and visualize bounded data to answer an explicit question | Source-owner query adapters; Vellum analysis/notebook artifact; Dendrite data/statistics/privacy SMEs; Daimon analysis kernels; Moira evaluation method; Aegis data authority | Exact source snapshots, schemas, transformations, parameters, environment, assumptions, missingness, uncertainty, result and reproducibility evidence are retained |
| Graph inspection and querying | Explore dependencies, provenance, journeys, authority, topology, causation and domain relationships through patterns, paths and neighborhoods | Typed relationship projection plus owner-native graph adapters; query workbench; Vellum saved views; Dendrite graph/domain SMEs; Aegis scoped disclosure | Node/edge types and owners are explicit; direction, time, multiplicity, missing/redacted edges and query semantics are visible; a path is not presented as causation |
| Insight | Convert observations and evidence into reusable explanations, predictions, opportunities, or decisions | Dendrite evidence surfaces; Vellum analysis; Moira multi-hypothesis synthesis; Daimon analysis; Chronos experiments/actions; Agora review | Claim-to-evidence links, uncertainty, competing explanations, affected population, decision relevance, validation plan and expiry/review trigger are explicit |

Across every listed or newly admitted mode, humans and agents need lightweight
freeform work and progressively more structure. Pantheon should offer templates
and facilitation without requiring a workflow instance merely to think, discuss,
or sketch. Promotion from exploratory work into a commitment or consequential
effect must be a visible action with a new owner receipt.

### Cross-mode continuity requirements

- Preserve the original question, assumptions, constraints, decisions, dissent,
  evidence, artifacts, and owners as typed references across transitions.
- Support branching, comparison, merge-by-review, supersession, abandonment,
  reopening, and reuse without rewriting historical rationale.
- Allow humans to collaborate with zero, one, or several agents; expose who or
  what authored, transformed, selected, reviewed, and approved each result.
- Separate ideation confidence, strategic preference, architectural decision,
  test result, release approval, operational health, and learned insight. None is
  a synonym for another.
- Provide fit-for-mode templates: brainstorming prompts, strategy canvases,
  ADRs, threat models, experiment cards, engineering plans, test charters,
  release manifests, runbooks, incident timelines, retrospectives, and insight
  briefs. Templates are versioned aids, not hidden workflow authority.
- Make transition criteria inspectable: why an idea became work, a prototype
  became engineered scope, a build became a package, or an incident became a
  systemic improvement.
- Carry confidentiality, purpose, retention, and disclosure constraints forward
  through derived artifacts and exports without granting broader authority.
- Permit reflection on agent behavior and reasoning evidence without presenting
  private chain-of-thought as a product requirement; expose decisions, sources,
  tool actions, evaluations, uncertainty, and concise rationale instead.
- Route every material output or mode transition through the declared
  stakeholder-review policy. Low-risk exploratory work may review asynchronously
  or at a promotion boundary; irreversible, high-impact or externally affecting
  work requires review before the effect and again when material assumptions,
  scope, evidence, risk, topology or affected populations change.
- Always present final production rollout as a named-human go/no-go. Also pause
  any other material decision when the selected ensemble method admits a
  supported human-advocate finding that human gating is warranted. The work
  owner may answer the concern but cannot bypass the gate through relabeling,
  automatic disposition, timeout or an agent-synthesized human response.

### Reflexive self-development and live replacement

Pantheon is itself a valid target of a Pantheon journey. An agent working on
Pantheon code may move among any admitted goal, work mode and role—including
future/custom modes absent from the catalogue—but recursive capability does not
collapse the development, authority, deployment, observation and recovery
boundaries. “Hot swap” here means a qualified, reversible replacement
of an exact component version or route with bounded disruption. It may be an
OTP code upgrade, a drained process replacement, a rolling instance update, or
a feature/configuration route; it never means arbitrary unversioned mutation of
live memory.

The minimum safe architecture has five independently identifiable planes:

| Plane | Owns | Must not depend exclusively on |
| --- | --- | --- |
| Development | Isolated worktree, source changes, design artifacts, tests and agent session | The live production checkout or mutable state of the target instance |
| Candidate | Reproducible build, immutable digest, dependency inventory, provenance, signatures, compatibility and migration declaration | A tag, branch, mutable path or the candidate's claim about itself |
| Deployment | Admission, placement, traffic/route switching, drain, state-transfer orchestration and promotion receipt | Authority embedded only in the candidate or the agent that authored it |
| Observation | Baseline/candidate events, logs, metrics, traces, profiles, domain probes, user feedback and declared gaps | Instrumentation implemented only by the candidate, especially for safety gates |
| Safety and recovery | Policy, invariant monitors, known-good artifact/configuration, rollback/rehome, kill switch and trusted operator path | Code, credentials, storage or control endpoints replaced by the same change |

No candidate may be its own sole builder, verifier, approver, deployer,
observer, safety monitor or rollback mechanism. Independence is proportional to
risk: a documentation-only local change may use automated policy and tests; a
change to Aegis, deployment control, audit, telemetry, state migration or the
recovery path requires a separately trusted control plane and normally human or
multi-party approval. This is a software adaptation of runtime-assurance
architecture: an advanced component may operate only while an independent
monitor can keep the system inside a defined safe envelope and return control to
a known-good path.

#### Reflexive change lifecycle

1. **Admit intent.** Create a Chronos task and room with desired outcome,
   affected components, risk class, invariants, acceptance criteria, user-impact
   hypothesis and accountable owner.
2. **Activate roles.** The agent declares and records role/work-mode transitions
   as it brainstorms, analyzes, architects, threat-models, prototypes,
   implements, tests, packages, delivers, operates and reflects. A transition
   never widens authority implicitly.
3. **Isolate development.** Resolve the exact source revision into a bounded
   worktree/sandbox; pin Dendrite SME/corpus digests, tools and dependencies;
   keep secrets and production data out unless separately admitted.
4. **Build evidence.** Produce reviewable diffs, deterministic tests, security
   and compatibility results, an immutable artifact, SBOM/provenance, upgrade
   and downgrade instructions, state-schema declarations and a recovery plan.
5. **Review and authorize.** Independent checks bind the exact artifact digest,
   target instances/incarnations, policy epoch, exposure, traffic cohort,
   budgets, observation window, stop conditions and approvers. The authoring
   agent cannot attest away a failed or missing gate.
6. **Exercise safely.** Run in an isolated sandbox, then shadow/mirror production
   inputs without external effects where lawful. Compare semantic outputs,
   resource posture and invariants against a pinned baseline; redact and bound
   copied data.
7. **Canary and replace.** Admit a bounded cohort or one non-authoritative
   instance, establish readiness, then use the owner's selected mechanism:
   synchronized code change, drain/restart, rolling replacement or route switch.
   Preserve single-writer/consensus, lease, fencing and authority-home laws.
8. **Observe in flight.** The agent and humans attach/detach through the normal
   TUI/CLI/headless contracts to inspect domain receipts, events, logs, metrics,
   traces, profiles, user-visible probes and safety monitors. Candidate and
   baseline populations, gaps, sampling and versions remain explicit.
9. **Decide.** An independent policy/owner holds, expands, promotes or rolls
   back. Thresholds are guardrails, not proof of correctness; human pause,
   takeover and abort remain available. Outcome-unknown fences progression.
10. **Settle and learn.** Reconcile in-flight effects and data, record the exact
    deployed state, notify affected users, close or continue work, run a
    retrospective, publish validated insight and create follow-up tasks.

#### State, compatibility and rollback contract

- A change declares wire/API, persistent-state, event, policy, telemetry and
  relationship-schema compatibility plus the permitted mixed-version window.
- Stateful replacement uses quiesce/drain, checkpoint or owner-native transfer;
  in-flight operation identities, leases, cursors, acknowledgements and
  uncertain effects remain attributable and fenced.
- Prefer expand/migrate/contract and backward-readable formats. Destructive data
  migration is a separately authorized effect with backup/restore evidence;
  reverting code does not imply that data, messages or external effects rolled
  back.
- Federated rollout declares instance ordering, quorum/home/shard constraints,
  protocol negotiation, revocation freshness and behavior under partition.
  Mixed-version operation must be tested, bounded and visible.
- A failed compatibility, readiness, invariant, safety-monitor, telemetry-health
  or recovery-path check refuses promotion before traffic changes. A failed
  rollback remains an incident with explicit residue, not a successful receipt.

#### Per-component reflexive constraints

| Target | Qualified replacement concern |
| --- | --- |
| Pantheon shell/Overview/Operations/Configuration | Preserve journey references and attachments; an external supervisor owns replacement. Current `pantheon manage` supports guarded local process re-exec after validation, not arbitrary live mutation or remote control |
| Agora | Preserve room authority, actor attribution, ordering/revision semantics, memberships and cursors; a replacement cannot infer task state from conversation |
| Chronos | Preserve admission, attempt identity, claims, lease expiry and fencing; old and new schedulers cannot both accept as current authority |
| Moira | Version workflow definitions and state transitions; maintain saga/effect identity, compensation and uncertain-effect fencing across mixed versions |
| Daimon | Preserve agent/session identity, checkpoint/tool/provider contracts, budgets and event sequence; changing the runtime cannot silently restart an effect or widen tools |
| Hestia | Preserve host/runtime identity, placement incarnation, admitted model artifact, resource/isolation posture and lifecycle uncertainty; replacement cannot silently reuse a stale endpoint or duplicate a runtime |
| Dendrite/Tellus | Corpus content already promoted out of band may hot reload under Dendrite's current contract; authoring/promotion and server/runtime replacement remain separate guarded changes with pinned consultation digests |
| Vellum | Protect dirty edits, revision witnesses, persistence receipts, collaboration state and coordinate/schema migrations before editor or service replacement |
| Hermes | Preserve producer/event identity, retention, cursor, acknowledgement and schema compatibility; replay or relay changes cannot duplicate domain effects |
| Aegis | Highest sensitivity: candidate policy/runtime cannot authorize its own installation, mint its own trust, erase prior decisions or replace the independent recovery root |
| Oculus | Preserve source/frontier, query, time window, sampling, gaps, schema and disclosure; a dashboard or backend replacement cannot restyle missing or partial telemetry as healthy completeness |
| Keystone | Shared-mechanism changes require downstream conformance and mixed-version evidence because one defect can violate every sovereign peer's laws |

Current qualification is intentionally narrower than this target. Pantheon has
an embedding `restart_peer` primitive and a drain/replace `restart_topology`
path; its local management protocol validates a new profile and process-reexecs
while retaining stdio. Dendrite notices already-promoted corpus changes on disk.
These are useful substrates, not proof of agent-driven build, promotion,
canarying, state migration, fleet rollout or independent recovery.

**Implementability gate:** reflexive development remains architectural direction,
not an implementable product profile, until a versioned deployment profile names
and qualifies four concrete owners: (1) source/build/package/provenance, (2)
deployment/traffic/state-transfer, (3) observation/baseline/probes, and (4)
safety/recovery/supervision/known-good state. For each owner the profile must name
its sovereign service or external system, identity and trust domain, commands
and receipts, persistence/recovery, authority checks, failure independence,
headless attachment, uncertain-outcome behavior and operator. One component may
implement multiple roles only when the risk profile demonstrates sufficient
separation; the candidate and authoring agent can never own the independent
observation or recovery root. A diagram, placeholder adapter, local script or
unselected “deployment owner” cannot satisfy this gate.

The deployment class includes a named accountable promotion principal, a
sovereign promotion-decision record owner and a distinct executor. The recovery
class includes a named recovery-policy administrator, a sovereign versioned
recovery-policy/last-known-good record owner and a distinct applier. The profile
must also state whether its separation is trusted-development convention or an
OS-enforced isolated boundary; same-user processes cannot support hostile-write
resistance claims.

The options, precedents, tradeoffs and local-first recommendations for these
owners—plus the P0 Participation Registry and custom Work-Mode Registry—are in
the [reflexive ownership decision brief](reflexive-ownership-decision-brief.md).

### Analysis, graph inspection, and query capability model

Pantheon needs a query workbench that composes owner-native query capabilities;
it should not copy every peer's content into an implicit universal database.
The same experience can query tables, event/time-series data, text/corpora,
typed graphs and live owner projections, while retaining the distinct semantics,
authority and completeness of each source.

| Capability | Necessary behavior |
| --- | --- |
| Source and catalogue discovery | Find authorized datasets, schemas, graphs, streams, saved analyses and owner projections; show owner, purpose, classification, coverage, update/freshness, retention, quality and access path |
| Schema and semantic inspection | Browse fields/types/units/enums/nullability, keys, dimensions/measures, node/edge types, constraints, examples and compatibility; distinguish physical schema from business meaning |
| Profiling and quality | Compute bounded counts, distributions, missingness, uniqueness, ranges, cardinality, drift, constraint violations and sampling posture without implying that statistically plausible data is correct |
| Query authoring | Text editor, structured builder and natural-language-to-query proposal with syntax/schema assistance, parameterization, version pinning and dialect/profile selection |
| Plan and cost preview | Resolve sources, authorization, estimated scan/fan-out/rows/bytes/time/cost/privacy risk, remote authorities and freshness before execution; explain refusals and partial plans |
| Bounded execution | Read-only by default; explicit row/byte/time/fan-out/concurrency/cost limits, paging/streaming/backpressure, cancellation, timeout and stable query/run identity |
| Results | Typed tables, records, graphs, series, distributions or owner-native shapes with source snapshot/frontier, query digest, parameters, start/end, truncation, sampling, errors and unavailable/redacted partitions |
| Exploration | Sort, filter, group, pivot, compare, drill, brush, annotate and link back to exact source records without mutating source truth |
| Visualization | Tables, charts, timelines, topology/provenance/dependency graphs and uncertainty views chosen from data semantics; accessible alternatives and the underlying values remain available |
| Statistical and analytical methods | Descriptive, diagnostic, inferential, forecasting, anomaly, cohort, experiment and scenario analysis with named population, assumptions, method/version, uncertainty, multiple-comparison and validation evidence |
| Reproducible analysis | Save query text/AST, source identities, schema/profile versions, parameters, environment/tool versions, transformations, outputs/digests, narrative and rerun policy as a Vellum-owned artifact |
| Lineage and promotion | Record source → query/transformation → result → visualization → claim → decision/work links; promoting a finding creates explicit review/admission rather than silently changing operational state |
| Sharing and collaboration | Permission-aware saved views, comments, review, parameter templates, scheduled refresh and bounded export; recipients can see freshness and reproduce only within their own authority |

SQL adapters should state their supported
[ISO/IEC 9075](https://www.iso.org/standard/76583.html) profile rather than
claim generic SQL compatibility. Property-graph adapters may expose
[ISO GQL](https://www.iso.org/standard/76120.html) or
[SQL/PGQ](https://www.iso.org/standard/79473.html) profiles; RDF graphs may use
the stable [SPARQL 1.1 Recommendation](https://www.w3.org/TR/sparql11-query/)
while treating [SPARQL 1.2](https://www.w3.org/TR/sparql12-query/) as a
version-negotiated draft until standardized. Language-neutral analytical
interchange may use the [Apache Arrow specification](https://arrow.apache.org/docs/format/),
and notebook import/export should preserve the
[Jupyter notebook model](https://docs.jupyter.org/en/stable/) without making
an arbitrary notebook kernel part of Pantheon's trusted core.

#### Graph inspection requirements

Pantheon already has several graphs, none of which should be collapsed without
typed mapping: task dependencies, room reply/addressing structure, workflow
effects, agent/tool traces, source/provenance relations, document references,
event causation, authority/delegation, topology/routing, and cross-owner journey
links. Graph inspection must provide:

- exact graph owner, schema/profile, snapshot/frontier, direction, edge meaning,
  multiplicity, temporal validity, confidence and visibility scope;
- node/edge details, neighborhood expansion, path/pattern queries, shortest or
  weighted paths where meaningful, reachability, components, cycles, centrality,
  clustering and graph diff—with algorithm and assumptions disclosed;
- bounded depth, breadth, result, time and remote fan-out, plus cancellation and
  progressive/partial results that name missing authorities;
- visual and textual/tabular representations, stable selection/deep links,
  filtering, grouping, collapse/expand, time travel and before/after comparison;
- explicit distinctions between dependency, sequence, correlation, derivation,
  delegation, containment, similarity and causation. Graph adjacency or a path
  never proves causal influence;
- provenance for inferred, aggregated or agent-proposed edges and a review path
  before they enter any owner-authoritative graph;
- query-plan explanation and policy-aware pruning so a “no path” result says
  whether no matching path exists in the searched snapshot or visibility/gaps
  prevented a complete answer.

Natural-language query is an authoring aid. The user sees the selected sources,
compiled query/plan, limits and authority before execution. An agent may propose
joins, graph mappings, methods or visualizations, but cannot invent schema
semantics, bypass row/edge policy, convert absence into zero, or publish a
finding as a fact without review.

### Ambient agentic subject-matter expertise

Dendrite provides the repository's concrete model for expertise that is “on
hand” without living permanently in an agent's context. Pantheon should compose,
not reinterpret, its current query surface: exact loaded corpus identities;
dependency and orthogonality queries; `describe`/`describe_element`; byte-exact
reference reads; fold routing; ambient/activated/reference surfaces; and corpus
reconciliation. Dendrite's disclosed `search_narrative` stub must remain a gap
until it has real indexed answer content.

| SME depth | Purpose | Consumption rule |
| --- | --- | --- |
| Ambient | Small, stable doctrine or cautions worth keeping available throughout relevant work | Load the identified ambient kernel under a strict context budget; show role, corpus digest and stable recommendation IDs |
| Activated | Decision-local guidance when an exact context cue applies | Consumer supplies authored FSM-state or dialogue-derived context tags to Dendrite's canonical fold and retrieves only the routed roles/surfaces |
| Reference | Deeper foundations, standards, examples and exceptions needed on demand | Fetch when a human/agent asks or activated guidance requires it; retain source identity/digest and citation chain |

The SMEs are versioned bodies of doctrine and methods, not personas pretending
to be people and not independent sources of authority. An agent may embody or
consult an SME role, but every recommendation remains attributable to the exact
role, grounding bytes/surface digest, context tag, activation rule and consumer.

Required ambient-SME capabilities:

1. **Roster and discovery.** Browse available roles, concerns, foundations,
   standards, depths, versions and canonical fold tags; distinguish missing,
   unrouted, inactive and unavailable expertise.
2. **Two activation paths, one rule table.** Authored FSM states carry context
   tags; free-form dialogue/analysis agents supply current activity tags. Both
   query the same Rune-owned fold. Pantheon does not add fuzzy aliases or infer
   a different hidden routing vocabulary.
3. **Standing consultation behavior.** Agent/execution profiles state when they
   must evaluate context and invoke the fold—for example before consequential
   design decisions, data handling, external calls, deployment or incident
   closure. Dendrite answers the query; the consuming Daimon/Moira profile owns
   timely invocation and evidence that it happened.
4. **Layered context loading.** Start with ambient kernels, activate narrow
   decision material, and fetch references only when needed. Context budgets,
   cache identity and refresh/invalidations are visible; compaction recovery
   rereads only missing routed material by digest.
5. **Ensemble quality.** Query orthogonality and concern coverage before treating
   multiple SMEs as independent lenses. Same-facet agreement is correlated
   evidence, not multiple votes. Missing expertise and unresolved disagreement
   remain visible.
6. **Human control and policy.** Users can inspect why an SME appeared, consult
   another role, challenge advice, suppress optional recommendations or report a
   corpus issue. Mandatory safety/policy checks are labeled and enforced by the
   owning workflow/authority—not disguised as optional advice.
7. **Influence provenance.** Agent outputs and decisions cite which SME entries
   were consulted, adopted, rejected or materially changed the work. Stable
   recommendation IDs support utility feedback, while Dendrite remains neutral
   and does not infer causal impact from mere retrieval.
8. **Freshness and change.** Corpus hot reload creates new content identity.
   Active work declares whether it pins, refreshes or requires review after SME
   changes; a role name alone never implies the same guidance.
9. **Privacy and disclosure.** Context tags, prompts, observations and retrieved
   expertise follow purpose and disclosure policy. Cross-domain/federated SME
   queries expose only negotiated surfaces, never server-local paths or unrelated
   corpus content.
10. **Advisory boundary.** Ambient means continuously available and
    context-triggerable, not an uncontrolled background agent. SMEs do not admit
    tasks, grant tools, approve risks, mutate artifacts, or declare truth. Any
    autonomous monitoring or effect is a separately authorized workflow with
    budgets, receipts and an accountable owner.

Analysis should itself activate relevant SMEs: data semantics and quality,
statistics, causal inference, privacy, security, humanism, cost, graph theory,
observability and domain-specific expertise. The analysis record retains both
the query/method lineage and the exact SME surfaces that shaped interpretation.

### Productively adversarial stakeholder-SME review

Every work item carries an explicit stakeholder-review policy; every material
work product and consequential mode transition must be challengeable by an
ensemble selected for the affected context. “Adversarial” means independently
seeking disconfirming evidence, hidden assumptions, conflicts, failure modes,
harms, exclusions and better alternatives. “Productive” means steelmanning the
intent, citing evidence, stating uncertainty and offering tests, mitigations or a
clearer decision—not maximizing objections, performing debate, blocking by
default or attacking people.

The ensemble is dynamic rather than a fixed committee. Each review starts with
the work's purpose, mode, lifecycle state, affected populations, stakeholders,
assets, decisions, effect boundary, reversibility and risk. Dendrite uses those
context tags to route exact versioned SME surfaces, expose concern coverage and
check whether the proposed lenses are actually orthogonal. Moira owns the
bounded review plan: question, method, admitted cohort/persona lenses, evidence
and claim kinds, independent reports, dissent, candidates, fuel/non-convergence,
Pareto analysis where applicable and the retained resolution. Daimon may execute
agent reviewers; Agora and Vellum carry human discussion and annotations; none
of them turns review agreement into authority.

The default concern roster is a prompt for contextual selection, not a claim
that every item is required or sufficient:

| Stakeholder/SME lens | Productive challenge |
| --- | --- |
| Requester, beneficiary and end user | Does this solve the actual need, under realistic conditions, with usable acceptance and remedy paths? |
| Affected non-user, marginalized group and public-interest representative | Who bears cost, risk, exclusion, surveillance or reduced agency without receiving the benefit or choosing the system? |
| Product, strategy and mission owner | Is the outcome coherent with objectives, alternatives, opportunity cost, commitments and measures of value? |
| Domain practitioner and frontline operator | Are domain semantics, exceptional cases, tacit work, handoffs and operational realities represented accurately? |
| Architecture, integration and data | Are boundaries, contracts, state, provenance, migrations, coupling, scale and federated failure behavior coherent? |
| Security, privacy, abuse and safety | How could a malicious, compromised, mistaken or pressured actor misuse it; what data, rights and safe envelopes are at risk? |
| Reliability, operations, support and observability | Can it be detected, diagnosed, controlled, supported, recovered and retired under partial failure and incomplete telemetry? |
| Engineering, testing and maintainability | Is it implementable, reviewable, testable, compatible, debuggable and sustainable without hidden manual heroics? |
| Accessibility, human factors and organizational behavior | Can people with varied abilities and contexts understand, control and challenge it; what incentives or cognitive traps does it create? |
| Legal, policy, compliance and audit | Which obligations, jurisdictions, records, separation of duties, explanations and redress mechanisms apply? |
| Finance, capacity, environmental and delivery | What are the lifecycle cost, resource, energy, schedule, dependency, vendor and opportunity-risk envelopes? |
| External partner, supplier, regulator or assessor | Are interface, assurance, disclosure, portability, accountability and exit expectations credible across boundaries? |
| Skeptic, red team and incident investigator | What observation would falsify the leading claim, what is the strongest counterexample, and how would failure appear after the fact? |

Required ensemble behavior:

1. **Contextual framing and cohort challenge.** Moira frames the exact review question and
   instantiates the selected Rune-owned method; Dendrite routes both stakeholder
   and technical concerns from the current context. A generic “review this”
   prompt is insufficient for consequential work. One of the first retained
   questions is **“Do we have the right ensemble?”** The answer is `yes`, `no`
   or `contested`, names its evidence and assessor, and remains revisable when
   affected populations, risks or available representatives change. The
   assembled reviewers cannot establish their own representativeness merely by
   agreeing that they are sufficient.
2. **Equity and coverage before count.** The plan records required, present, missing,
   unavailable, conflicted and intentionally excluded lenses with rationale.
   Ten reviewers sharing one concern do not substitute for one missing affected-
   stakeholder or safety lens. Equity means meaningful standing, accessible
   participation, authorized material, opportunity to challenge framing, and
   proportionate attention to who bears benefit, cost and risk; it does not
   mean equal headcount or allow a convenient stakeholder to represent everyone.
   A `no` answer stops substantive disposition as `WrongEnsemble`; a
   `contested` answer preserves the dispute and seeks the missing human input or
   explicit governing response rather than averaging it away.
3. **Stakeholder standing before persona.** Every member identifies the
   constituency or technical domain they serve and whether they are an actual
   stakeholder, authorized representative, agent-modeled stakeholder
   perspective or technical SME. Actual and authorized representation retains
   its accountable principal, mandate and conflicts. A persona lens may shape
   how a member examines the question, but it is never the member's stakeholder
   identity, constituency, standing or authority.
4. **Grounded independence.** Each report identifies human/agent principal,
   stakeholder representation/mandate and persona lens, model/profile where applicable, exact
   grounding, evidence, assumptions and conflicts. Dendrite orthogonality is
   evidence about concern separation, not proof of cognitive independence;
   common models, prompts, data and organizational incentives remain visible.
5. **Actual stakeholders over simulation.** An agent SME can anticipate questions
   and expose omissions but cannot claim lived experience, consent, mandate or
   representation. Materially affected people and accountable domain owners can
   contribute, dispute the framing and request human review under disclosure and
   accessibility protections.
6. **Independent first pass.** Reviewers inspect the same pinned work/evidence
   before seeing peer conclusions when feasible, reducing anchoring and
   sycophancy. Later rounds may cross-examine claims, synthesize or request tests
   without erasing the first-pass record.
7. **Bidirectional challenge.** The ensemble must attack the proposed solution
   and the status quo, compare plausible alternatives, identify what is already
   strong and distinguish fatal flaws, bounded risks, preferences and open
   questions.
8. **Actionable findings.** A finding names affected stakeholder/invariant,
   evidence, severity/likelihood or epistemic status, confidence, reproduction
   or falsification method, proposed response and owner. Unsupported role-play,
   duplicate objections and rhetoric are marked rather than vote-counted.
9. **Dissent preservation.** Moira retains independent reports, rebuttals,
   unresolved conflicts, dominated/non-dominated candidates and bounded non-
   convergence. Consensus is not required; disagreement cannot be averaged into
   confidence or silently summarized away.
10. **Response and disposition.** The work owner responds to each material
   finding as accepted, mitigated, tested, deferred, risk-accepted, disputed or
   out of scope with evidence and accountable rationale. Reviewers may challenge
   the disposition; they do not acquire decision or effect authority.
11. **Risk-proportional gates.** Exploratory work may accumulate review for its
    next promotion boundary. High-impact, irreversible, security/authority,
    external-publication, production or affected-person decisions require the
    selected ensemble before action. Material change invalidates or narrows old
    review and reactivates the affected lenses.
12. **Human-advocate escalation.** A reviewer acting under the declared human-
    advocate lens may request human gating with evidence, affected people,
    consequence and rationale. When the review method admits that request as a
    supported, non-frivolous material finding, Moira pauses the decision and
    emits a stable gate request. A named human sees the exact decision,
    alternatives, evidence, dissent, uncertainty, reversibility and deadline,
    then records go, no-go, revise or defer. The finding cannot be dismissed as
    out of scope to evade presentation, but it does not grant the advocate veto
    or effect authority. Final production rollout requires this named-human
    presentation and explicit go/no-go unconditionally.
13. **Observe outcomes.** The ensemble's claims and predicted risks link to
    tests, canary probes, user feedback, incidents and outcome measures. A
    retrospective scores neither reviewer popularity nor objection volume; it
    asks which concerns were useful, missed, wrong or made obsolete by evidence.
14. **Headless parity.** Versioned TUI/CLI/machine contracts can create or attach
    to the same review, inspect framing/coverage/reports/dissent, submit an
    authorized report or disposition, detach and later resume by cursor. Client
    exit never resolves, approves, cancels or destroys the deliberation.

The review artifact contains the pinned work revision, question/method, context
and fold identities, intended/actual cohort, coverage/orthogonality result,
independence limitations, reports and evidence, dissent, owner dispositions,
unresolved risks, human-gate requests and presentation/response receipts,
resolution kind, authority decision references, expiry/review trigger and
outcome-follow-up. It remains review evidence—not acceptance,
approval, authorization, proof or a peer-effect receipt. This composes Moira's
current methodical-deliberation boundary and Dendrite's current fold/surface/
lattice boundary rather than inventing a new universal reviewer authority.

### Headless, CLI, and attach/detach operating model

Pantheon must not require the terminal presentation process to own the lifetime
of the work it displays. A TUI, one-shot CLI, automation client, IDE or remote
workbench should use the same owner-native clients and resource identities. UI
state may be local, but task, room, workflow, agent, document, query and event
state stays with its sovereign owner.

#### Lifetime and identity taxonomy

| Layer | Identity and lifetime | Detach/close consequence |
| --- | --- | --- |
| Component service process | Instance/incarnation serving one or more logical authorities | Client detach does nothing to it; shutdown/drain is an explicit management operation |
| Domain resource | Task, room, saga, document, stream, grant namespace or grounding corpus | Persists according to its owner; UI exit cannot complete, archive, delete or transfer it |
| Durable execution/analysis session | Daimon agent session, governed Moira participation, notebook/query run or similar owner object | May continue, wait, checkpoint or require explicit reattachment under its owner contract; close/cancel/interrupt are distinct |
| Provider/kernel binding | Potentially ephemeral process behind an execution or analysis session | Loss may yield waiting, detached, reattach-required or outcome-unknown; it never silently creates a replacement session |
| Transport connection | One authenticated protocol connection and negotiated session | Disconnect invalidates connection-local handles/subscriptions and may create uncertain in-flight delivery; durable resource identity remains |
| Client attachment | One authorized viewer/controller attached to selected resources and cursors | Detach stops this client's observation/control and releases only attachment-owned leases; domain work continues by default |
| Presentation session | TUI layout, focus, filters, local drafts, scroll/cursors and selected workset | May be saved locally or discarded; it grants no authority and does not become owner state unless explicitly published |
| Operation attempt | One exact mutation/query/control identity and its dispatch/result state | Survives presentation loss where recovery is promised; changed-input resubmission is not reattachment or replay |

“Attach” is therefore overloaded unless qualified. Pantheon commands and UI must
name whether they attach to a component instance, logical authority, domain
resource, execution/analysis session, event cursor or Pantheon management
process. A durable public reference identifies the target; transport session
handles, numeric grant handles, file descriptors and bearer credentials do not.

#### Attach contract

1. Discover or receive an exact target reference through a bounded authorized
   catalogue, explicit configuration or prior journey link. Never scan ambient
   machine/network state and attach by display-name coincidence.
2. Resolve the current authority/home/endpoint with provenance and expiry;
   authenticate the caller and instance; negotiate protocol, capability, limits,
   disclosure and consistency profile.
3. Authorize the attachment mode independently: inspect/read-only, interact,
   control, administer or presentation handoff. Read access never implies control.
4. Fetch a revisioned snapshot plus cursor/frontier, then subscribe from that
   boundary. Buffering, backpressure, gaps, resnapshot and duplicate handling are
   explicit. Attach does not replay natural-language history into a provider.
5. Revalidate current resource incarnation, policy and session state before
   controls. Stale attachments become read-only/refused or refresh; they do not
   retarget a new resource sharing the same name.
6. Record content-safe attachment evidence—principal, client, target, mode,
   protocol/profile, start/end and reason—without treating attention as approval.

Multiple clients may observe the same resource. Concurrent control requires an
owner-defined collaboration, controller-lease, turn-taking or optimistic-
revision contract. “Take over” must show and fence the prior controller where
exclusive control applies; a terminal-size or last-writer convention is not a
control policy.

#### Detach, disconnect, reattach, close, and shutdown

- **Detach** is an explicit clean client action. It persists the last safe
  presentation cursor if selected, ends subscriptions, releases attachment-only
  resources, and returns a receipt. It does not mutate the displayed domain
  resources or stop independently owned processes.
- **Disconnect** is unplanned transport loss. In-flight mutation may be
  `outcome_unknown`; connection-local authority and handles expire according to
  the owner. The client retains recovery identity and must not convert the event
  into clean detach or cancellation.
- **Reattach** creates a fresh authenticated transport/client attachment to the
  same stable resource, resnapshots/reconciles from a retained cursor, and
  reacquires current authority. It never revives a stale transport session or
  bearer handle.
- **Provider/kernel reattachment** is a separate owner mutation used only when a
  durable execution/analysis session reports a compatible detached state. It
  uses the exact retained profile/checkpoint and current authorization; it is
  not ordinary UI attachment and cannot fall back to “create new.”
- **Close/leave/archive** changes a domain resource and requires its owner guard.
  A Daimon close may require cancellation first; leaving an Agora room changes
  membership; closing an analysis session may terminate its kernel. None is a
  synonym for detach.
- **Cancel/interrupt** requests work attenuation and can race with completion.
  Client receipt and later terminal owner evidence remain distinct.
- **Shutdown/drain** changes component-process topology. A client attached to an
  external process cannot acquire lifecycle authority merely by exiting. A
  Pantheon process that explicitly hosts peers may offer “detach shell and leave
  supervised owners running” only through a durable supervisor handoff; otherwise
  it must accurately require drain on host exit.

The classic [tmux client/session separation](https://man.openbsd.org/tmux.1)
is useful presentation precedent: clients attach to persistent sessions and may
detach without destroying them. Jupyter's
[session and kernel APIs](https://jupyter-server.readthedocs.io/en/stable/developers/rest-api.html)
similarly distinguish listing/attaching to running computation from explicit
kernel interrupt or deletion. Pantheon must add stronger owner-specific
authority, receipts, uncertainty and federation semantics to both patterns.

#### Headless parity requirements

Headless parity means semantic reachability, not a screen-shaped JSON dump. Each
qualified TUI journey must have a versioned, bounded machine contract for the
same owner observations, challenges, commands, receipts and recovery states.

- Provide `list/discover`, `inspect/snapshot`, `watch/follow`, `attach`, `detach`,
  `execute/control`, `recover/lookup`, and where supported `close/cancel` as
  separate operations. Exact command names may differ, but their semantics must
  not be collapsed into one generic action.
- Use versioned JSON for finite results and NDJSON or another negotiated framed
  stream for events/progress. stdout is machine data; human diagnostics go to
  stderr; schemas, exit statuses and refusal classes are stable and documented.
- Support pagination, bounded batches, cursors/frontiers, resume-from-cursor,
  heartbeats, backpressure, gap/resnapshot instructions and graceful stream
  detach. EOF is not automatically semantic acknowledgement.
- Replace modal confirmation with a two-phase challenge/response or exact
  confirmation document/fingerprint that binds target, current revision, action,
  bounds, authority context, consequence and expiry. Noninteractive mode never
  weakens confirmation or defaults an ambiguous choice.
- Accept prompts, query text, documents and secrets through bounded files, stdin,
  protected descriptors or credential providers—not argv, URLs, logs or shell
  history. Machine output remains content-safe at the selected disclosure level.
- Long-running agent, workflow, analysis and query operations return stable run/
  session references early. Callers can detach, poll/watch later, reconnect after
  process loss and recover exact receipts without holding one terminal open.
- A headless client can select an existing bounded workset, perform the same
  sparse/batch controls as the TUI, receive per-child dispositions, stop on
  uncertainty and explicitly resume only unsent children.
- TUI and CLI can hand off the same resource/session using a content-free stable
  reference and optional presentation cursor. Authority is reacquired by the new
  client; it is never serialized into the handoff token.
- All analysis, graph-query and ambient-SME operations expose the same catalogue,
  plan, bounds, source/corpus identities, partial results, lineage and influence
  evidence headlessly. Visualizations have data/specification alternatives.
- Attach/detach and headless contracts operate against local, remote and
  federated homes. Connection loss, route change and trust expiry preserve the
  same uncertainty and reauthentication laws.

#### Per-component attachment behavior

| Component / capability | Attach target | Clean detach must preserve | Separate explicit actions |
| --- | --- | --- | --- |
| Agora | Room/thread plus participant identity and event frontier | Membership, room history and unread position according to owner policy | Speak, edit, react, mark read, leave, moderate, archive |
| Chronos | Workstream/task/attempt/feed and revision/cursor | Task state, scheduling and existing leases not owned by this attachment | Claim/release, pause, cancel, reschedule, complete, retire |
| Moira | Saga/participation/effect frontier and evidence cursor | Workflow progress, uncertainty fences, timers and peer-effect identities | Advance, reconcile, compensate, cancel, terminal disposition |
| Daimon | Exact agent session/turn/event cursor | Session checkpoint, active/waiting work, budget and retained events | Prompt, permission decision, advance, cancel, provider reattach, close |
| Hestia | Hosting authority/runtime incarnation, artifact-catalog revision and lifecycle revision | Verified local artifact cache, runtime placement, admitted artifact/resource posture and owner-retained lifecycle | Qualify provider behavior, invoke inference, schedule work, or kill an unknown process; acquisition and runtime lifecycle remain separately authorized operations |
| Tellus / Dendrite | Load/corpus/fold/SME/query reference | Corpus identity and queryable service; finite query needs no durable attachment | Reload/publish corpus in owner tools; long search/index jobs only if separately modeled |
| Vellum | Workspace/document/revision and optional collaboration cursor | Source state, saved state and other collaborators; local unpublished draft policy is explicit | Edit, save, discard draft, release lock/lease, close workspace, rebind destination |
| Hermes | Stream/consumer and disclosed event frontier | Stream and consumer cursor; observation must not acknowledge delivery | Acknowledge, reset/retire consumer, compact, redrive |
| Aegis | Authority namespace/epoch or managed admin session | Durable authority state; ordinary observation can reconnect | Issue, attenuate, authorize, revoke; managed session disconnect deliberately abandons its session-local grant names/handles rather than reattaching them |
| Oculus | Source catalogue/query/run cursor and explicit time window | Saved query identity, backend data and independent telemetry pipelines | Start/stop producers, acknowledge domain events, settle incidents, mutate peer state |
| Analysis/query | Saved analysis plus exact run/result cursor | Persisted run/checkpoint/result where the selected owner promises it | Execute, cancel, interrupt kernel, schedule refresh, publish result, close session/kernel |
| Pantheon Operations | External instance or explicitly owned supervised topology | External peer processes; hosted peers only after supervisor handoff | Drain, stop, restart, promote/fail over, rehome, abandon residue |

#### Current qualification versus required capability

Pantheon currently qualifies bounded headless observation for configured peers,
several separate guarded CLI controls, configured interactive attachment, a
local stdio management session, and Daimon inspection/guarded provider
reattachment. Headless Chronos is observation-only; some Vellum, Agora and other
controls remain interactive-only or separate one-shot commands. Configuration
selects attachments at process launch, and normal TUI exit drains peers Pantheon
explicitly hosts.

One workspace profile now crosses interactive and aggregate-observation
adapters without a reduced profile copy. The headless observation validates
requirements against the complete configured route, exercises only the
capabilities it actually implements, and returns valid but unexercised
requirements with an explicit `not_exercised` disposition. This is an adapter
boundary repair, not control parity: a disposition is neither a control receipt
nor permission to ignore an unknown capability, and missing bounded session-set
machine controls remain missing behavior.

Required future work is dynamic authorized attach/detach, resumable watch
cursors, session/resource catalogues, full headless control/query/analysis/SME
parity, presentation handoff, and supervisor-backed “leave hosted work running.”
These are Required/Later capabilities, not claims about the current qualified
profile.

### Events, logs, metrics, and operational signals

Oculus is the sovereign observability-query peer for this capability family. It
owns source catalogues, bounded query/follow runs, saved analysis definitions,
typed pivots, partial-source/gap evidence and backend adapters. It does not own
producer instrumentation, peer-domain meaning, Hermes streams, alert authority,
incident/workflow state or the underlying telemetry stores. Pantheon and a
future `oculus-tui` reuse its public observation/query client; headless callers
use the same contract without constructing presentation state.

“Event” is overloaded. Pantheon must label the kind and owner before presenting
or correlating it. A shared timestamp or trace ID does not erase the different
meaning, delivery, durability or access policy of each signal.

| Signal | Answers | Required semantics | Must not be treated as |
| --- | --- | --- | --- |
| Domain event | What owner-recognized domain transition occurred? | Closed owner schema, actor/causation, resource revision/sequence, occurrence time, replay/retention and authenticity | A generic log line, global order, downstream-effect receipt or authorization |
| Integration event | What bounded fact was offered across an owner boundary? | Producer/source, type/schema/version, subject, stable event identity, correlation, delivery and consumer acknowledgement | Proof the consumer acted or that the originating domain state is still current |
| Audit/security record | Who attempted/decided/did what under which policy and evidence? | Protected integrity, identity, time, scope, decision/outcome, disclosure, retention and review policy | Ordinary debug telemetry or a reusable grant |
| Telemetry event | What noteworthy point occurrence did instrumentation observe? | Instrumentation scope/version, resource, event/observed time, attributes, sampling and pipeline status | Owner-domain transition or durable effect proof |
| Log record | What diagnostic or narrative record did a source emit? | Structured body where possible, resource, scope, severity, occurrence/observed timestamps, trace/span correlation, redaction and retention | Complete history, stable API, metric sample or automatically true explanation |
| Metric point/stream | What measured value or aggregated population existed over a declared instant/interval? | Name, type, unit, attributes, temporality, start/end, population, aggregation, resets/gaps, exemplar and cardinality limits | Individual occurrence, log, causation, exact count after sampling, or success proof |
| Trace/span | How did one logical operation traverse timed boundaries? | Trace/span identity, parent/link semantics, resource/instance, interval, status, sampling and clock quality | Domain identity, authorization, exact causal proof, complete population or recovery record |
| Profile | Where did sampled resource consumption occur? | Profile type, period, sample population/rate, symbols/build identity, aggregation and privacy | Exact per-request timing or full execution history |
| Alert | Which rule evaluated a signal into an attention state? | Rule/version, evaluation window, data freshness, pending/firing/resolved state, labels, deduplication and inhibition | Incident, diagnosis, user impact, page acknowledgement or recovered service |
| SLI/SLO/error budget | How is service behavior compared with an explicit objective over a population/window? | Population, good/valid events, objective, window, exclusions, data quality and burn calculation | Contract unless declared, individual performance, task completion or reason to hide failures |

OpenTelemetry currently treats traces, metrics and logs as supported signals;
events are a named form of log record while its dedicated Events signal remains
under development. Pantheon should use the stable
[OpenTelemetry log data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/)
and [metrics data model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/)
where selected, preserving their occurrence-versus-observed timestamps, resource,
instrumentation scope, metric temporality and gaps. CloudEvents remains useful
for bounded integration-event context. The [OpenMetrics 1.0 specification](https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md)
is established wire-format prior art but its CNCF project is archived/merged
into Prometheus; new profiles should state the exact Prometheus/OpenMetrics or
OTLP version rather than claim a generic current standard.

#### Event capabilities

- Catalogue event owners, streams/feeds, schemas, versions, subjects, producers,
  consumers, retention, ordering scope, privacy and delivery guarantees.
- Inspect and follow bounded owner-native domain events and Hermes streams by
  stable cursor; filter on indexed content-safe fields; disclose payload only
  through the owner's explicit authority and limits.
- Search by time, event/source/type/subject/resource, actor, journey, causation,
  correlation and trace reference while distinguishing event time, ingest time
  and observation time.
- Replay or redrive only through owner-specific consumer/effect contracts.
  Replay is not mutation retry; acknowledgement is not downstream completion.
- Define event-triggered work through an explicit schema-to-Moira/Chronos intake
  policy with deduplication, authority, capacity, expiry and poison-event handling.
- Detect gaps, duplicate delivery, out-of-order observation, schema skew,
  retention loss, stalled consumers and loops in federated relays.

#### Log capabilities

- Discover authorized log sources and structured schemas; search full text and
  fields; filter by resource/service/instance, severity, scope, time, journey,
  trace/span and deployment; save/share parameterized queries.
- Live-tail/follow with backpressure, bounded buffers, sampling/drop notices,
  pause/resume cursor and clean detach. A tail is not guaranteed lossless unless
  the source profile says so.
- Render structured fields and original body safely, with multiline support,
  encoding/parse status, redaction/tokenization evidence and access-time audit.
- Group/deduplicate repeated patterns, compare before/after deployments, derive
  frequency trends and pivot from log to exact trace/metric/event exemplars.
- Preserve source timestamp and observed/ingested timestamp separately; expose
  clock offset/uncertainty and late arrival rather than imposing false total order.
- Control volume through severity/scope policy, rate limits, sampling and
  retention without allowing diagnostics failure to change domain outcomes.
- Prevent prompts, document contents, credentials, bearer grants, raw PII or
  secrets from entering default logs; support emergency sensitive access only
  through explicit purpose, disclosure and audit policy.

#### Metric capabilities

- Catalogue metric name, description, owner, type, unit, attribute schema,
  temporality, collection interval, population, aggregation, retention and
  compatibility version.
- Correctly distinguish monotone counters, gauges, histograms and exponential/
  native histograms; show process resets, counter discontinuities, missing and
  stale series, not-a-number values and delta/cumulative conversion.
- Query, aggregate, compare and visualize rates, ratios, distributions,
  percentiles, saturation, errors, throughput, work age, budgets and user
  outcomes over explicit windows and populations.
- Enforce label/cardinality budgets at instrumentation and query time. Resource,
  session, user, task, room, document, prompt or unrestricted error text must not
  become unbounded metric labels.
- Prefer aggregatable histograms for cross-instance latency distributions; show
  bucket/schema differences and never average precomputed percentiles.
- Link sampled exemplars to traces/logs/events where authorized without copying
  sensitive high-cardinality identity into every series.
- Build recording/derived metrics, dashboards, anomaly rules, SLI/SLOs and alerts
  as versioned reviewed artifacts with dependency, owner and cost information.
- Expose telemetry-pipeline self-observation: scrape/export success, queue depth,
  dropped/rejected points, backpressure, sampling, transform errors, clock
  quality, storage lag and query freshness.

#### Unified operational inquiry

The workbench should support a pivot loop rather than one giant merged record:

1. start from a user report, alert, domain event, log pattern, metric change,
   trace, topology/resource, deployment or journey;
2. preserve time window, filters, source/frontier and authorization while moving
   to related signals through typed links or exemplars;
3. compare nominal baseline, recent change, affected/unaffected populations and
   missing telemetry before proposing a hypothesis;
4. consult Dendrite observability, SRE, distributed-systems, security, privacy,
   data and domain SMEs as context requires;
5. capture queries, panels, selected records, graphs, hypotheses, counterevidence
   and uncertainty in an incident/analysis artifact;
6. use owner-native receipts and user-visible verification—not telemetry alone—
   to declare mitigation, recovery or business outcome;
7. convert validated learning into instrumentation, alert, architecture,
   runbook, test or product work and later evaluate effectiveness.

TUI and headless clients need parity: bounded catalog/query/snapshot, live
follow/watch, dashboard/SLO inspection, alert acknowledgement where an alert
owner supports it, stable cursors, clean detach and later reattach. Headless
output includes query/window/population, partial sources, sampling, drops,
retention and schema; terminal detachment never acknowledges an event, alert or
incident and never stops telemetry producers or collectors.

#### Ownership and non-goals

- Hermes carries bounded domain/integration events; it does not automatically
  become the log, metric, trace, alert, audit or observability backend.
- Pantheon's existing bounded local instrumentation is content-safe performance
  and operation observation. It is not owner history, cross-process completeness,
  product SLO certification or a reason to weaken independent peer telemetry.
- Telemetry export is passive. Sink/collector failure, saturation or sampling
  may degrade observability and must be visible, but cannot change the domain
  command result or silently block a safety receipt.
- Logs and traces can contain sensitive derived information even when message
  bodies are absent. Access, retention, residency, deletion and external export
  require their own classification and policy.
- “No logs,” zero metric rate, absent trace, resolved alert or green dashboard
  never proves that no event/effect/impact occurred.

### Federated and distributed topology model

Pantheon must support mixed topologies: embedded components, same-host sovereign
processes, remote services, replicated logical authorities, sharded authorities,
worker pools, and federation across administrative trust domains. Not every
component needs to exist at every site, and different component kinds may select
different distribution laws in the same journey.

“Mesh” means that independently owned instances can discover permitted peers,
negotiate a compatible profile, authenticate, exchange bounded owner-native
commands/events/replicas, and survive partial connectivity. It does not require
an all-to-all network, one global control plane, one replicated database, or
active-active mutation for every domain.

#### Topology vocabulary

| Term | Required meaning |
| --- | --- |
| Deployment | An operated collection of sites, trust relationships, component instances, routing policy and governance |
| Trust domain | One administrative identity root and policy boundary; it need not equal a DNS domain or organization |
| Site / failure domain | A host, cluster, zone, region, device or organization boundary whose correlated failure matters |
| Component kind | Agora, Chronos, Moira, Daimon, Hestia, Tellus, Vellum, Hermes, Aegis, Oculus or another sovereign protocol family |
| Logical authority | The named owner of a resource namespace and its mutation order, independent of the process currently serving it |
| Instance | One concurrently running service member with a unique, observable identity |
| Incarnation | One lifetime of an instance or authority home; restart/replacement changes it and fences stale sessions |
| Endpoint | A transport location for reaching an instance; it is discovery data, not identity or authority |
| Home | The logical authority selected to order mutations for a resource or shard |
| Replica | A bounded projection governed by a declared consistency, disclosure, repair and promotion contract |
| Worker | An execution-capable instance that owns no durable work merely by being selected |
| Federation edge | An explicit, directional relationship between authorities or trust domains with protocol, disclosure, limits and lifecycle |
| Route | A resolution from stable logical authority/resource identity to eligible endpoint(s), with revision, source, expiry and health evidence |

Resource references must therefore be globally unambiguous within the selected
federation namespace—for example `(component kind, authority namespace,
resource identity, incarnation/revision where required)`. Hostnames, process
IDs, display names and current endpoints must not become durable resource IDs.

#### Permitted multi-instance patterns

Each logical authority selects and advertises exactly one pattern per resource
class. Clients refuse ambiguous or unsupported combinations.

| Pattern | Suitable use | Required law |
| --- | --- | --- |
| Single home plus remote clients | First remote profile for mutable task, room, saga or document state | All mutations return to the named home; reconnect cannot change home silently |
| Single home plus read replicas | Geographic read latency, bounded offline observation and disclosure | Replicas consume only home-authoritative events/snapshots, detect gaps/divergence, never allocate authoritative revisions and never re-export imported authority |
| Replicated state machine | High availability for one logical authority | A selected consensus protocol establishes one committed order, quorum rules, membership change, durable term/epoch and stale-leader fencing |
| Sharded authorities | Scale independent rooms, workspaces, streams, workflows or work graphs | Stable ownership function/directory, per-shard ordering, explicit reshard/rehome protocol, no invented global order |
| Active-active convergent data | Only domain objects with a formally selected merge algebra | Operation/state preconditions prove convergence; causal metadata, tombstones, bounds, authorization and semantic conflict presentation remain explicit |
| Stateless or checkpointed worker pool | Daimon execution, indexing, evaluation or other replaceable compute | Scheduler assigns an exact attempt; worker identity, profile and environment are attributed; lease/attempt fencing prevents duplicate authority |
| Gateway federation | Cross-organization policy, protocol or disclosure boundary | Gateway cannot amplify authority or reinterpret domain success; end-to-end owner identities, request IDs, receipts and uncertainty survive transit |

The [Raft replicated-state-machine model](https://raft.github.io/) is relevant
when one authority needs highly available ordered mutation. The
[SWIM membership model](https://research.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
shows why suspicion and failure declaration are separate, but membership cannot
choose a semantic writer. [CRDT theory](https://perso.lip6.fr/Marc.Shapiro/papers/2011/CRDTs_SSS-2011.pdf)
permits coordination-free convergent updates only for data types satisfying its
merge/operation conditions; “eventual consistency” is not permission to merge
arbitrary tasks, grants, workflows or effects.

#### Per-component federation semantics

| Component | Useful multi-instance roles | Default authority and consistency rule | Federation-specific capability |
| --- | --- | --- | --- |
| Pantheon shell / Overview | Many user workbenches and site-local projections | No shell becomes global semantic owner; each observation names source authority, route, site, freshness and completeness | Federated directory, topology/partition map, cross-site journey projection, scope-preserving deep links and degraded/offline views |
| Agora | Room homes, remote participants, bounded room replicas, optional home clusters | One authoritative home/order per room in the first profile; replica acknowledgement is not participant read or publication success | Server federation, invites/membership, disclosure negotiation, gap repair, home discovery and later explicit home transfer; Matrix's [server-server specification](https://spec.matrix.org/latest/server-server-api/) is useful prior art, not an adopted wire contract |
| Chronos | Workstream homes, graph shards, scheduler/claim replicas, remote intake | One authority orders each task graph and lease epoch; global fairness or ordering is not inferred across workstreams | Attributable remote admission, cross-authority dependencies, resource brokerage, lease fencing, rehome of quiesced work and federated portfolio projection |
| Moira | Saga homes, workflow shards, standby/consensus replicas, remote peer routers | One saga authority orders its decision/effect frontier; peer effects remain remote owner receipts | Durable remote-effect outbox/inbox, route pinning, reconciliation across partitions, coordinator recovery and controlled saga-home transfer |
| Daimon | Provider gateways, regional worker pools, session homes, checkpoint replicas | One exact execution attempt has one admitted owner even if compute is replaceable; duplicate providers do not imply duplicate permission | Capability/attestation-aware placement, session reattachment, checkpoint portability, data/egress locality, result deduplication and worker-loss fencing |
| Hestia | Host authorities, runtime placements, accelerator pools and lifecycle replicas | One placement authority orders each runtime incarnation; endpoint liveness is not readiness, qualification or execution authority | Resource/posture-aware placement, remote host protocol, drain/rehome, orphan reconciliation, artifact locality and duplicate-runtime fencing |
| Tellus | Source-local indexes, read replicas, federated query brokers | Source identity, version, provenance and disclosure remain authoritative at their named owner; broker ranking is derived | Bounded fan-out, per-source freshness/completeness, query budget, result provenance, cache invalidation and no-result versus unreachable distinction |
| Vellum | Workspace homes, read replicas, remote editors, selected collaborative replicas | First profile retains one authoritative workspace home; replicas cannot allocate revisions or persist edits | Disclosure-bound replication, source-revision gap repair, remote edit admission, explicit collaboration/merge profile and controlled workspace transfer |
| Hermes | Stream homes, partitions, consumer groups, selected replicated logs | Ordering is per declared stream/partition unless a stronger profile proves otherwise; acknowledgement remains consumer progress | Producer routing, replicated retention, consumer rebalance, cross-site relay with loop prevention, schema/disclosure negotiation and gap/lag posture |
| Aegis | Trust-domain authorities, policy decision replicas, attenuating delegation bridges | Authentication, trust and authorization stay distinct; a foreign identity or grant is not locally authoritative without an explicit relationship | Trust-bundle lifecycle, policy/claim mapping, epoch/revocation propagation, constrained delegation, cross-domain audit and fail-closed stale-policy behavior |
| Oculus | Query brokers, source catalogues, saved analyses, dashboards and selected telemetry gateways | Every result preserves source/frontier, time, sampling, gaps, disclosure and backend-specific limits; correlation is not domain causation | Federated query planning, partial-result settlement, cursor handoff, locality/cost policy, schema negotiation and source-health projection |
| Operations / Configuration | Site operators, fleet views, deployment controllers | Each controller owns only declared instances/sites; remote reachability does not imply lifecycle authority | Inventory, placement, rollout, drain/failover, certificate/route rotation, dependency-aware maintenance, disaster recovery and topology change receipts |

Agora and Vellum already provide strong repository precedents: their federation
workspaces select one authoritative home, bounded direct-home replicas, explicit
disclosure, trust-epoch/session binding, gap repair, and ordinary owner-authorized
remote mutations. Chronos provides attributable federation admission contracts.
These are peer-qualified precedents, not current Pantheon remote qualification;
Pantheon's present profile remains local attach/host.

#### Federation control-plane capabilities

1. **Identity and trust.** Workload, user, component-kind, logical-authority,
   instance and incarnation identities are distinct. Mutual TLS protects a
   channel but does not grant domain authority. A selected profile should support
   short-lived workload identities and explicit cross-domain trust such as
   [SPIFFE federation](https://spiffe.io/docs/latest/spiffe-specs/spiffe_federation/),
   whose relationships are directional and bind each trust domain to its exact
   bundle endpoint/profile. Protocol sessions bind negotiated hellos to the
   authenticated channel; [TLS 1.3 exporters](https://datatracker.ietf.org/doc/html/rfc8446#section-7.5)
   and [RFC 9266 channel binding](https://datatracker.ietf.org/doc/html/rfc9266)
   provide relevant mechanisms.
2. **Discovery and directory.** Static configuration, DNS SRV, orchestrator
   discovery or a signed authority directory may locate endpoints. Discovery
   records carry component kind, protocol/profile range, authority namespaces,
   instance/incarnation, site, limits, expiry and provenance. DNS SRV
   [RFC 2782](https://datatracker.ietf.org/doc/html/rfc2782) locates services but
   does not authenticate them or decide which logical authority owns a resource.
3. **Negotiation.** Peers exchange closed protocol/schema versions, optional
   capabilities, limits, disclosure classes, compression, ordering, replay,
   retention and consistency profiles. No common safe profile means refusal,
   never best-effort semantic downgrade.
4. **Routing.** Resolve the logical home before dispatch, pin the exact route and
   authority epoch to an operation, detect redirect loops, and re-resolve only
   under the owner contract. Load balancing among equivalent replicas/workers
   cannot move a mutation among independent authorities.
5. **Durable remote effects.** Outbox/inbox or equivalent owner contracts bind
   canonical intent, source/destination authorities, request identity, route
   epoch, dispatch state and exact receipt. Store-and-forward preserves expiry,
   revocation checks, ordering and uncertainty; it does not turn at-least-once
   transport into exactly-once domain effects.
6. **Partition behavior.** Every command and observation declares whether it
   refuses, serves stale/read-only state, queues bounded work, proceeds under a
   lease/quorum, or accepts convergent updates. The UI identifies partitioned,
   suspect, stale, read-only and unavailable separately. No automatic failover
   may create two writers.
7. **Membership and placement.** Join, leave, replace, drain, promote, demote,
   rebalance and rehome are guarded workflows with quorum/home evidence and
   rollback limits. Liveness suspicion is observation, not proof of death or
   authority transfer.
8. **Data governance.** Placement and replication obey classification,
   residency, purpose, retention, encryption, export and deletion policy.
   Replication is a disclosure and creates independently managed copies; delete
   propagation and cryptographic erasure claims require explicit evidence.
9. **Federated observability.** Use globally unique service namespace/name/
   instance identifiers consistent with
   [OpenTelemetry service conventions](https://opentelemetry.io/docs/specs/semconv/resource/service/).
   Preserve trace causation across trust boundaries only under disclosure policy;
   topology, route, queue, replica, quorum and clock-quality evidence are
   first-class. Trace continuity is not authority or effect proof.
10. **Compatibility and rollout.** Mixed versions are normal. Rolling upgrade,
    capability skew, schema evolution, downgrade refusal, canaries and rollback
    are qualified across every supported topology. One healthy instance cannot
    certify the fleet.

#### User-visible federation requirements

- A resource always displays its logical home/authority and relevant site or
  trust-domain boundary; raw infrastructure detail may remain progressively
  disclosed.
- Global search, inbox, portfolio and executive views show which authorities
  were queried, which were unreachable/stale/redacted, and whether results are
  complete enough for the selected decision.
- Users can choose or understand placement constraints such as data residency,
  latency, provider/tool availability, cost, confidentiality and failure domain.
- A remote action preview shows destination authority, route freshness, trust,
  disclosure, expected latency, offline/partition policy and recovery contract.
- Rooms, tasks, workflows, sessions and documents remain navigable across homes;
  unavailable remote owners yield stable references and last-known evidence,
  not broken or silently recreated local objects.
- Operations views distinguish instance health, logical-authority availability,
  quorum/home status, replica lag, route health and end-user journey health.
- Administrators can simulate loss of a host, site, region, route or trust
  relationship and see affected authorities, journeys, data and recovery
  objectives before approving topology change.
- External and executive projections preserve federation gaps: a polished
  aggregate must not hide that one region, organization or authority was absent.

### Sovereign component map

| Component / surface | Authoritative question | Individually useful for | Must not silently become |
| --- | --- | --- | --- |
| Pantheon shell | Which peer projections are available, focused, fresh, or degraded? | Navigation, cross-linking, global discovery, action palette, lifecycle overview | Workflow engine, semantic bus, credential store, or source of peer readiness |
| Overview | What is the bounded current observation of the whole local workspace? | Triage, freshness, degraded-state discovery, guarded-action discovery | A claim that configured means ready or that green telemetry means durable success |
| Agora | Who participates in which rooms, and what ordered conversation facts exist? | Human/agent rooms, threads, addressed messages, moderation, read positions | Task tracker, workflow proof, agent runtime, notification delivery proof |
| Chronos | What work is admitted, eligible, leased, paused, cancelled, or terminal? | Tickets, dependency graphs, scheduling, capacity, attempt attribution | Agent executor, chat system, workflow coordinator, or document owner |
| Moira | What durable multi-step workflow decision and effect frontier is known? | Sagas, compensation, reconciliation, deliberation, accountable selection | Distributed transaction manager, scheduler, model runtime, or owner of peer effects |
| Daimon | What happened in an already-authorized, budgeted agent/model/tool execution? | Sessions, prompts, permissions, progress, cancellation, reattachment, evidence | Work selector, retry scheduler, room, editor, or authorization policy |
| Hestia | Which verified local model artifacts are available, where is an admitted local-inference runtime placed, and what lifecycle/resource/posture evidence exists? | Human-gated catalog discovery/acquisition/verification/retirement; explicit start, readiness, drain, crash/orphan reconciliation and later distributed placement | Provider-behavior qualification, inference execution, task scheduling, or authorization; artifact acquisition never implies instantiation or use |
| Tellus (Dendrite-backed) | Which authored method, surface, fold, source, and freshness evidence is available? | Provenance inspection, method discovery, context validation, stale-grounding warnings | Workflow runtime, semantic proof by routing, or mutation authority |
| Vellum | What source/document state, history, edit, and persistence evidence exists? | Reading, editing, save, durable operation recovery, semantic proposals | Git authority, task completion, workflow state, or agent transcript |
| Hermes | What event is retained on a stream and where is a consumer cursor? | Durable delivery, replay, disclosure, acknowledgement, retention operations | Conversation history, workflow completion, telemetry, or downstream-effect proof |
| Aegis | Does explicit current authority permit this action on this resource? | Issue, attenuate, authorize, inspect, revoke, audit redacted decisions | Authentication provider, credential vault, UI confirmation, or durable workflow |
| Oculus | What authorized events, logs, metrics, traces, profiles and alerts are observable across explicit sources and gaps? | Catalogue, bounded query/follow, pivot, dashboards, SLO inspection and telemetry-pipeline health | Domain history, global causal truth, incident settlement, generic storage engine, or authority |
| Operations | Which Pantheon-owned local peers are live, draining, fenced, or replaced? | Status, supervised startup, reverse drain, exact restart, residue handling | Peer semantic recovery or service-manager/fleet control without a selected profile |
| Configuration | What exact profile, topology, selector, and launch-only capability set is active? | Preflight, diff, capability discovery, safe whole-process replacement | Secret store, live semantic mutation, or implicit authority grant |
| Keystone | Which reusable bounded mechanism and law is shared by multiple owners? | Codecs, framing, replay, effect knowledge, storage gates, safe observation | Universal protocol, generic domain model, or application component |

### The evidence chain

Every aggregate mutation should be explainable as the following chain. Steps may
be omitted when not applicable, but they must not be conflated.

1. **Intent** — a human or identified system requests a bounded outcome.
2. **Admission** — the owning domain accepts the exact request at a revision.
3. **Authority** — Aegis or the owner-specific policy allows the exact action in
   the current context.
4. **Scheduling** — Chronos creates or selects an eligible exact task attempt.
5. **Coordination** — Moira records the durable forward step and effect identity.
6. **Execution** — Daimon invokes the selected provider/tool under a frozen
   budget and reports attributed evidence.
7. **Peer effect** — Agora, Vellum, or another owner accepts its own exact
   command and returns its own receipt.
8. **Delivery** — Hermes retains and consumers acknowledge event records when
   that route is selected.
9. **Observation** — projections refresh from each owner; Pantheon correlates
   them without declaring a new truth.
10. **Settlement** — Moira records completed, failed, compensated, or uncertain
    workflow disposition; Chronos independently records its task outcome.

### Typed relationship model

Pantheon requires a bounded relationship projection. It should contain opaque
references and link types, never copied peer state or bearer authority.

| Link | Source owner | Target owner | Meaning | Explicit non-meaning |
| --- | --- | --- | --- | --- |
| `task-conversation` | Chronos/Moira adapter | Agora | This room/thread is the selected interaction venue for this task/attempt | Room activity is not task progress |
| `task-workflow` | Chronos/Moira | Moira | This exact task attempt participates in this saga | Saga completion is not task acceptance unless Chronos records it |
| `workflow-execution` | Moira | Daimon | This effect requested this execution | Provider output is not peer-effect completion |
| `execution-grounding` | Daimon/Dendrite adapter | Tellus | This execution consumed the identified grounding digest | The digest is not proof that generated claims are true |
| `workflow-document` | Moira/Vellum adapter | Vellum | This effect targets this scoped document/revision | A proposal is not an edit; an edit is not a durable save |
| `workflow-publication` | Moira/Agora adapter | Agora | This effect targets this actor-bound publication | Prepared or dispatched is not accepted |
| `peer-event` | producer adapter | Hermes | This peer occurrence produced this minimized event | Event acknowledgement is not peer-effect completion |
| `operation-authority` | command owner | Aegis | This operation was evaluated at this authority epoch | Decision evidence is not a reusable grant |
| `analysis-source` | analysis owner | data/query owner | This exact source snapshot/frontier was selected by this analysis | Source inclusion does not establish quality, representativeness or causal relevance |
| `analysis-result` | analysis owner | Vellum/decision/work owner | This retained result/claim informed this artifact, decision or admitted work | Query output is not an accepted conclusion or operational mutation |
| `sme-consultation` | Daimon/Moira/analysis owner | Dendrite | This activity tag/fold selected and consulted this exact SME surface digest | Retrieval is not adoption, independent agreement, authority or proof |
| `work-review` | work/artifact owner | Moira deliberation | This exact work revision was challenged under this method, stakeholder cohort, grounding and review interval | Review existence or reviewer agreement is not quality, acceptance, approval or authority |
| `review-grounding` | Moira deliberation | Dendrite | This ensemble plan used these fold, surface, concern and orthogonality identities | Routed roles do not prove actual stakeholder representation, cognitive independence, complete coverage or truth |
| `review-finding` | Moira deliberation | work/decision/risk owner | This attributed finding and its retained disposition concern this exact work/decision revision | Accepted, mitigated, deferred or disputed disposition is not proof that the concern is resolved |
| `human-gate` | Moira deliberation/decision owner | named human gate principal | This exact decision revision was presented because policy or an admitted human-advocate finding required human adjudication | Gate request is not approval or veto; only the attributable current human response settles the gate |
| `client-attachment` | presentation/client owner | peer resource/session owner | This identified client observed or controlled this exact target under this mode and cursor interval | Attachment is not ownership, attention, approval or continued authority after detach |
| `presentation-handoff` | client owner | client owner | This stable target/cursor reference was offered for another client to attach | The reference is not a credential, grant, assignment or provider-session transfer |
| `signal-correlation` | telemetry owner | peer operation/resource/event | This signal was emitted/observed with a declared correlation or exemplar reference | Correlation is not identity, causation, authority, completeness or effect proof |
| `alert-incident` | alert owner | Chronos/Moira incident owner | This alert contributed to opening or updating this incident journey | Alert firing/resolution is not incident existence, diagnosis or recovery |
| `slo-journey` | SLO owner | service/journey owner | This selected journey population contributes to this SLI/SLO window | SLO classification is not an individual journey's semantic result |
| `role-participation` | participant/session owner | journey resource owner | This human or agent acted in this declared role under the identified delegation and interval | A role label is not identity, authority, approval, seniority or accountability transfer |
| `work-mode-instance` | work-mode registry owner | semantic instance owner | This instance pins this admitted mode contract/version and projection profile | Registration does not make the registry/Pantheon semantic owner or grant permission to instantiate/transition/migrate |
| `work-mode-migration` | instance owner | old/new mode registrations | This exact instance accepted this authorized version migration and receipt | Catalogue activation, aliasing or compatibility claims cannot reinterpret old state or prove owner effects reversed |
| `change-candidate` | VCS/build owner | package/provenance owner | This exact source revision and build activity produced this immutable candidate | Authorship or a successful build is not review, authorization or fitness for deployment |
| `candidate-deployment` | deployment owner | target component/instance owner | This candidate digest was admitted to this target/cohort under this rollout revision | Admission or placement is not readiness, promotion, semantic success or user acceptance |
| `deployment-observation` | observation owner | deployment/cohort | This baseline/candidate signal population and probe window observed this deployment | A healthy signal is not effect proof, complete coverage or permission to expand rollout |
| `deployment-decision` | promotion-decision record owner | deployment owner | This accountable principal requested this exact hold/expand/promote/rollback/abort under current observation and authority evidence | Observer evaluation, Aegis allow or record acceptance alone is not execution or semantic success |
| `recovery-policy` | recovery-policy record owner | recovery executor/known-good artifact | This authorized policy revision designates these exact trust, compatibility, retention and last-known-good facts | Promotion, a healthy window or policy storage alone does not perform recovery or reverse data/effects |
| `deployment-rollback` | deployment/recovery owner | prior deployment revision | This rollout invoked the declared recovery transition and recorded its residue | Code rollback is not data reversal, external-effect compensation or proof of recovery |
| `journey-trace` | Pantheon | all | Content-free correlation for navigation and latency | Trace existence is not recovery evidence |

Every link needs: versioned link type, source/target owner, opaque stable
references, creation provenance, visibility scope, freshness, and a broken-link
state. Links must be navigable in both directions when both owners are available,
but one owner's deletion or retention gap must not rewrite another owner's
history.

## Foundational laws and derived requirements

### Distributed-system laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| [FLP impossibility](https://www.podc.org/influential/2001-influential-paper/) shows that deterministic consensus cannot guarantee termination in a fully asynchronous system with even one crash | Timeouts and disconnects cannot be translated into failure. Preserve `outcome_unknown`, fence automatic progress, and provide lookup/reconciliation |
| [Linearizability](https://www.cs.cmu.edu/~wing/publications/HerlihyWing90.pdf) makes a concurrent operation appear to take effect between invocation and response | Peer mutations need revision/incarnation witnesses and a precise accepted point; stale commands refuse instead of retargeting |
| [Leases](https://web.eecs.umich.edu/~mosharaf/Readings/Leases.pdf) use bounded time to maintain consistency despite host/communication failures | Chronos claims expire, renew explicitly, use monotone time, and require fencing so an old worker cannot publish as the current attempt |
| [CAP's formal tradeoff](https://dspace.mit.edu/entities/publication/dfcb6512-af44-4fc9-8e8b-1a8f28f1b53a) applies under partition | Each federated profile must state whether an operation refuses/unavailable or accepts weaker freshness; the UI must expose which |
| The original [Saga paper](https://www.cs.princeton.edu/research/techreps/598) decomposes long-lived work into transactions with compensations | Moira records forward steps and explicit compensation; Pantheon never promises global rollback or atomicity across peers |
| Kafka's [delivery-semantics analysis](https://kafka.apache.org/43/design/design/) limits exactly-once claims to cooperating transactional boundaries | Use exact-input idempotency and owner receipts. Never claim end-to-end exactly once merely because Hermes or one producer deduplicates |
| [Reactive Streams](https://github.com/reactive-streams/reactive-streams-jvm/blob/v1.0.4/README.md) makes bounded asynchronous demand explicit | Feeds, session events, and telemetry require paging/backpressure, gap states, and bounded buffers; overload must not change domain truth |

### Authority and security laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| NIST [ABAC SP 800-162](https://csrc.nist.gov/pubs/sp/800/162/upd2/final) evaluates subject, object, action, and environment | Every protected command binds all four; denials should identify remediable missing facts without leaking sensitive attributes |
| NIST [Zero Trust Architecture](https://csrc.nist.gov/pubs/sp/800/207/final) rejects implicit trust from network location or ownership | Local socket, same UID, Pantheon supervision, or room membership cannot imply semantic authority |
| [Macaroons](https://research.google/pubs/macaroons-cookies-with-contextual-caveats-for-decentralized-authorization-in-the-cloud/) demonstrate monotone attenuation with contextual caveats | Delegated agent authority must only narrow resource/action/time/budget; no adapter may amplify it |
| [Zanzibar](https://research.google/pubs/zanzibar-googles-consistent-global-authorization-system/) makes causal consistency between authorization and object changes a first-class property | High-risk actions must bind an authority epoch/revision and current resource facts; stale allow decisions cannot float indefinitely |
| OAuth [Rich Authorization Requests](https://www.rfc-editor.org/rfc/rfc9396.html) represents fine-grained authorization details and narrowing | Action previews should show the exact structured resource/action/bounds rather than a generic “Allow agent?” prompt |
| NIST [AI RMF](https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-ai-rmf-10) and its [Generative AI Profile](https://nvlpubs.nist.gov/nistpubs/AI/NIST.AI.600-1.pdf) separate governance, mapping, measurement, and management | Agent profiles require named purpose, risk, evaluation, monitoring, accountable owner, incident response, and decommissioning—not just model selection |
| [OWASP Agentic AI Top 10 (2026)](https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/) treats tool misuse, identity/privilege abuse, and cascading failures as distinct hazards | Default-deny tools, content/data boundaries, per-effect approval, least privilege, sandbox evidence, egress control, and kill/recovery paths are required |

### Reflexive change and runtime-assurance laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| NASA's formalization of [Simplex runtime assurance](https://ntrs.nasa.gov/citations/20230017350) separates an advanced controller from a trusted monitor/controller that can preserve safety | Candidate agent/runtime code may operate only inside a declared envelope; an independently trusted monitor and known-good path retain the power to refuse, switch, stop and recover |
| Erlang/OTP [release handling](https://www.erlang.org/doc/system/release_handling.html) distinguishes simple module loading from synchronized suspend/state-change/resume and full runtime restart | “Hot swap” is mechanism-specific. State-format, supervision, dependency or runtime changes require explicit upgrade/downgrade instructions, compatibility evidence and sometimes a drained restart rather than clever live mutation |
| Kubernetes [rolling updates](https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/) gradually replace instances and expose progress, pause/resume and revision rollback | Replicated Pantheon components need bounded surge/unavailability, readiness and progress deadlines, cohort identity, rollout history and stop/rollback controls; rollout mechanics do not prove semantic compatibility |
| [The Update Framework](https://theupdateframework.github.io/specification/latest/) uses separated signing roles, thresholds, versioned metadata and rollback/freeze/mix-and-match defenses | Candidate acquisition and promotion require immutable identity, delegated/threshold trust where appropriate, freshness/version checks and an independent root; online agent credentials cannot become the ultimate installation trust root |
| NIST [SSDF SP 800-218](https://csrc.nist.gov/pubs/sp/800/218/final) groups secure development around preparation, software protection, well-secured production and vulnerability response | Reflexive development must protect source/build/release integrity, record provenance, test security requirements, handle residual vulnerabilities and feed incident learning back into the development lifecycle |
| SLSA [provenance](https://slsa.dev/spec/v1.2/provenance) describes verifiable information about where, when and how an artifact was produced | Every deployable candidate binds source, builder, invocation, materials and subjects to an immutable digest; provenance is evidence for policy, not self-authorization or proof of correct behavior |

Derived invariant: the system under change cannot be the sole authority for the
change, the sole observer of its consequences, or the sole means of recovery.
Recursive improvement is permitted only while that independent control loop
remains reachable, current and outside the candidate's failure domain.

### Epistemic and provenance laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| W3C [PROV-DM](https://www.w3.org/TR/prov-dm/) distinguishes entities, activities, agents, usage, generation, derivation, attribution, association, and delegation | Grounding artifacts need exact entity/activity/agent lineage; mere co-occurrence is not derivation |
| The original [RAG paper](https://proceedings.neurips.cc/paper/2020/hash/6b493230205f780e1bc26945df7481e5-Abstract.html) identifies provenance and updating knowledge as open problems | Retrieved context must carry source identity, version/freshness, extent, and use evidence; a cited answer still needs claim-to-source support checking |
| [in-toto](https://in-toto.io/docs/getting-started/) verifies planned steps, authorized functionaries, and signed link metadata | Agent-produced software artifacts should carry an attestable chain from plan through tools/tests/review, while Pantheon only presents the evidence |
| [Belnap's four-valued logic](https://doi.org/10.1007/978-94-010-1161-7_2) contains both missing and contradictory information without explosion | Moira's `Maybe` must preserve true-only, false-only, both, and neither; disagreement is not flattened into confidence |
| [Dung's argumentation framework](https://doi.org/10.1016/0004-3702(94)00041-X) makes attacks and defensibility of arguments explicit | Deliberation should retain claims, evidence, counterclaims, attacks, and unresolved dissent instead of emitting only a synthesized answer |
| [Pareto optimality](https://ems.press/content/book-chapter-files/27384) identifies nondominated alternatives under multiple objectives | Pareto membership removes dominated candidates but neither proves nor selects a survivor; selection requires accountable policy/authority |

### Data, query, and analytical laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| Codd's [relational model](https://doi.org/10.1145/362384.362685) separates logical relations and queries from physical storage representation | Query adapters expose typed logical schemas and declared dialect semantics; UI convenience cannot infer joins, keys, units or missing-value meaning from storage layout |
| [ISO SQL](https://www.iso.org/standard/76583.html), [ISO GQL](https://www.iso.org/standard/76120.html), SQL/PGQ and W3C SPARQL define distinct data and graph models | Capability negotiation names the exact language/profile/version and result semantics; “supports queries” is too weak for interoperability or reproducibility |
| Pearl's [causal-diagram framework](https://people.csail.mit.edu/tdanford/discovering-causal-graphs-papers/pearl-causal-diagrams-empirical-research.pdf) requires explicit subject-matter assumptions to identify causal effects from observational data | Dependency, provenance, temporal order, correlation and graph paths are not causation; causal claims name graph assumptions, identification method and experimental/observational limits |
| The American Statistical Association's [p-value principles](https://www.amstat.org/asa/files/pdfs/P-ValueStatement.pdf) reject threshold-only conclusions and require transparent reporting | Analysis reports effect sizes, uncertainty, design, assumptions, multiplicity and all material analyses; statistical significance is not truth, importance or decision authority |
| The [FAIR principles](https://www.nature.com/articles/sdata201618) apply findability, accessibility, interoperability and reuse to data, algorithms, tools and workflows | Saved analyses need persistent identities, rich metadata, authorization-aware access, formal vocabularies, provenance, license/usage conditions and domain standards |
| A query is evaluated against a selected source state under a declared isolation/freshness contract | Every result carries source authority and snapshot/frontier or explicitly reports drift; rerunning identical text against changed data is a new observation, not replay of the old result |

### Telemetry and measurement laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| OpenTelemetry's [log model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) distinguishes source occurrence time from collector observation time and correlates, rather than merges, trace context | Preserve both timestamps, source/resource and instrumentation scope; cross-host ordering and causation cannot be inferred from wall-clock display order |
| OpenTelemetry's [metrics model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/) defines temporality, start/end windows, gaps, sums, gauges and histogram populations | Metric queries and charts retain type/unit/temporality/reset/population semantics; delta/cumulative conversion and reaggregation are visible transformations |
| [OpenMetrics](https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md) distinguishes temporally aggregated metric snapshots from singular events | Metrics reduce information deliberately; an aggregate cannot reconstruct exact events, and missing series/NaN require source-specific interpretation rather than being treated as zero |
| Prometheus's [metric-type guidance](https://prometheus.io/docs/concepts/metric_types/) separates counters, gauges, histograms and client-calculated summaries | Choose instruments by semantics; rates account for reset, gauges may decrease, histogram populations can aggregate under compatible schemas, and summary quantiles generally cannot |
| [Little's Law](https://doi.org/10.1287/opre.9.3.383) relates average inventory, arrival/throughput rate and time only for a stable conserved system over compatible populations | Chronos flow metrics may cross-check WIP/throughput/cycle time only when boundaries, averaging window, stability and inclusion rules match; it is not a per-item prediction |
| Observability is lossy under sampling, aggregation, retention, overload and pipeline failure | Every operational conclusion exposes collection coverage and gaps. Telemetry can falsify a health assumption when evidence appears, but silence cannot prove health or absence |

### Internal algebraic laws

Keystone's `EffectKnowledge` is a four-element diamond lattice:

- `Unknown` is the bottom (no conclusive evidence).
- `Failed` and `Completed` are incomparable.
- observing both yields `Conflict`, the top.
- join is commutative, associative, idempotent, and monotone.

It is isomorphic to accumulating the subsets of `{failed, completed}`. This is a
knowledge order, not a chronology or a retry policy. A UI may render conflict and
unknown prominently, but it may not “resolve” them by choosing the friendlier
status.

The component architecture's closed command/outcome algebras derive the
following universal requirements:

- commands, outcomes, refusals, lifecycle states, and confirmation views are
  finite and versioned;
- mutation carries expected revision/incarnation and stable operation identity;
- matching replay observes the original result; changed input with the same key
  conflicts;
- every retained collection has cardinality and byte bounds;
- time, identity, authorization, and capability facts enter explicitly;
- diagnostics are passive and content-safe;
- engine transitions remain deterministic and effect-free;
- safety, liveness, durability, delivery, authorization, and observability claims
  are tested and stated separately.

### Federation-specific laws

| Foundation | Consequence for Pantheon |
| --- | --- |
| A replicated-state-machine consensus algorithm such as [Raft](https://raft.github.io/) establishes one committed log through leader terms and quorums | Consensus must be selected and qualified per logical authority; putting several instances behind a load balancer does not create one writer or durable failover |
| [SWIM](https://research.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf) separates scalable membership dissemination from failure detection and includes suspicion before declaration | Instance membership/liveness may drive routing evidence, but suspicion or timeout cannot transfer task, room, saga, stream, document or policy authority |
| [CRDTs](https://perso.lip6.fr/Marc.Shapiro/papers/2011/CRDTs_SSS-2011.pdf) converge only when their state/operation and delivery conditions are satisfied | Active-active support is an explicit domain algebra with causal, authorization, deletion and resource-bound laws—not a generic federation switch |
| [SPIFFE federation](https://spiffe.io/docs/latest/spiffe-specs/spiffe_federation/) makes trust relationships directional and binds a named foreign domain to explicit bundle endpoint parameters | Cross-domain trust is configured, attributable, revocable and independently authorized; endpoint discovery or certificate validity cannot imply reciprocal trust |
| Matrix's [server-server model](https://spec.matrix.org/latest/server-server-api/) requires homeservers to validate federated events and repair missing history | Remote events remain untrusted input until owner validation; room history repair, state resolution and participant authority require Agora-specific rules rather than a generic event mesh |
| OpenTelemetry's [service resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/service/) distinguish service namespace/name from globally unique concurrent instance identity | Observations identify the logical service and exact serving instance separately; restarts, replicas and failovers cannot alias each other in traces or operations views |

## Component use cases and journeys

### Pantheon shell, Overview, Operations, and Configuration

#### Individual use cases

- Open one workspace and see configured, observing, current, stale, degraded,
  busy, uncertain, and fenced peer states.
- Navigate by surface, recent journey, typed relationship, owner, workstream,
  room, saga, session, document, stream, or operation.
- Search commands and discover unavailable actions with exact prerequisites.
- Inspect a redacted active configuration and resolved capability catalogue.
- Preflight a replacement profile, view a typed diff and minimum restart
  boundary, then confirm one exact replacement.
- Observe Pantheon-owned process readiness, drain order, cleanup receipts, and
  retained residue without confusing them with peer semantic health.
- Export a bounded, content-free support bundle containing versions,
  capabilities, freshness, topology, error classes, and correlation IDs.
- Restore per-user layout, focus, filters, and recent links without restoring
  authority or stale confirmations.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Surface navigation, focus routing, command palette, help, timeline | Qualified | One keyboard model; component-local keys act only when focused |
| Aggregate health/freshness and guarded control discovery | Qualified | Configured is visually distinct from observed/current |
| Local provider onboarding and qualification | Required | Explicit endpoint/catalogue selection; owner-backed redacted configuration; completion and tool-continuation checks; retained conformance/deviation evidence; actionable failure orientation; no JSON/env/Makefile ceremony on the normal path |
| Typed cross-peer links and journey view | Required | Deep-link without copying domain state; broken/stale links remain inspectable |
| Persisted role activation and projection | Required, P0 | Select owner; retain versioned human/agent activation and transition history; project identical current/prior role, scope, delegation, budget, grounding, accountability, policy and expiry semantics across UI/TUI/headless without granting authority |
| Custom work-mode registry and lifecycle | Required, P1 | Select bounded registry owner; declaratively register/version/activate/migrate/deprecate/retire/tombstone modes through J34; exact-version instances preserve sovereign semantic ownership and no executable extension or authority enters through registration |
| Global bounded search | Required | Search owner-provided indexes/results; show source, revision, completeness, and gaps |
| Analysis/query/graph workbench | Required | Compose owner-native catalogues and query profiles; preview plan/authority/cost, bound execution, preserve typed results and lineage |
| Event/log/metric inquiry | Required | Owner-native catalog/query/follow, signal pivots, dashboards/SLOs/alerts, cursors, gaps/sampling/cardinality and headless attach/detach |
| Unified inbox/attention view | Required | Aggregate references to owner-native permission, uncertainty, failure, review, and mention items; no new completion state |
| Action preview | Required | Show owner, exact resource/action, authority source/epoch, revision, blast radius, reversibility, and recovery path |
| Headless observation | Qualified for configured peers | Bounded versioned aggregate JSON; honest partial failure; current mode-specific route catalogue remains visible |
| Full headless semantic parity | Required | Every qualified observation/control/query/recovery journey has versioned bounded inputs, outputs, challenges, receipts, streams and deterministic exit/refusal semantics |
| Dynamic attach/detach and handoff | Required | Attach exact existing resource/session without create; detach affects only the client; reattach resnapshots/reconciles; TUI/CLI handoff carries no authority |
| Persistent session catalogue/watch | Required | Authorized bounded discovery, stable references, cursor/frontier continuation, gaps, multiple clients and current controller state |
| Accessibility contract | Required | Keyboard operation, visible focus, no traps, non-color status, screen-reader labels, reduced-motion/terminal equivalents |
| Supervisor-backed shell detach | Later | Leave explicitly hosted peers running only after durable lifecycle ownership handoff; otherwise drain remains required |
| Management profiles beyond local stdio | Later | Require selected authentication, attach/detach, service-manager, remote/fleet, and credential models |

#### Precedents and practices

- VS Code's [workbench model](https://code.visualstudio.com/docs/editing/get-started/userinterface)
  validates activity navigation, primary/secondary sidebars, editor groups,
  panels, status, restoration, and a command palette. Pantheon should adopt the
  navigational grammar, not VS Code's ambient terminal/filesystem authority.
- Backstage's [software catalog](https://backstage.io/docs/features/software-catalog/)
  demonstrates a portal that organizes independently owned tools around typed
  entities and ownership metadata. Pantheon should similarly expose owners and
  broken integrations rather than hide them behind one generic API.
- W3C [WCAG 2.2](https://www.w3.org/TR/WCAG22/) requires keyboard access, no
  keyboard traps, visible focus, and accessible status messages. WAI-ARIA's
  [patterns](https://www.w3.org/WAI/ARIA/apg/patterns/) give the reference
  behavior for dialogs, grids, tabs, trees, toolbars, and alerts.

#### Non-goals

- No universal edit, send, retry, approve, or complete action.
- No secret or bearer grant in profiles, links, traces, or saved layout.
- No “all systems healthy” inference from process liveness alone.
- No silent live-apply of topology changes.

### Agora — conversation

#### Individual use cases

- Create a room with exact initial membership/admission facts.
- Discover rooms, membership, capabilities, topics, threads, and bounded retained
  history visible to the current participant.
- Speak as an authenticated participant; address one or more members explicitly;
  reply in a thread; react; edit, withdraw, or moderate according to role.
- Advance private read positions without implying attention or delivery.
- Search retained messages with explicit source revision/index generation and
  available/incomplete/stale/unavailable status.
- Invite, join, leave, change roles, kick, ban/unban, and change room policy with
  attributable events.
- Use a task-linked room for human-agent steering while preserving human and
  software participant symmetry.
- Recover a lost create/speak reply only by exact request replay or retained
  actor-publication observation.
- Federate or encrypt rooms only under a selected transport/security profile.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Room/message observation, room creation, speaking | Qualified in Pantheon | Preserve participant binding, revision, exact confirmation, and durable receipt |
| Threads/topics/reactions/revisions/moderation/search | Peer capability; composition to assess | Compose only owner-client capabilities that serve named journeys |
| Task/saga/session relationship cards | Required | Show linked opaque refs and freshness; room text never updates task state |
| Mentions and addressed agent steering | Required | Addressed message creates no execution until separately admitted by Moira/Chronos |
| Agent presence/status projection | Required via typed link | Presence comes from Daimon; Agora only shows linked participant and conversation facts |
| Content safety, retention, legal hold/export policy | Later | Requires deployment policy and explicit disclosure authority |
| E2E group security | Later | MLS requires selected authentication, delivery, key custody, device, and recovery designs |

#### Precedents and practices

- Matrix's [client-server specification](https://spec.matrix.org/latest/client-server-api/)
  provides mature room, event, thread, receipt, sync, membership, moderation, and
  gap patterns. Its separation of read receipts from event delivery reinforces
  Agora's private read-position law.
- W3C [Activity Streams 2.0](https://www.w3.org/TR/activitystreams-core/) offers
  a portable actor/activity/object vocabulary and explicit extension behavior;
  it is useful at federation adapters, not as Agora's internal model.
- IETF [MLS](https://www.rfc-editor.org/rfc/rfc9420.html) and its
  [architecture](https://www.rfc-editor.org/rfc/rfc9750.html) ground future
  asynchronous group encryption, epochs, membership changes, forward secrecy,
  and post-compromise security. MLS does not supply room moderation, delivery,
  authentication, or retention policy.

#### Non-goals

- A message, topic resolution, read marker, or reaction never means task done.
- A room participant does not inherit task, file, tool, or workflow authority.
- Search index absence is not authoritative message absence when incomplete or stale.

### Chronos — work and scheduling

#### Individual use cases

- Create one task or an atomic bounded dependency graph.
- Browse backlog, ready, blocked, leased, paused, uncertain, terminal, cancelled,
  and unrunnable work with explanations.
- Filter/group by workstream, priority, capability, parent/project label, owner,
  time window, attempt, and linked journey.
- Claim an exact task incarnation under an expiring lease; renew, report, release,
  or fence it.
- Configure global/task pause, not-before, capacity, concurrency, priority, and
  scheduling eligibility.
- Cancel one or a batch with exact-reference receipts and dependency propagation.
- Observe task change feeds and resynchronize after retention gaps.
- Inspect linked Moira saga and Daimon attempt without importing either state.
- Reconcile lost admission/cancellation replies by scoped lookup.
- Project tasks into calendar/to-do formats without making calendar clients the
  authority.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Observe and mutate selected task/scheduling profile | Qualified | Writes remain launch-enabled and revision guarded |
| Planning views and change feed | Qualified under flags | Bounded consistent pages, explicit gaps, no prose-scraped eligibility |
| Task templates/intake forms | Required | Versioned schemas, required evidence, risk class, owner, definition of done |
| Typed parent/child and dependency visualization | Required | Hierarchy and dependency are distinct relations; cycles refuse |
| Agent assignment/attempt launch | Required through Moira | Chronos creates attempt reference; it does not start Daimon directly as hidden behavior |
| SLA/deadline/calendar projection | Required | Preserve earliest start, due, estimate, recurrence instance, timezone, exceptions |
| Flow analytics | Required | WIP, throughput, work-item age, cycle time, blocked/uncertain age; explicit populations and censoring |
| Fair/resource-aware scheduling | Later | Must state workload/arrival assumptions, units, reservations, release, starvation and uncertainty accounting |

#### Precedents and practices

- The 2025 [Kanban Guide](https://kanbanguides.org/) grounds explicit workflow,
  WIP control, active management, and flow metrics. Metrics should inform policy,
  never mutate task truth.
- The [Scrum Guide](https://scrumguides.org/scrum-guide) distinguishes product
  goal, ordered backlog, sprint goal, selected work, and a usable increment. These
  are useful projections; Chronos should not hard-code Scrum.
- IETF [RFC 5545](https://datatracker.ietf.org/doc/html/rfc5545) defines stable
  task/calendar identity, due/start/duration, status, priority, recurrence, and
  exceptions. The newer [RFC 10051](https://datatracker.ietf.org/doc/html/rfc10051)
  adds explicit task dependencies, milestones, estimates, scheduling, assignment,
  and richer status.
- GitHub's [sub-issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues)
  and Linear's [views](https://linear.app/docs/custom-views) show expected
  hierarchy, saved filters, shareable views, and project/cycle projections.

#### Non-goals

- No inferred completion from a commit, chat message, process exit, or saved file.
- No implicit retry of an uncertain provider or peer effect.
- No grant/lease reuse after incarnation or authority epoch changes.

### Moira — durable workflow and deliberation

#### Individual use cases

- Admit a workflow from an explicit request and mapping; conversation ingress is
  passive until this command succeeds.
- Inspect the complete durable saga: steps, effects, receipts, attempts,
  compensation state, timers, cancellations, and uncertainty fences.
- Advance an eligible forward step, pause, resume, cancel, fail-terminal, keep
  uncertain, compensate, or reconcile with exact confirmation.
- Coordinate task admission, execution, Agora publication, Vellum mutation, and
  Hermes event production while retaining each peer's identity.
- Recover after coordinator or peer restart from stored intent and historical
  lookup without blindly repeating effects.
- Run a grounded chatbot profile with one bounded answer and policy-selected
  publication.
- Run a workflow/agent deliberation with independent persona lenses, claims,
  evidence, dissent, Pareto frontier, validator results, and explicit fuel.
- Resolve to proven Boolean, authorized choice, four-valued maybe, or bounded
  non-convergence (`Mu`).
- Select one report for action with a second authority decision distinct from
  choosing a Pareto candidate.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Saga inspection/evidence/operator receipts and guarded reconciliation | Qualified | Exact revision, current authority, durable receipt, no retry on uncertainty |
| Agent participation/session/event inspection | Peer-qualified, Pantheon absent | Compose as the central aggregate agent journey rather than duplicating Daimon state |
| Agent prompt/permission/cancel/reattach confirmations | Peer-qualified, Pantheon absent | Owner-native confirmation views; command scoped and exact-revision bound |
| Deliberation inspection | Qualified read-only composition | Pantheon attaches the independently selectable owner endpoint and shows the exact question/method, ensemble admission and assessor evidence, stakeholder representation/standing, claims, grounding, reports, evidence, dissent, candidates and resolution in TUI and headless paths; it infers no fitness, frontier, authority or disposition |
| Stakeholder-ensemble review | Qualified inspection foundation; disposition composition required | Frame exact question/method/cohort; admit independently attributed reports; preserve evidence, dissent, coverage, non-convergence, disposition and resolution without vote-to-truth or vote-to-authority |
| Visual saga graph/timeline | Required | Separate step state, effect knowledge, peer receipt, and operator action |
| Workflow templates and version migration | Required | Versioned authored definitions, compatibility/refusal, dry-run, migration evidence |
| Timer/signal/compensation catalogue | Required | Closed types and exact idempotency/authority/recovery rules per handler |
| Bulk/portfolio workflows | Later | Only after bounded fanout, independent failure, backpressure, and authority policy are selected |

#### Precedents and practices

- The Saga paper and OMG [BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/)
  provide the vocabulary for long-running steps, events, compensation, human
  tasks, subprocesses, and boundary conditions. Pantheon needs a bounded subset,
  not arbitrary executable BPMN.
- Temporal's [activity guidance](https://docs.temporal.io/activity-definition)
  demonstrates durable workflow history, stable activity identity, idempotency,
  heartbeats, and retry policy. Its own documentation notes activities may
  re-execute; Pantheon's stricter no-blind-retry rule is appropriate for
  externally visible, non-idempotent effects.
- Current agent systems make steerable sessions and durable work visible:
  GitHub exposes [issue-to-session-to-PR](https://docs.github.com/en/copilot/how-tos/github-copilot-app/managing-issues-and-pull-requests),
  [session logs and continuity](https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/manage-and-track-agents),
  and [automations](https://docs.github.com/en/copilot/how-tos/github-copilot-app/using-automations).
  Pantheon should improve on these by retaining owner-native authority and
  outcome-uncertainty evidence.

#### Non-goals

- No global ACID or rollback claim.
- No automatic conversion of model output into peer commands.
- No “majority vote means true”; no Pareto survivor means selected.
- No compensation assumed to erase an already-observed external effect.

### Daimon — agent and model execution

#### Individual use cases

- Execute one already-authorized attempt under exact principal, profile, context,
  tool, time, token, call, and byte budgets.
- Open an interactive agent session under a frozen credential-free launch policy.
- Observe presence, session facts, ordered events, usage, provider capabilities,
  tool requests, and permission challenges.
- Submit a prompt, advance a session, approve/deny one exact permission, cancel a
  pending turn, close, or explicitly reattach after process loss.
- Run multiple selected sessions as a bounded set and see partial settlement.
- Resume provider-native state only when the provider advertises and proves the
  matching capability; never replay history as an implicit prompt.
- Invoke host-governed filesystem or terminal capabilities only inside configured
  roots/command registries with fresh Aegis and Chronos checks.
- Recover retained outcome/evidence after daemon restart without another provider call.
- Hand a session from background execution to interactive human supervision.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Presence, session/session-set inspect and control | Qualified in Pantheon | Separate terminal completion from interactive ready/turn/permission lifecycle |
| Prompt, permission, cancel, close, advance, reattach | Qualified | Exact challenge/revision/key; one attempt; honest process-loss uncertainty |
| Live event stream with backpressure/gaps | Required | Ordered bounded cursor, retention state, event kinds, no transcript in telemetry |
| Budget dashboard and forecast | Required | Configured vs consumed vs remaining; hard stop disposition; per-tool/provider attribution |
| Tool/resource manifest | Required | Stable capability IDs, risk class, input/output schema, sandbox/egress evidence, current authority |
| Artifact/output handoff | Required via Moira | Output/proposals are evidence; only workflow-selected adapters may create peer effects |
| Provider portability/conformance | Required | ACP/MCP/provider capability negotiation, TCK evidence, known deviations, version pin |
| Provider setup and local-hosting experience | Required via Hestia | Configure through owner-backed Pantheon surfaces; distinguish Daimon-attached provider endpoints from Hestia-owned artifact/runtime placements; discover catalog entries from selected registries; review source, license, digest/signature, size, compatibility, storage and resource impact; independently confirm acquire/update/retire and start/restart/drain; show lifecycle separately from Daimon model/capability/qualification evidence; select for a session without exposing credentials or provider commands; never imply instantiation, execution authority or workflow completion from acquisition or hosting |
| Remote/background execution | Later | Requires identity, sandbox, network/secret, retention, audit, and incident-response profiles |

#### Precedents and practices

- [Agent Client Protocol v1](https://agentclientprotocol.com/protocol/overview)
  supplies sessions, prompts, updates, permissions, filesystem/terminal
  capabilities, cancellation, and resume negotiation. Daimon correctly treats
  advertised capability values as evidence to validate, not truth to infer.
- [MCP 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28)
  standardizes tools, resources, prompts, authorization, capabilities, caching,
  and extensions. MCP is an inbound projection; it must not replace Daimon's
  native execution/recovery contract.
- Cursor's [background-agent model](https://docs.cursor.com/background-agent)
  shows the expected status/follow-up/takeover journey and also documents the
  exfiltration risk of networked auto-run. GitHub and Codex similarly emphasize
  isolated environments, reviewable changes, steering, and durable session state.

#### Non-goals

- Daimon does not choose work, schedule retry, grant authority, publish a message,
  save a document, or declare task/workflow completion.
- Provider prose is never a tool receipt or factual proof.
- A permission UI approval is never sufficient without the owner/Aegis decision.
- Hestia owns a verified local model-artifact catalog and supervises explicitly
  admitted local-inference runtime placements. Catalog acquisition and runtime
  lifecycle are separate human-gated capabilities.
  Pantheon is its presentation/composition shell; Daimon remains provider
  qualifier and execution owner. Hestia artifact acquisition is a separate
  human-gated network/disk/license decision and never grants runtime or
  execution authority.

### Hestia — inference hosting and placement

**Current qualified owner slice (2026-11-18):** Hestia now binds one exact
prepared runtime incarnation to a private local process, refuses occupied or
non-loopback endpoints, and settles start from bounded TCP-accept evidence with
an observation time and opaque endpoint reference. Its standalone local daemon
retains lifecycle state and process handles across fresh bounded Unix-client
connections: presentation detach does not stop a runtime, discarded-reply
retries recover identity without duplicate start, and clean shutdown refuses
while lifecycle work remains active. Private executable, argument, environment
and readiness configuration never enters the public protocol. TCP accept is an
explicit readiness basis—not model-loaded, provider-conformant,
inference-success or task-completion evidence. Crashed and still-running
readiness-timeout outcomes stay distinct; timeout fences the runtime as
`OutcomeUnknown`. Pantheon now explicitly binds a Daimon provider identity to a
Hestia runtime identity and presents Hestia lifecycle/readiness/incarnation,
model, resource and control evidence beside Daimon qualification at the human
invocation gate and in headless observation. Configured missing, mismatched,
fenced or non-Ready evidence refuses before arming. `hestiactl` supplies bounded
ordinary observe/start/drain/shutdown access to the same owner. The reusable
`hestia-tui` component and Pantheon Hestia tab now present exact bound runtimes
and require review, arm and one-shot confirmation for start/restart/drain; keys
and command-palette actions share one behavior path. A real tmux journey proves
incarnation transitions, Daimon correlation and headless/TUI parity. Persistence
across abrupt daemon loss and orphan reconciliation remain open.

Hestia now also publishes a separately authorized, bounded static artifact
catalog through its ordinary client. Its private adapter streams one selected
source into an owner-private partial file under a hard expected-size ceiling,
verifies SHA-256, and atomically admits only the verified artifact. Pantheon's
Hestia tab can operate artifact-only or mixed profiles; `Tab` switches runtime
and artifact panes, while keys and minibuffer commands share the same behavior
facade. The acquisition sheet discloses model, registry, source revision,
license, format, expected bytes and digest, then requires review → license
acceptance/arm → one confirmation. Headless output independently carries the
same owner-published catalog evidence. A real tmux qualification admits a
bounded fixture, proves TUI/headless parity, and verifies that payload bytes
remain outside the Pantheon repository. Admission explicitly grants no
instantiation, qualification, invocation or scheduling authority.

Live registry search, signature/compatibility/resource-impact evidence,
restart reconciliation of partial transfers, cancellation, multi-file
manifests, update/retirement, runtime instantiation from an admitted artifact,
enforced resource/sandbox preflight and Aegis-backed artifact/lifecycle
authority remain required before this is a complete operational hosting
product. Devstral remains the first intended real dogfood acquisition, subject
to an explicit human storage/license/resource gate. Filesystem permissions are the current
local deployment boundary, not an Aegis-backed authorization claim.

#### Individual use cases

- Admit one exact host-owned runtime profile and model-artifact identity without
  qualifying or invoking the model.
- Discover a bounded selected registry catalog; review provenance, license,
  size, digest/signature, compatibility, storage destination and resource
  impact; then independently authorize download, verify, admit, update or retire.
- Start a new runtime incarnation, observe starting/loading/ready/degraded/
  draining/stopped/crashed/orphaned/uncertain, and retain exact lifecycle evidence.
- Drain or restart only the selected incarnation; fence duplicate/stale endpoints
  and reconcile process loss without assuming that liveness means readiness.
- Present placement, host/site, executable/profile digest, endpoint, artifact,
  quantization/context facts, resource expectations, enforced versus inherited
  sandbox/egress controls and lifecycle budgets without exposing secrets.
- Later, choose among authorized host/accelerator pools and rehome only through a
  guarded workflow that preserves authoritative placement and runtime identity.

#### Required first profile

- A closed versioned public protocol and ordinary client separate observation,
  start, drain, restart and reconciliation; each mutation binds host authority,
  runtime identity/incarnation, expected revision and idempotency identity.
- A private local-process adapter owns executable paths, arguments, environment
  references, readiness probe and OS process handle. Those deployment details do
  not become the distributed protocol or Pantheon configuration semantics.
- The owner refuses an endpoint already in use, missing/mismatched artifact or
  executable, exceeded resource/isolation preflight, unsupported posture, stale
  revision/incarnation and an uncertain prior lifecycle effect.
- Model acquisition/update/delete is a separate human-gated capability with
  network, storage, provenance, license/trust and rollback evidence; it is not an
  implicit consequence of start or provider selection.
- `hestia-tui`, Pantheon and headless clients eventually reuse the same public
  client. Detaching presentation never drains a runtime; explicit operator or
  workflow authority is required for lifecycle mutation.

#### Non-goals

- Hestia does not implement inference APIs, qualify provider behavior, select
  tasks/models, manage agent sessions, schedule retries, grant authority, record
  workflow completion or treat process/port health as successful execution.
- Hestia is not a generic service manager. Its vocabulary and evidence remain
  specific to admitted inference-runtime placement and lifecycle.

### Tellus — grounded methods, context, and provenance

#### Individual use cases

- Observe Dendrite readiness, corpus/profile identity, generation, freshness,
  bounds, and supported grounding capabilities.
- Resolve an exact authored Rune method/state, context tags, fold, SME surface,
  concern lattice, and orthogonality verdict.
- Discover the SME roster and canonical fold vocabulary; retrieve exact ambient,
  activated or reference-depth guidance with stable recommendation identities.
- Query dependencies and graph structure, resolve/read exact references,
  describe one rune or element, route current context through the fold, and
  reconcile consequential observations against the selected FSM/SME corpus.
- Inspect provenance from source entity through parsing/indexing/folding to the
  projection consumed by an execution.
- Compare the execution's retained grounding digest with current source/index
  state and label it current, superseded, stale, incomplete, or unavailable.
- Explain why a source or surface was included/excluded and where its bounded
  extent begins and ends.
- Detect corpus conflicts, missing evidence, changed authority digest, and
  cohort substitution before workflow admission.
- Deep-link to the owning artifact or source without disclosing unauthorized content.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Bounded observation/provenance health | Qualified | Observation-only in Pantheon; no mutation inference |
| Dendrite structural/expertise query vocabulary | Peer-qualified; Pantheon composition required | `query_dependencies`, `resolve_ref`, `read_ref`, `describe`, `describe_element`, `query_orthogonality`, `query_fold`, `list_fold_rules`, `query_surface`, and `reconcile_observations` retain exact load/corpus identities and named failures |
| Ambient/activated/reference SME surfaces | Peer-qualified; Pantheon composition required | Context-budgeted retrieval, fold provenance, stable entry identity, influence evidence and visible unavailable/unmatched routing |
| Stakeholder concern routing and ensemble coverage | Peer-qualified foundations; composition required | Context-to-role fold, exact surface digests, concern roster, pairwise orthogonality, missing/conflicted lens evidence and material-change rerouting without claiming representativeness or independence |
| Cross-corpus narrative search | Known Dendrite gap | Keep `search_narrative` visibly unavailable until structured citation indexing supplies real answer content; no model-fabricated substitute |
| Source/surface/fold detail disclosure | Required | Explicit authorization, size bounds, source revision, extent, freshness, completeness |
| Claim-to-evidence map | Required | Each material claim links to supporting/contradicting extents; unsupported remains visible |
| Tellus diff | Required | Compare two digests/generations and show added, removed, superseded, unresolved inputs |
| Corpus lifecycle | Required in owner tools | Ingest/validate/index/retire/rebuild with provenance and safe activation; Pantheon observes |
| Quality evaluation | Required | Retrieval recall/precision on named suites, attribution fidelity, stale/conflict/abstention tests |

#### Precedents and practices

- W3C PROV supplies the correct lineage vocabulary. Its warning that usage plus
  generation is not automatically derivation is crucial: a context window
  containing a document does not prove the output depends on or supports it.
- SLSA's [provenance model](https://slsa.dev/spec/v1.2/provenance) and
  [in-toto attestations](https://in-toto.io/docs/specs/) are useful precedents
  for immutable subject/material/builder/invocation identities and verification.
- RAG prior art supports retrieval as non-parametric context, but not truth.
  Pantheon therefore needs source freshness, conflict, extent, and attribution
  views rather than a generic “grounded” badge.

#### Non-goals

- Routing, retrieval score, freshness, model citation, or fold membership alone
  does not prove a claim.
- Tellus never grants tool, edit, conversation, task, or workflow authority.

### Vellum — documents and editing

#### Individual use cases

- Open workspace inventory and bounded document snapshots.
- Navigate documents, symbols, annotations, history, and semantic relationships.
- Search text/symbols with source revision and index generation evidence.
- Propose one or multiple revision-bound edits; preview diff and semantic impact;
  validate dependencies; accept or reject atomically where supported.
- Save, replace-and-save, or execute a stable durable operation under independent
  edit/persist authority.
- Recover a lost save/edit reply by exact replay or historical operation lookup.
- Inspect dirty, clean, externally superseded, indeterminate, and persistence
  evidence without flattening them.
- Support linked undo/redo and author-scoped history while retaining external
  write conflicts.
- Hand an agent proposal to a human editor for review without giving the model
  direct persistence authority.
- Export/publish only through separately selected destination and overwrite policy.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Inventory, selected edit/save, durable execute/lookup | Qualified in Pantheon | Exact revision, independent authority, byte-exact persistence, lost-reply recovery |
| Diff/review/annotation/history surface | Required | Stable source coordinates, author/provenance, comment lifecycle, stale mapping behavior |
| Bounded workspace intelligence and agent context manifest | Required | Revisioned workspace inventory and incremental symbol/reference/dependency index; each run discloses included and omitted sources, freshness, retrieval rationale, limits and exact content identities without claiming every workspace byte was prompted |
| Multi-document change set | Required | Declared read/write dependencies, prepare/validate/publish, all-or-refuse owner semantics |
| Language/semantic services | Required through ports | LSP/Tree-sitter profile, version pin, cancellation, freshness, incomplete/error distinctions |
| External writer/collaboration | Required before multi-user claims | Explicit concurrency model, merge/conflict, local undo ownership, saved-state evidence |
| Save As/export/publication | Later | Separate Persist/Rebind/Overwrite authority, destination identity, create-exclusive policy, recovery |
| VCS/worktree integration | Later owner | Explicit repository owner; Vellum may edit files but Pantheon/Vellum must not gain ambient Git authority |

#### Precedents and practices

- The [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
  establishes editor/language-server separation, capability negotiation,
  versioned document changes, cancellation, diagnostics, symbols, references,
  and workspace edits.
- Unicode [UAX #29](https://unicode.org/reports/tr29/) defines grapheme/word/
  sentence boundaries, while [UAX #15](https://unicode.org/reports/tr15/)
  defines normalization and stability. Vellum must make byte/code point/
  grapheme/UTF-16/display coordinates explicit and reject ambiguous conversions.
- CRDT and operational-transformation research informs future collaboration,
  but choosing one algorithm is not sufficient: intention, offline divergence,
  selective undo, metadata bounds, saved state, and external side effects need
  their own laws and tests.

#### Non-goals

- A syntactically valid or semantically clean edit is not authorized, saved,
  reviewed, merged, or deployed.
- Diagnostics and analysis results never become edit authority.
- “Save succeeded” requires the selected adapter's durable evidence, not a clean UI flag.

### Hermes — event ledger and delivery

#### Individual use cases

- Observe bounded service/stream state and retention edges.
- Publish through owner-specific producer adapters using stable producer and
  idempotency identity.
- Subscribe/select a stream and consumer; page/disclose a bounded event range.
- Observe current cursor, lag, retention/publication gaps, and unavailable state.
- Acknowledge an exact range through the peer-owned current-cursor guard.
- Compact a selected stream under exact state and authority with impact preview.
- Replay after consumer restart and deduplicate at the application boundary.
- Correlate minimized workflow/conversation events without leaking payload into
  Pantheon traces or activity receipts.
- Quarantine/dead-letter or repair only under separately specified owner semantics.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Ledger/cursor observation, disclosure, acknowledge, compact | Qualified in Pantheon | Exact stream/consumer, bounded disclosure, stale guard, uncertainty fence |
| Producer catalogue/append | Adapter-owned, not a Pantheon command | Every producer schema has owner, version, idempotency, privacy and replay contract |
| Subscription/consumer management | Required | Explicit create/retire/reset authority and retention impact; no global catalogue leakage |
| Lag/gap/retention planning | Required | Sequence edges, age/bytes, capacity forecast, slow-consumer policy, irreversible-loss preview |
| Schema registry/compatibility | Required | Owner/version, backward/forward policy, unknown event handling, bounded payload |
| Dead-letter/re-drive | Later | Requires per-event effect idempotency and authorization; redrive is never a generic retry |

#### Precedents and practices

- CNCF [CloudEvents](https://github.com/cloudevents/spec/blob/main/cloudevents/spec.md)
  standardizes source, type, subject, identity, time, schema, and data context.
  It is suitable at adapter boundaries; semantic payloads stay owner-defined.
- Kafka's producer/consumer/idempotency/transaction analysis grounds separate
  publication, processing, and offset claims.
- W3C [Trace Context](https://www.w3.org/TR/trace-context/) and OpenTelemetry
  [semantic conventions](https://opentelemetry.io/docs/specs/semconv/) support
  cross-service correlation. Trace/baggage must be content-safe and must not
  carry authority or recovery truth.

#### Non-goals

- Acknowledgement proves only the selected consumer cursor advancement.
- Hermes is not an audit log merely because it is durable; audit requires
  identity, completeness, tamper/retention policy, access control, and review.
- Hermes is not a general log, metric, trace, profile, alert or SLO backend;
  bridging any of those requires a selected adapter and preserves signal type.
- Event arrival or absence never proves the external business effect by itself.

### Aegis — authority

#### Individual use cases

- Inspect the current authority namespace/epoch, bounded redacted grants,
  delegations, and decision evidence.
- Issue a scoped grant to an authenticated principal/session.
- Attenuate resource, action, time, count, budget, or contextual bounds without
  amplification.
- Authorize one exact request under current subject/object/action/environment.
- Revoke the epoch and observe old grants become unusable.
- Run a bounded managed administration session with named operations while
  numeric handles remain private and connection-scoped.
- Abandon uncertain/session-local authority on disconnect and deliberately
  rebuild under new operation identities.
- Explain denial in terms of missing/expired/stale/refused facts without
  disclosing secrets or reconstructing bearer grants.
- Audit who decided what, on which resource and epoch, while keeping decision
  evidence non-reusable.

#### Required capabilities

| Capability | Status | Acceptance intent |
| --- | --- | --- |
| Observe, issue, attenuate, authorize, revoke, managed session | Qualified in Pantheon | Exact confirmation/current epoch, session-local handles, one attempt, uncertain fence |
| Resource/action vocabulary browser | Required | Owner-defined schema, examples, risk, delegation rules, current policy version |
| Authority request inbox | Required | Group owner-native pending requests; approve/deny with exact details and expiry |
| Break-glass workflow | Required for production | Named accountable principal, narrow scope/time, dual control where policy requires, immutable incident link, revocation |
| Policy simulation/diff | Required | Evaluate candidate facts without issuing grants; compare policy versions and impacted journeys |
| Credential/federation integrations | Later | Aegis consumes authenticated facts; it does not become password/OAuth/device/network authenticator |

#### Non-goals

- Pantheon focus, OS ownership, process supervision, user confirmation, or agent
  identity never grants authority by itself.
- Redacted decision evidence cannot be replayed as authorization.
- Revocation cannot retroactively erase a completed external effect.

### Keystone — shared mechanisms

#### Individual use cases

- Adopt bounded primitives, codecs, framing, replay, storage gates, private Unix
  publication, effect knowledge, lifecycle obligations, and passive observation.
- Run law suites and consumer qualification before promoting a mechanism from
  reference to adopted or qualified.
- Pin an immutable source revision and audit which APIs/profiles are qualified.
- Fault-inject dispatch, commit, corruption, saturation, identity replacement,
  and cleanup boundaries.

#### Required capabilities

- A machine-readable mechanism/profile/consumer/evidence catalogue.
- Property/law tests, adversarial decode tests, allocation/bound tests, and at
  least two real consumers for general qualification.
- Explicit nonclaims for durability, power loss, authentication, retry,
  migration, and domain semantics.
- Compatibility and migration records for every adopted wire/storage mechanism.
- No “shared” extraction merely because two types have similar names; shared
  failure meaning and laws must match.

## Aggregate user journeys

### J1. Create a ticket that invokes an agent with a user-interaction room

This is the cornerstone journey requested for Pantheon.

**Current qualified presentation slice (2026-11-18):** Pantheon now composes
Chronos task admission, exact owner binding, a fresh per-task Daimon session,
guarded agent turns, direct or group-direct Agora task conversation, and a refresh-backed
return to the Kanban board. `Shift-N` and the minibuffer action share one
behavior route; Pantheon retains the bounded task-form text, waits for Chronos's
applied receipt, then asynchronously requires Daimon's validated owner catalogue
to map that one exact Chronos task ID to the frozen execution task key. Missing,
ambiguous, wrong-owner or mismatched evidence stops before session creation and
orients the operator: the ticket exists, but no agent was invoked. After the
match, a human reviews and confirms one fresh owner-created Daimon session and
the first prompt. Pantheon then prefills an Agora conversation between the
configured operator and one legacy agent or a bounded exact agent cohort,
retains Agora's ordinary review/arm/confirm gate,
and separately reviews the exact durable Chronos-task-to-room relationship.
Only an explicitly selected addressed message can open the ordinary guarded
prompt flow; ambient chatter remains inert. A bounded participant-to-session
route selects each exact configured Daimon session independently of the UI's
ambient selection, and every routed session must again be owner-projected Ready.
The configuration surface shows the exact cohort and routes; every action reports
success/refusal plus the next safe step. The single-agent hosted tmux evidence is
[`tests/live/work_item_agent_journey.sh`](../tests/live/work_item_agent_journey.sh).
The shared-context gate uses a distinct ambient task agent, creates a human+SRE+
project-management group, promotes “Do we have the right ensemble?” only after
explicit review, and proves exactly two routed receipts while excluding the
ambient session; see
[`tests/live/agora_shared_agent_context.sh`](../tests/live/agora_shared_agent_context.sh).
The selected Moira deliberation is now an independent presentation endpoint:
the Moira surface switches between saga, participation and deliberation panes
through the behavior facade, while headless reuses the same owner-defined exact
projection. A real read-only daemon gate proves ensemble admission, assessor and
coverage/equity digests, authorized-representative standing, accountable
principal, mandate, conflict, evidence and dissent in both paths; see
[`tests/live/moira_deliberation.sh`](../tests/live/moira_deliberation.sh). This is
inspection, not a human finding disposition or authority grant.
The selected execution profile now freezes an explicit opaque task key,
incarnation and attempt alongside principal, harness profile, context/tools and
budgets; prompt, permission and reattach operations retain that exact attempt
while receiving fresh execution/idempotency identities. TUI and observation-
only headless projections show the same credential-free facts and explicitly
mark role/work mode, principal type, delegation, accountable owner, authority,
policy epoch and expiry as `not_published`. The live fixture rejects any
different task envelope. Daimon now also retains a service-owned, content-free
receipt for the exact ordered context evidence admitted to each successful
open, prompt or reattach. Its TUI presentation and Pantheon's observation-only
headless projection show the same execution, manifest, Vellum reference/source
digest and byte count without publishing the document content; Moira's public
projection binds that receipt into its retained provider-evidence digest.
Refusal or uncertainty stops the chain. Agora's private-local owner now also
supports a versioned, bounded create-time cohort, and the hosted lifecycle gate
bootstraps distinct human and agent participants without granting Pantheon
runtime registration authority. This slice does **not** yet claim dynamic exact
task-attempt creation, a Moira workflow, dynamic Vellum document selection and
proposal publication, per-effect Aegis decisions, Dendrite context assembly,
runtime Agora participant administration, an independently executed stakeholder-
ensemble method, verification of standing/mandate claims, human-gated finding disposition, or
evidence-backed Chronos status and comment transitions. Those remain Required
below and need owner-published contracts rather than presentation-side inference.

Moira's selected participation surface now adds one bounded next step for an
already-admitted agent member: Pantheon reviews the exact retained binding,
arms the action, and sends only `PrepareOpen { participation }`. Moira alone
checks current pristine Admitted state and derives the storage/revision-bound
Open operation identities; the durable pending operation replays after owner
restart without a second mutation. This closes presentation-to-owner effect
preparation but does not admit the participation, dispatch a worker, create a
new Daimon session, or connect the operation automatically to newly admitted
Chronos work.

The configured one-step Moira owner worker is now an explicit follow-on
composition seam rather than a hidden Pantheon-to-Daimon call. Its opaque
configuration remains owner-controlled; Pantheon launches it only after the
selected participation retains a prepared Open and accepts only a bounded
process receipt naming that exact participation, outcome and revision. This is
implemented and owner-process-tested, but a combined live Pantheon gate remains
required before worker dispatch is considered qualified J1 evidence.

1. In Chronos, the user creates a task with objective, acceptance criteria,
   risk class, required capabilities, budget envelope, due/not-before facts,
   dependencies, and an explicit “agent-assisted” execution profile.
2. Chronos durably admits the exact task graph and returns task/incarnation
   references. No agent has started.
3. A Moira workflow template is instantiated with links to the task and desired
   conversation policy. Moira records the task mapping and effect identities.
4. Agora creates a task room (or binds an existing exact room) with the human,
   selected agent participant identity, and optional reviewers. The room receipt
   is linked to the saga/task; room creation is not task start.
5. Aegis evaluates separate authority for task claim, grounding reads, tool use,
   document edits, publication, and any external effects. Grants are scoped and
   attenuated to the exact journey.
6. Chronos selects an eligible exact attempt and issues a lease. This does not
   execute the agent.
7. Moira prepares an execution effect. Daimon opens the agent session with frozen
   profile/context/tools/budgets and the exact task attempt.
8. Dendrite supplies exact grounding identities and freshness evidence. Daimon
   retains what context was actually admitted.
9. Pantheon displays one journey view: task state, saga frontier, room, agent
   session events/budgets, grounding, candidate artifacts, authority requests,
   and receipts—each labeled by owner and freshness.
10. The human steers the agent by addressed Agora messages. An adapter admits an
    addressed message into Moira/Daimon only through an explicit, replay-safe
    command; ordinary room chatter is never interpreted as a prompt.
11. Tool permissions appear in the attention view with exact command/resource,
    originating task/saga/session, risk, bounds, and consequence. Aegis and the
    command owner decide after human confirmation where policy requires it.
12. Daimon emits ordered progress/evidence and typed proposals. Proposals do not
    modify peer state.
13. Moira selects authorized proposals and invokes Vellum/Agora adapters under
    independent stable operation IDs. Vellum edits/saves; Agora publishes a
    task-room response; Hermes may carry minimized wake/workflow events.
14. Lost replies enter `outcome_unknown`. Pantheon offers only owner-supported
    lookup/replay/reconcile actions and explains why blind retry is unsafe.
15. The human reviews diffs, test evidence, claims/sources, dissent, and remaining
    risks. Acceptance is explicit and may require a separate action authority.
16. Moira settles the saga. Chronos records task outcome independently. The room
    remains a conversation history; it does not become the completion record.

Necessary aggregate UI:

- a stable journey URL/reference;
- owner-labeled status cards with timestamps/revisions;
- task ↔ saga ↔ room ↔ session ↔ document links;
- a chronological, content-safe operation timeline plus owner-native content
  panes where authorized;
- a unified attention queue for permissions, uncertainty, review, mention, and
  degraded-owner states;
- a “next safe action” explanation derived from owner capabilities;
- distinct controls for pause scheduling, cancel execution, close session,
  cancel workflow, compensate effect, and archive room.

### J2. Convert conversation into work

1. A participant marks an exact Agora message/thread as a candidate work request.
2. Pantheon previews bounded message identity/room/revision and asks for task
   fields; it does not scrape or infer final semantics silently.
3. A Moira intake command records the explicit mapping and causation.
4. Chronos admits the task/graph and returns its own reference.
5. Agora optionally receives an addressed confirmation with the new task link.
6. Duplicate intake replays return the same mapping; changed task input conflicts.

### J3. Schedule recurring autonomous maintenance

1. Define a Chronos task template and RFC 5545/10051-compatible recurrence
   projection, timezone, exception dates, estimate, window, dependencies, and
   concurrency/resource limits.
2. Bind a Moira workflow version and a Daimon execution profile with hard budgets.
3. Aegis issues only the recurring action scopes and expiry policy required.
4. Each recurrence creates a distinct task incarnation and attempt; it never
   reuses a prior effect identity.
5. Results enter a selected Agora room and/or Vellum report only through owner
   adapters. Hermes carries minimized notifications.
6. Missed, overlapping, paused, or uncertain occurrences follow explicit policy;
   “catch up” is never an implicit burst of retries.

### J4. Run multi-agent deliberation before a risky change

1. Select a Rune-authored method/state and Dendrite-verified fold/surfaces.
2. Moira admits independent persona lenses with named concern coverage,
   orthogonality evidence, budgets, and stopping fuel.
3. Chronos owns attempts; Daimon executes each member independently.
4. Moira retains attributed claims, evidence, counterclaims, dissent, and
   correlated-failure warnings before synthesis.
5. Deterministic validation yields proofs only for decidable predicates. Pareto
   analysis yields nondominated candidates only.
6. An accountable Aegis-authorized choice selects one candidate, and a separate
   action authorization selects what may affect Agora/Vellum/other peers.
7. `Maybe` and `Mu` remain valid terminal outcomes and are presented as such.

### J5. Produce and review a grounded code/document change

1. Chronos task and Moira saga bind the target workspace/document scope.
2. Dendrite supplies source/method/semantic grounding with revision and freshness.
3. Daimon produces a typed proposal with read dependencies and evidence digest.
4. Vellum validates current document revisions, coordinate mappings, authority,
   and semantic dependencies before preparing a change set.
5. Pantheon presents source-attributed diff, diagnostics, tests, unresolved
   claims, and permissions.
6. Human acceptance invokes Vellum's edit/save operations. Git/worktree/PR work
   requires a separate future VCS owner; a saved document is not a merged change.

### J6. Human takeover and handback

1. A user opens an active journey and requests takeover.
2. Chronos pauses new eligibility or preserves the current lease according to
   explicit policy; Moira pauses at a safe boundary.
3. Daimon cancels only the pending turn or transitions to a supported handoff;
   process loss after dispatch remains uncertain.
4. The human edits in Vellum and converses in Agora under their own identity.
5. Handback creates fresh revision/context/tool evidence and, when necessary, a
   new attempt. It never resumes under stale authority or hidden history replay.

### J7. Permission escalation and revocation

1. An agent requests a tool/effect not covered by current authority.
2. Pantheon shows exact subject, resource, action, environment, reason, scope,
   duration, count/budget, originating journey, and alternatives.
3. The user narrows or denies; Aegis issues/attenuates under current epoch.
4. The owner revalidates immediately before the effect.
5. Revocation blocks future use and is visible in the journey; it does not claim
   to undo effects already accepted by another owner.

### J8. Recover an uncertain effect

1. A connection drops after dispatch and the client reports `outcome_unknown`.
2. Pantheon marks the exact effect and downstream saga frontier fenced.
3. The owner-specific recovery contract is selected: exact replay, historical
   lookup, retained observation, or operator disposition.
4. Evidence joins monotonically: unknown may become completed or failed; mutually
   inconsistent evidence becomes conflict, never silently picks a winner.
5. Moira advances/compensates only after definitive evidence or an explicit,
   authorized operator decision that preserves the uncertainty record.

### J9. Incident response and safe local restart

1. Overview detects stale/degraded/uncertain state through bounded observation,
   not through semantic guesses.
2. The operator opens owner runbooks and sees active work, uncommitted/uncertain
   effects, dirty documents, consumer lag, authority epoch, and residue.
3. Pantheon preflights the exact replacement profile and calculates the minimum
   restart boundary and drain order.
4. Guarded work settles; peers drain in reverse dependency order. Every shutdown
   receipt is retained even when one peer refuses.
5. Any non-clean/unknown drain fences replacement. Clean drain permits exact
   process replacement with digest-bound configuration.
6. After reopen, each owner proves its own durable state; Pantheon re-observes and
   correlates the incident operation ID without treating it as peer identity.

### J10. Event-driven workflow trigger

1. A peer adapter publishes a versioned minimized event to Hermes under stable
   producer/idempotency identity.
2. A selected consumer observes the event and cursor.
3. Trigger policy explicitly maps the event schema to a Moira workflow intake;
   event presence alone does not authorize or admit work.
4. Moira/Chronos/Aegis validate mapping, idempotency, authority, and capacity.
5. Only after durable workflow admission does the consumer acknowledge according
   to the selected delivery policy.
6. Crash between admission and acknowledgement replays safely by identity.

### J11. Compliance/audit reconstruction

1. Select a journey and bounded time/resource scope.
2. Gather owner-native task allocation, saga decisions, execution evidence,
   grounding provenance, authority decisions, document receipts, conversation
   publications, event cursors, and management receipts.
3. Verify identities, versions, revisions, digests, gaps, retention boundaries,
   and conflicting evidence.
4. Export a redacted manifest with source pointers and verification results.
5. State nonclaims: missing retention is not absence; telemetry is not a receipt;
   signatures/immutability/completeness depend on selected owner profiles.

### J12. Multi-workstream portfolio view

1. Select multiple Chronos workstreams and save a user-scoped projection.
2. Aggregate only comparable owner-defined fields and flow populations.
3. Navigate to linked sagas/rooms/sessions/documents without merging them.
4. Capacity and prioritization changes remain per-workstream guarded commands.
5. Cross-workstream dependency or shared-resource scheduling is a new owner
   contract, not inferred from labels.

### J13. Turn an idea into a delivered, operable capability

1. A requester opens a lightweight Agora/Vellum ideation space and records the
   motivating problem, constraints, observations, and desired outcome.
2. Humans and optional Daimon agents diverge into alternatives. Dendrite attaches
   relevant evidence; authorship, uncertainty, and contradictions remain visible.
3. A strategy review selects an option and success measures, preserving rejected
   alternatives and rationale. Explicit promotion admits architecture and
   engineering work to Chronos and a versioned Moira workflow.
4. Architecture and security reviews create linked decisions, boundaries,
   threats, controls, tests, operational obligations, and accountable approvals.
5. A timeboxed prototype tests the riskiest assumptions. Its evidence supports a
   discard, revise, or engineer decision; prototype output is never implicitly
   production-ready.
6. Engineering, testing, packaging, and delivery each produce owner-native
   receipts. The delivered artifact is bound to source, tests, provenance,
   approval, destination, rollout and recovery evidence.
7. Nominal operations observe service/user outcomes. Incidents create linked
   response journeys. Retrospectives and subsequent insight may amend strategy,
   architecture, backlog, controls, runbooks, or the product itself.

### J14. Perform architecture and security design review

1. Select exact requirements, current-system revisions, data classifications,
   quality attributes, and deployment assumptions.
2. Model system context, component/data/control flows, trust boundaries, assets,
   failure modes, threats, abuse cases, and operational dependencies in Vellum.
3. Ground applicable standards and prior decisions through Dendrite. Independent
   reviewers or Daimon sessions challenge assumptions and seek missing concerns.
4. Moira compares alternatives against explicit, possibly conflicting criteria;
   uncertainty and dissent survive synthesis.
5. Reviewers request changes; accountable owners accept residual risks and
   decisions under Aegis policy and separation-of-duties constraints.
6. Accepted decisions create Chronos implementation, validation, monitoring,
   migration, and revisit work. Changed assumptions or observed failures can
   supersede, but never erase, the decision.

### J15. Run a rapid prototype and evaluation loop

1. State a falsifiable hypothesis, riskiest assumption, audience, time/resource
   box, evaluation method, and safety/data constraints.
2. Aegis grants an isolated, minimum tool/data scope. Daimon and Vellum create the
   prototype without production credentials or implicit delivery rights.
3. Moira runs the evaluation matrix across representative scenarios, including
   expected, edge, adversarial, accessibility, and failure/recovery behavior.
4. Results retain environment, versions, inputs, oracle, sampling, failures,
   limitations and qualitative feedback. Observability distinguishes prototype
   behavior from the instrumentation's own gaps.
5. A reviewer chooses discard, revise, preserve as experiment, or promote to
   engineered work. Promotion creates new acceptance, architecture, security,
   test, packaging and operational obligations rather than relabeling the demo.

### J16. Convert an operational incident into durable insight

1. An owner-native signal or user report starts an incident under an explicit
   impact and uncertainty statement; telemetry alone does not declare root cause.
2. An Agora incident room, Chronos response work, Moira runbook, Hermes event
   evidence, and authorized Daimon investigation are linked without merging their
   records. Break-glass actions are time-bounded and independently attributable.
3. Operators contain impact, recover service, verify user-visible outcomes, and
   reconcile uncertain or duplicated effects before declaring resolution.
4. A blameless timeline distinguishes observed facts, interpretations, decisions,
   counterfactuals, and still-unknown conditions. Contributing conditions are not
   compressed into one convenient cause.
5. Retrospection derives candidate insights with evidence and confidence.
   Accepted improvements become owned strategy, architecture, security,
   engineering, test, observability, runbook, or training work.
6. Later effectiveness review verifies whether actions reduced the named risk;
   closing follow-up tickets alone is not proof of learning.

### J17. Connect individual work to executive strategy and decisions

1. Leadership records an objective with rationale, assumptions, measures,
   horizon, constraints, accountable owner and review triggers; it is not yet a
   promise that every related activity will succeed.
2. Portfolio and team owners create typed contributions from initiatives,
   decisions, risks and Chronos work. Contributors can see why their work
   matters and challenge a broken or obsolete linkage.
3. Pantheon summarizes owner-native progress, outcomes, forecasts, incidents,
   evidence gaps and confidence without equating task counts with value.
4. Material deviation produces a decision brief: what changed, evidence,
   alternative explanations, scenarios, tradeoffs, recommendation, dissent,
   requested decision and deadline.
5. An executive decision records accountable rationale and creates or changes
   bounded commitments, work, risk acceptance or resource constraints through
   their owning systems. The brief itself does not mutate those owners.
6. Contributors receive the decision and its implications at an appropriate
   level of detail. Later outcome review tests the original assumptions and
   measures, allowing the strategy to adapt without rewriting history.

### J18. Collaborate with and report to an outside stakeholder

1. An internal owner names the external party, relationship, purpose, expected
   participation, information classes, jurisdictions, duration and sponsor.
2. Aegis and the identity boundary establish least-privilege, expiring access to
   selected rooms, tasks, artifacts, milestones or evidence projections. A
   preview shows both disclosed and intentionally omitted fields.
3. The stakeholder contributes questions, evidence, review, acceptance, dispute
   or feedback under their own identity. Their input remains attributable and
   cannot silently authorize internal execution.
4. Notifications and reports communicate commitments, changes, incidents and
   outcomes in agreed language and cadence, including confidence, freshness and
   a correction/escalation route.
5. Export and audit evidence records exactly what was disclosed. Revocation or
   expiry blocks future access without claiming to recall already received data.
6. Relationship closure transfers any open obligations, preserves required
   records, revokes grants and credentials, and verifies deletion or retention
   according to the selected contract and policy.

### J19. Run one journey across distributed component homes

1. A user opens a task whose Chronos home is in one site, Agora room home in a
   second, Vellum workspace home in a third, and eligible Daimon workers in
   several regions. Pantheon resolves stable authority references to current
   routes and displays freshness, trust and placement constraints.
2. Moira pins the exact peer authorities, route epochs and operation identities
   before requesting effects. Aegis authorizes each cross-domain action under
   the destination's current policy; transport identity grants nothing alone.
3. Chronos selects an eligible attempt and Daimon placement policy chooses a
   worker meeting model/tool, data-residency, sandbox, latency and budget needs.
   The worker receives an exact attempt, not a transferable Chronos lease.
4. The agent converses through Agora and proposes Vellum changes at their named
   homes. Every remote mutation returns an owner receipt; Hermes delivery and
   replicas carry observation without becoming effect proof.
5. A link partition leaves the room readable from an explicitly stale replica
   while document mutation refuses and the already-dispatched effect becomes
   uncertain. The UI does not redirect writes to a convenient replica.
6. Reconnection resumes by owner lookup/replay and Moira reconciliation. The
   final journey reports per-authority outcomes and gaps, not one synthetic
   “distributed success.”

### J20. Replace or fail over a component instance without duplicating effects

1. Operations observes instance degradation separately from logical-authority
   availability and determines whether the selected profile is single-home,
   replicated-state-machine, shard, replica or worker-pool.
2. For planned replacement, the controller stops admission, drains or transfers
   leases, records replica/quorum/home frontiers, fences the old incarnation and
   verifies durable state before activating the replacement route.
3. For unplanned loss, a timeout creates suspicion only. Promotion occurs solely
   through the authority's consensus/home-transfer contract; otherwise mutation
   remains unavailable or read-only.
4. Clients reject stale incarnation, leader, lease, route and authority epochs.
   Equal uncertain requests use the original identity and lookup/replay contract.
5. The replacement proves the original receipt/state frontier before work
   resumes. Operators reconcile any ambiguous in-flight effects rather than
   resubmitting them under new identities.
6. The topology receipt records affected resources, old/new instances and
   incarnations, evidence, downtime/data-loss bounds and unresolved uncertainty.

### J21. Move a logical authority between sites or rebalance shards

1. An administrator previews source/destination sites, authority namespaces,
   resources, dependencies, data classes, residency, capacity, compatibility,
   replication lag, recovery objectives and affected journeys.
2. The owner creates a versioned rehome plan with a cutover epoch, quiescence or
   consensus transition, snapshot/event frontier, validation and abort limits.
3. Data transfers under explicit disclosure authority and integrity checks.
   Catch-up never exposes the destination as writable before its promotion law.
4. Cutover fences the source writer, validates destination state, atomically or
   monotonically changes the authoritative directory, and retains redirects only
   as bounded routing hints.
5. Clients with stale routes refuse or follow authenticated, loop-bounded owner
   redirects while preserving operation identity and expected resource revision.
6. Post-cutover verification covers semantic state, receipts, replicas,
   consumers, policies, links and user journeys; source retirement follows the
   declared retention/erasure policy.

### J22. Operate through a site or trust-domain partition

1. Pantheon displays reachability by authority/site and shows the configured
   behavior of every affected operation: refuse, stale read, bounded queue,
   quorum operation or convergent local update.
2. Executives and operators see incomplete portfolio/incident views with exact
   missing populations. Users see which actions remain safe and which are fenced.
3. Trust-bundle, credential and revocation freshness limits are enforced; stale
   trust never silently converts into a grace-period permission unless the exact
   policy explicitly allows and audits it.
4. Store-and-forward queues retain canonical requests, expiry, authority epoch,
   ordering and capacity. Saturation refuses before dropping or duplicating work.
5. Healing triggers authenticated session renegotiation, topology resnapshot,
   replica gap repair, exact replay/lookup and semantic reconciliation before
   normal mutation resumes.
6. Conflicting evidence, unauthorized offline effects, expired work and data
   that cannot be reconciled remain visible for accountable disposition.

### J23. Analyze federated data and inspect its relationship graph

1. An analyst states a question, intended decision, population, time horizon,
   sensitivity, acceptable freshness and stopping budget. Pantheon discovers
   authorized owner-native table, stream, graph and journey-query capabilities.
2. The analyst inspects schemas, semantics, quality and provenance. Context tags
   activate Dendrite's data, statistics, privacy, security and relevant domain
   SMEs; the record shows why each role was routed and which surface digest was
   consulted.
3. A human or Daimon agent authors a parameterized query. Pantheon shows the
   compiled dialect/profile, sources, snapshots, joins/mappings, remote fan-out,
   authorization, expected rows/bytes/time/cost and completeness before running.
4. Bounded execution streams typed partial results with cancellation. Each
   source records its own frontier, unavailable/redacted partitions, truncation,
   sampling and errors; partial federation is never presented as a complete zero.
5. The analyst moves between tabular/statistical views and the typed relationship
   graph, exploring paths, cycles, clusters and temporal changes. Edge semantics
   remain visible and a dependency/correlation path is not labeled causal.
6. A Vellum analysis artifact preserves question, query/plan, parameters,
   sources, schemas, code/environment, transformations, visualizations, methods,
   SME influence, findings, uncertainty, dissent and reproducibility evidence.
7. Review challenges data quality, method, interpretation and disclosure. An
   accepted insight may create Chronos work or a decision record through explicit
   admission; the query result itself never mutates operational owners.

### J24. Use ambient agentic SMEs throughout a work journey

1. A Daimon execution profile selects a Dendrite corpus/fold, context budget,
   mandatory consultation points and optional advisory behavior. It loads only
   the relevant ambient kernels with exact role and content/surface digests.
2. As work changes—from ideation to architecture, data handling, external calls,
   testing, delivery or incident closure—the agent emits explicit activity tags
   or Moira supplies authored FSM-state tags to Dendrite's same canonical fold.
3. Dendrite returns matched roles and exact activated surfaces. Unmatched tags,
   unavailable expertise, changed corpus, non-orthogonal roles and coverage gaps
   are visible; the agent cannot fill them by inventing hidden SME content.
4. The agent consults reference-depth material only as needed, then attributes
   recommendations it adopted, rejected or used to revise the work. Retrieval
   does not count as influence and agreement does not count as independent votes.
5. A user can inspect the roster, activation rule, guidance, foundations and
   contribution; add another SME; challenge optional advice; or escalate a corpus
   defect. Mandatory safety checks remain explicit workflow/policy gates.
6. Context compaction or handoff reloads only missing pinned material by digest.
   Corpus change triggers the selected pin/refresh/review policy, never silent
   substitution under the same role name.
7. The journey closes with SME coverage and influence evidence, unresolved
   dissent and improvement feedback. SMEs themselves never approve, authorize,
   publish, edit, schedule or declare the work complete.

### J25. Attach a TUI to existing work, detach, and return later

1. A user starts Pantheon without creating or hosting new domain resources and
   selects an authorized catalogue containing an existing Chronos task, Moira
   saga, Agora room, Daimon session and Vellum artifact across several homes.
2. The TUI resolves each exact authority, authenticates, negotiates capability
   and disclosure, takes revisioned snapshots, and subscribes from owner cursors.
   Unavailable peers remain stable broken/degraded links rather than local clones.
3. The user observes and controls work through owner guards. Another read-only
   client may attach concurrently; exclusive control, if needed, uses an explicit
   owner lease/takeover rather than terminal focus.
4. The user detaches the TUI. Subscriptions close and presentation cursors/layout
   are saved, but the room, task, saga, agent turn, query and owner processes
   continue under their own policies. No leave, acknowledge, cancel or drain is
   inferred from terminal exit.
5. A later TUI authenticates afresh using the stable journey/session references,
   observes current incarnations and policy, resumes from retained cursors or
   resnapshots across gaps, and reconciles any operations that were in flight at
   disconnect.
6. If the provider binding—not merely the UI—was lost, Daimon exposes the exact
   reattach-required state and a separate guarded provider reattachment. It never
   replays the prompt history or silently creates a new agent session.

### J26. Run an agentic analysis entirely headlessly

1. Automation uses the CLI to discover compatible existing data/query, Dendrite,
   Moira and Daimon targets and receives stable references plus exact capability
   profiles. It does not open TUI state or create resources by attaching.
2. A plan command returns the sources, graph/query dialects, SME fold/surfaces,
   authority, budgets, placement and expected effects. Where policy requires, a
   separate actor returns the exact challenge fingerprint or approval receipt.
3. Execute admits one durable journey/run and immediately returns its reference.
   The invoking shell may detach while owner processes continue.
4. A later `inspect` or `watch` attaches from a cursor and emits bounded framed
   events. Disconnect preserves the cursor and exact operation identity; it does
   not cancel the model, query, workflow or event delivery.
5. Permission waits, uncertain effects, partial federated query results and SME
   changes appear as typed states. Follow-up commands bind exact current
   revisions; automation cannot answer an unspecified confirmation interactively.
6. Final headless output links query/source lineage, graph results, SME influence,
   agent/tool evidence, artifacts and owner receipts. A separate detach ends the
   client; close/cancel/archive remain explicit if desired.

### J27. Hand a live session between CLI, TUI, and another operator

1. A headless client starts or locates a durable agent/workflow/analysis session
   and emits a content-free handoff reference containing no bearer authority.
2. A TUI uses the reference to authenticate, attach read-only, resnapshot and
   render the same owner state. It requests control only if an action is needed.
3. If exclusive control applies, the current controller approves/releases or the
   owner performs a policy-authorized takeover, fencing the old controller epoch.
4. The TUI supplies human steering or permission decisions through exact owner
   challenges, then detaches without stopping the work.
5. Another operator or CLI attaches under its own identity and authority. The
   audit trail distinguishes viewers, controllers, approvers and domain actors;
   presentation handoff is never reported as task assignment or effect approval.

### J28. Investigate an operational signal across events, logs, and metrics

1. A metric rule detects elevated journey latency/error-budget burn and creates
   a versioned alert with evaluation window, population, freshness and rule
   identity. The alert opens or links to an incident only through explicit intake.
2. An operator attaches by TUI or headless CLI to the incident and relevant
   telemetry sources. The initial view shows affected/unaffected populations,
   recent changes, topology, signal gaps, sampling, ingestion lag and cardinality
   posture rather than declaring a root cause.
3. The operator pivots from the metric's authorized exemplars to traces, then to
   structured logs and exact owner-domain/Hermes events. Source/observed time,
   trace correlation, event causation and durable owner receipts remain distinct.
4. Dendrite routes observability, SRE, distributed-systems and relevant domain
   SMEs. Humans and agents form competing hypotheses and query evidence for and
   against them; “no matching logs” is recorded as an incomplete observation.
5. A suspected deployment/configuration change is compared against baseline.
   The accountable operator executes containment or rollback through the owning
   workflow and authority—not through the telemetry backend.
6. Recovery requires user-visible probes and owner-native semantic evidence in
   addition to improving metrics. Alert resolution records only that its rule no
   longer fires; the incident owner separately declares impact ended.
7. The incident artifact retains exact event/log/metric/trace queries, windows,
   selected evidence, gaps, decisions and receipts. Retrospection creates owned
   fixes to instrumentation, alerts, tests, architecture, runbooks or product.
8. Clients detach without acknowledging events, alerts or incidents. A later
   client resumes from saved signal cursors and current owner state.

### J29. Design and operate a trustworthy telemetry contract

1. The telemetry steward and domain owner define the user/system question,
   signal kind, population, semantic owner, schema/instrument, type/unit,
   temporality, attributes, privacy class, retention and expected decisions.
2. They choose domain/integration event, audit record, structured log, metric,
   trace or profile deliberately. A domain transition is not reduced to a log;
   high-volume occurrences may be aggregated into metrics with declared loss.
3. Security/privacy and observability SMEs review content, label cardinality,
   identifiers, redaction, sampling, exemplars, cost and cross-domain disclosure.
4. Instrumentation is tested for schema/type/unit stability, counter resets,
   histogram aggregation, timestamps, trace linkage, overload/drop behavior and
   the invariant that telemetry sink failure cannot change domain results.
5. Versioned dashboards, recording rules, SLIs/SLOs and alerts state their source
   dependencies, windows, exclusions, missing-data behavior and owners. Synthetic
   fixtures cover nominal, degraded, silent-pipeline and false-positive cases.
6. After rollout, the steward monitors pipeline health, usefulness, cost,
   cardinality, noise and decision outcomes. Schema evolution and retirement
   preserve compatibility or announce a deliberate breaking profile.
7. Periodic review removes unused sensitive/noisy telemetry, updates thresholds
   from evidence, and verifies that operators can complete representative TUI
   and headless investigations within declared time and access constraints.

### J30. Let an agent improve Pantheon and observe the change in flight

Status: architectural target; not implementation-ready until the four owner
classes in the reflexive-development implementability gate are selected and
qualified for the deployment profile.

Prerequisite gate: the profile resolves the exact build, deployment,
observation and recovery owners and verifies their current identities,
authority, compatibility, readiness and failure independence. Missing,
placeholder, stale or candidate-controlled ownership refuses the journey before
source mutation.

1. A human or authorized automation admits a Pantheon improvement with an
   accountable owner, affected users/components, invariants, risk class,
   acceptance criteria, maximum exposure and explicit rollback conditions.
2. A Daimon agent attaches to the task room, declares an engineering role, and
   uses an isolated source worktree. It may transition through ideation,
   architecture, security, prototyping and testing; each transition records its
   objective and re-evaluated delegation without widening authority implicitly.
3. Dendrite routes a context-specific, concern-covered stakeholder-SME cohort;
   Moira frames productively adversarial reviews at intent, design and
   promotion boundaries. Independent reports challenge user value, affected
   parties, architecture, security/privacy, engineering, operations,
   observability, support, accessibility, cost and recovery as applicable.
   Missing lenses, grounding digests, evidence, dissent and dispositions remain
   visible; neither agreement nor review count authorizes the change.
4. The agent produces source changes, tests, compatibility/migration analysis,
   threat model, rollout plan and recovery plan. The qualified build owner creates
   an immutable candidate with dependency inventory and provenance.
5. Independent gates bind that digest to the exact target/cohort and policy
   epoch. Material ensemble findings have explicit owner dispositions and
   unresolved risks accompany the decision. The authoring agent may address
   findings but cannot approve its own deployment, change Aegis to authorize it,
   or weaken a failed test/monitor.
6. The qualified deployment owner exercises the candidate in sandbox and effect-free
   shadow operation, then admits a bounded canary. For a local single-instance
   component this may require a drained process replacement; for a replicated
   component it may be a rolling instance or route change. The qualified
   deployment owner, not Pantheon presentation, defines the mechanism.
7. Humans and the agent attach through Agora plus TUI/CLI/headless observation
   to compare baseline and canary domain probes, events, logs, metrics, traces,
   resource use and user feedback supplied through the qualified observation
   owner. Independent safety and telemetry paths remain available if the
   candidate crashes or lies.
8. The qualified observer submits evidence but cannot decide. Intermediate
   hold, abort, rollback or bounded exposure reduction may follow a separately
   qualified policy, but final promotion to the authoritative/default production
   population is always presented to a named human rollout principal for an
   explicit go/no-go. Any admitted human-advocate gate request also pauses its
   exact material decision for named-human adjudication. Aegis authorizes the
   current human principal and transition; the qualified decision-record owner
   persists presentation and response; and the deployment owner executes it.
   The recovery-policy owner separately controls last-known-good designation.
   Silence, timeout, stale response, outcome-unknown, lost authority,
   observation gaps or failed recovery readiness cannot produce promotion.
9. The journey settles exact code/configuration and residual state/effects,
   records user communication, and enters retrospective/insight mode. Validated
   lessons update tests, runbooks, SMEs or new work rather than rewriting the
   evidence that supported the original decision.

### J31. Hand one journey across human and agent roles

1. A human requester frames an ambiguous opportunity in a room. An ideation
   agent creates alternatives, an analyst agent tests available evidence, and a
   human stakeholder challenges assumptions; all contributions retain author
   type and role.
2. The same agent session may propose a strategy and architecture, but role
   transitions are visible and do not convert a proposal into a decision. A
   human or separately authorized decision owner selects the direction.
3. Engineering and test agents implement and evaluate the selected scope while
   humans attach, steer, pause, edit, review or take over. Headless automation
   sees the same owner states and receipts as interactive clients.
4. A release agent assembles the package and requests deployment; separation of
   duties selects a different approver/operator where policy requires it.
5. During operations, an incident agent diagnoses within its runbook and hands
   control to a human before an out-of-bounds action. After recovery, human and
   agent participants conduct a shared retrospective and turn supported findings
   into improvements.
6. The journey view shows continuous intent/evidence across roles without
   merging identities, inventing human approval, or treating an agent's earlier
   authority as valid for its later role.

### J32. Abort or recover a failed reflexive rollout

1. A canary violates an invariant, user-visible probe, security boundary,
   compatibility promise or resource bound. An independent monitor stops new
   exposure and records the exact candidate, cohort, trigger and evidence.
2. The qualified deployment owner drains or fences the candidate. A named
   recovery-policy administrator obtains exact Aegis authorization; the
   recovery-policy owner persists the selected current manifest/last-known-good
   revision; and the qualified recovery executor applies it and receipts the
   transition. Stale candidate instances, leases and controllers cannot resume
   or publish as current.
3. Code/configuration rollback, state restoration, message/effect
   reconciliation and user remediation are tracked separately. Irreversible
   external effects receive compensation or explicit residue; they are never
   described as undone merely because old code is running.
4. If rollback or observation is uncertain, the journey becomes an incident,
   preserves both conflicting evidence and limits further automation. A human
   can use the independent operator path even if Pantheon presentation or the
   changed component is unavailable.
5. Recovery is declared only after owner-native invariants and user-visible
   probes pass and topology/state are reconciled. The agent can analyze the
   failure and propose a repair, but cannot erase the failed candidate's audit
   history or self-approve a second attempt.

### J33. Review any work mode through a stakeholder-SME ensemble

1. A material work product or transition requests review with its exact
   revision, purpose, current/custom work mode, decision/effect boundary,
   affected populations, stakeholder map, constraints, evidence, uncertainty,
   reversibility and risk.
2. Moira admits a bounded plan under the applicable Rune-owned review method and
   claim/evidence/dissent schema. Dendrite folds the context into exact
   stakeholder and technical SME surfaces, reports concern coverage and tests
   proposed lenses for pairwise orthogonality. Before substantive execution the
   plan retains the answer to “Do we have the right ensemble?” as `yes`, `no` or
   `contested`, with affected-population, stakeholder-map, equity/coverage,
   missing/excluded and assessor evidence. The owner sees missing, unavailable,
   conflicted, overrepresented and excluded concerns; `no` refuses substantive
   disposition and `contested` requires explicit resolution or human escalation.
3. Human stakeholders and independently admitted agent reviewers receive only
   authorized material. Each performs a pinned first pass under an attributed
   stakeholder representation and optional persona lens before cross-review where feasible. Agent-modeled
   stakeholder perspectives and agent-produced technical expertise are labeled
   separately; neither can claim lived experience, consent or mandate.
4. The ensemble steelmans the goal, challenges the proposed work and status quo,
   seeks disconfirming evidence, compares alternatives, identifies strengths and
   failure modes, and proposes tests or mitigations. Unsupported assertions,
   correlated agreement and duplicate objections remain visible.
5. Moira retains each report, evidence, rebuttal, dissent, candidate and bounded
   non-convergence. Pareto analysis may remove dominated alternatives but cannot
   select a result; vote count and model agreement cannot establish truth,
   stakeholder acceptance, risk acceptance or authority.
6. The accountable work owner dispositions material findings as accepted,
   mitigated, tested, deferred, risk-accepted, disputed or out of scope, with
   rationale, owner and follow-up. Required policy/approval is evaluated
   separately by Aegis and the semantic owner.
7. A supported, non-frivolous finding from the declared human-advocate lens may
   request human gating for the exact material decision. Moira retains the gate
   request and pauses progression until a named human receives the decision,
   alternatives, evidence, dissent, uncertainty and consequences and records
   go, no-go, revise or defer under current authority. The advocate does not
   acquire veto/effect authority, but the owner cannot evade presentation by
   marking the request out of scope or substituting an agent response.
8. A material change to purpose, scope, evidence, affected population, risk,
   architecture or deployment invalidates the affected portion of review and
   triggers a new routed cohort. Otherwise review may be reused by exact digest
   and declared applicability rather than repeated performatively.
9. Tests, rollout observations, user feedback and incidents link back to the
   ensemble's predictions. Retrospection evaluates concern utility and blind
   spots, updates work or Dendrite corpus through its governed lifecycle, and
   never rewards objection volume or retrospective certainty.

### J34. Register, evolve, deprecate and retire a custom work mode

1. An authorized mode author proposes a stable namespace/name and semantic
   owner plus a versioned declarative contract: purpose, roles, inputs, outputs,
   lifecycle/transitions, invariants, evidence and uncertainty terms,
   authority/effect boundaries, review triggers, recovery, retention/disclosure,
   accessibility and interactive/headless projections. The registration payload
   contains no executable plugin, bearer authority or hidden network route.
2. Pantheon validates bounded encoding, identifier ownership, referential
   integrity, closed transition/outcome shapes, schema compatibility, projection
   completeness and resource limits. Aegis and each referenced owner evaluate
   who may register, instantiate, transition, migrate, deprecate or retire the
   mode; registration itself grants none of those actions.
3. Dendrite routes registry, domain, security, human-factors, operations and
   affected-stakeholder lenses; Moira retains the productively adversarial review
   of semantics, missing states, unsafe transitions, evidence nonclaims and
   lifecycle consequences. Material findings require owner disposition before
   admission, but review agreement is not authority.
4. The selected registry owner admits the exact digest and returns an immutable
   registration/version receipt. Pantheon catalogues the mode and generates or
   binds its navigation, forms, status vocabulary, action previews and versioned
   TUI/CLI/machine projections without becoming semantic owner.
5. Authorized humans and agents instantiate the same version under explicit
   roles and delegations. Each instance pins the registration digest; unknown or
   unavailable versions refuse rather than falling back to a similarly named
   mode. Owner-native state and receipts remain authoritative.
6. A compatible revision is proposed and reviewed with a machine-checkable
   compatibility declaration. Existing pinned instances continue unchanged;
   new instances use the new version only after activation. Optional migration
   is explicit, revision-bound, dry-run where possible, separately authorized
   and durably receipted.
7. A breaking revision receives a new major identity and may run side by side.
   No registry alias reinterprets old instances, transitions, evidence or links.
   Federated sites negotiate supported versions and expose partial availability
   instead of silently downgrading.
8. Deprecation blocks or warns on new instantiation according to policy while
   preserving discovery, observation, completion/recovery and export for pinned
   instances. Owners receive a migration/retirement horizon; exceptions are
   explicit and time-bounded.
9. Retirement occurs only after the selected disposition for every retained
   instance—completed, migrated, cancelled, archived, exported or explicitly
   stranded with accountable residue. Tombstone metadata preserves identity,
   schema, provenance, audit interpretation and broken-link explanation; it
   carries no executable code or reusable authority.
10. Rollback of catalogue activation may stop new use of a bad revision but
    cannot erase its receipt, reinterpret instances already created, reverse
    owner effects or resurrect a retired version without a new authorized
    lifecycle decision.

### J35. Run a monthly or quarterly executive business review

1. An authorized owner instantiates the exact `BusinessReviewV1` definition for
   a bounded period, scope, evidence cutoff and named Executive Sponsor. Monthly
   cadence emphasizes operating exceptions; quarterly cadence additionally
   reconsiders strategy, investment, scenarios and continuation.
2. Owner-native projections supply objectives, outcomes, financial and operating
   evidence, customer/stakeholder impact, commitments, dependencies, incidents,
   risks and explicit gaps. The review retains source, revision, population,
   freshness and uncertainty rather than copying a parallel status truth.
3. Daimon may prepare analysis and narrative; Dendrite routes business, domain,
   human-impact, financial, security, operational and public-interest lenses;
   Moira retains independent reports, dissent and human-gate requests. Agent
   authorship remains visible and cannot impersonate the Executive Sponsor.
4. Pantheon presents an executive decision instrument: material changes,
   expected-versus-observed outcomes, intervention points, unresolved dissent,
   risks/opportunities and a bounded decision agenda. Red, unknown, no-go and
   stop-this-work are normal outcomes, not presentation failures.
5. Each decision packet pins alternatives, recommendation, evidence,
   uncertainty, affected people, financial/operational consequences,
   reversibility and deadline. The named authorized human records go, no-go,
   revise or defer; any admitted human-advocate gate is resolved separately.
6. Accepted next-period commitments create owner-native work and funding/effect
   requests through explicit commands and receipts. Publishing the review does
   not itself admit a task, move money, authorize an effect or prove an outcome.
7. The next review links observed outcomes and changed assumptions to the prior
   decisions without rewriting their evidence or treating sponsor confidence as
   metric truth.

### J36. Propose, sponsor and govern a project through fresh stage decisions

1. A proposal owner instantiates `ProjectProposalV1` with a named Executive
   Sponsor, problem/opportunity evidence, affected stakeholders, intended
   outcomes, baseline, success/failure/stop criteria and decision horizon.
2. The proposal compares status quo, minimum intervention, recommendation and
   credible alternatives across value, lifecycle cost, capacity, risk,
   reversibility and time. Scope, non-goals, architecture, dependencies and
   semantic owners remain explicit.
3. Dendrite and Moira run the selected stakeholder-SME ensemble. Human-advocate
   findings can require named-human adjudication; dissent, missing evidence and
   unresolved risks accompany the proposal rather than disappearing into an
   executive summary.
4. The Executive Sponsor makes a revision-bound funding/sponsorship decision
   with exposure ceiling, conditions, delegated authority, evidence required at
   the next gate and an explicit go, no-go, revise or defer response. Sponsorship
   creates no peer effect by itself.
5. Discovery, build and pilot proceed only through their owner-native work and
   authority contracts. Each stage publishes actual evidence and receives a
   fresh decision; initial sponsorship is never replayed as technical acceptance
   or rollout approval.
6. Final production rollout is always presented to a named human for an explicit
   go/no-go under the exact current proposal, candidate, observation, affected
   population and recovery posture. Silence, timeout, prior funding or automated
   policy cannot promote.
7. Benefits-realization review compares observed outcomes with the promised
   baseline and costs, then explicitly continues, revises, stops or closes the
   project. Closing work does not erase residual effects, dissent or remediation.

## Necessary cross-cutting functionality

### Discovery and navigation

- Typed global search with owner, source revision, completeness, authorization,
  and retention-gap metadata.
- Stable deep links to exact owner resources and a journey link graph.
- Saved user-scoped views, recent history, favorites, and role-appropriate defaults.
- Keyboard-first command palette and discoverable unavailable actions.
- Per-owner and aggregate help generated from negotiated capabilities.

### Attention and collaboration

- One inbox of references to owner-native mentions, permissions, uncertain
  effects, failed/degraded owners, review requests, stale grounding, expiring
  leases/grants, and consumer lag.
- Deduplication/grouping by stable operation or journey, with source and age.
- Acknowledge/snooze/assign are inbox projection state unless the owner exposes a
  corresponding semantic command.
- Task-linked rooms, addressed human/agent messages, explicit prompt admission,
  review annotations, and handoff receipts.
- Review inboxes expose requested lens/mandate, pinned target, due/risk context,
  missing coverage, independent first-pass state, material findings, owner
  disposition and re-review triggers without leaking another reviewer's private
  draft or turning acknowledgement into agreement.
- Human-gate items are visually distinct from ordinary review: they show why
  presentation is required, the exact decision revision and alternatives,
  affected people, evidence/dissent, consequence, reversibility, authority and
  deadline. Headless clients may prepare or observe them but cannot manufacture
  the named human's response.

### Control safety

- Owner-native action preview and confirmation text.
- Current revision/incarnation/epoch revalidation immediately before command.
- Stable idempotency/operation identity generated before dispatch and persisted
  when cross-process recovery is promised.
- `pre_dispatch`, `dispatched`, `completed`, `failed`, `unknown`, and `conflict`
  distinctions where applicable.
- No generic retry button for uncertain effects; display supported recovery only.
- Blast-radius preview for batch, revocation, compaction, cancellation,
  compensation, and restart actions.
- Final production rollout never offers an automation-only confirm path. It
  requires a fresh named-human presentation and explicit go/no-go; other admitted
  human-advocate gates use the same non-bypassable presentation contract.

### Provenance and audit

- Causation/correlation separate from identity/idempotency/authority.
- W3C PROV-shaped entity/activity/agent relations at export boundaries.
- Immutable digests for exact configuration, grounding, model/profile, tool set,
  workflow definition, and artifact inputs where the owner supports them.
- Gap, truncation, redaction, retention, and unverifiable states are first-class.
- Content-safe telemetry; sensitive content remains in authorized owner surfaces.

### Reliability and operations

- Readiness, liveness, freshness, degradation, saturation, and semantic certainty
  displayed separately.
- Golden signals—latency, traffic, errors, saturation—from the
  [Google SRE model](https://sre.google/sre-book/monitoring-distributed-systems/),
  plus RED/USE populations and tail latency.
- Per-boundary logical-call versus physical-attempt traces, using W3C Trace
  Context and OpenTelemetry conventions without authority in baggage.
- Bounded feeds with backpressure, gap recovery, and load shedding that never
  changes domain outcomes.
- Backup/restore/migration/rollback procedures stated per owner and tested against
  namespace/epoch/incarnation rollback hazards.

### Security and privacy

- Threat models using STRIDE for security and LINDDUN for privacy at every new
  cross-peer link and content projection.
- Least privilege, default deny, non-amplifying delegation, short-lived grants,
  explicit revocation, separation of duties, and current-context checks.
- Prompt-injection boundaries between untrusted room/document/event content and
  tool/action instructions.
- Tool sandbox, filesystem root, command registry, environment allowlist, network
  egress, secret injection, output bounds, child-process cleanup, and provenance.
- Data classification, minimization, purpose limitation, retention, redaction,
  disclosure audit, and subject-access/deletion behavior by owner.
- Supply-chain attestations and exact dependency/provider/plugin provenance.

### Accessibility and human factors

- WCAG 2.2 AA-equivalent behavior for GUI/web projections and the corresponding
  keyboard/focus/status semantics in TUI.
- No action depends only on color, animation, hover, pointer precision, or timed
  input; status changes are announced without stealing focus.
- Clear separation of selection, focus, checked/marked state, pending command,
  authorization, and confirmed effect.
- Progressive disclosure: summary → exact evidence → raw owner record where
  authorized.
- Destructive/irreversible actions name the target, scope, consequence, recovery,
  and current witness; frequent safe actions remain low-friction.

### Interoperability

- Native peer protocols remain canonical; MCP/ACP/HTTP/CloudEvents/iCalendar/
  ActivityStreams/LSP are adapters with explicit conformance profiles.
- Capability negotiation is versioned and fail-closed. Unknown optional fields may
  be ignored only where the protocol explicitly permits it.
- Canonical bounded encoding, exhaustive decode rejection, immutable fixtures,
  protocol downgrade policy, and compatible migration are qualification gates.
- Standalone, embedded, attached, hosted, interactive, and headless paths preserve
  semantic parity while making topology differences visible.

## Prior-art capability comparison

| Precedent | Strong pattern to adopt | Boundary Pantheon must improve/preserve |
| --- | --- | --- |
| VS Code | Workbench layout, command palette, activity navigation, context panels, restoration | Extensions/terminal/file operations often share ambient user authority; Pantheon needs owner-scoped capabilities |
| Backstage | Entity catalog, ownership, plugin-organized tools, broken entity visibility | Catalog metadata is not runtime readiness or peer authority |
| Linear/GitHub Issues | Fast intake, hierarchy, dependencies, saved views, projects/cycles, discussion | Comments/labels/commit links must not infer execution or completion |
| Slack/Matrix | Rooms/channels, threads, mentions, membership, moderation, receipts, sync | Conversation must remain separate from work admission, agent execution, and authority |
| Temporal/Step Functions | Durable histories, retries, timers, activities, signals, compensation patterns | External non-idempotent effects require explicit uncertainty and owner-specific recovery |
| GitHub Copilot/Codex/Cursor | Issue-to-agent, isolated workspace, live session, steering, review, takeover, automation | Pantheon should expose exact authority, budgets, grounding, effect receipts, and cross-peer recovery rather than a single opaque run status |
| Kafka/CloudEvents | Stream/consumer identity, offsets, replay, schemas, common event context | Acknowledgement and exactly-once claims stop at their cooperating boundary |
| OpenTelemetry/Prometheus/OpenMetrics | Correlated traces/logs/metrics, resource and instrumentation identity, metric types/temporality, exemplars and established exposition/query practice | Signal correlation remains non-authoritative; preserve sampling, aggregation, resets, gaps, cardinality and exact supported profile |
| SQL/GQL/SPARQL/Jupyter/Arrow | Standard query/data models, computational narratives and typed analytical interchange | Pantheon negotiates exact profiles and preserves source snapshots, bounds, lineage and authority; it does not invent one universal schema |
| tmux/Jupyter sessions | Persistent server-side sessions/computation with multiple later clients and separate attach/detach/interrupt/delete actions | Pantheon generalizes the lifecycle pattern across sovereign owners with fresh authorization, revision/cursor recovery, receipts and uncertainty |
| Dendrite/Rune SME surfaces and folding | Versioned ambient/activated/reference expertise, deterministic context-to-role routing, orthogonality and corpus provenance | Pantheon/Daimon/Moira own timely consultation and influence evidence; Dendrite advice does not become workflow, effect or decision authority |
| Erlang/OTP releases and Kubernetes Deployments | Explicit upgrade/downgrade instructions, synchronized state change or restart; rolling replacement, readiness, progress, pause and rollback | Pantheon selects and qualifies a mechanism per component/state topology; “hot” cannot conceal incompatible state, mixed-version risk, uncertain effects or failed recovery |
| Simplex runtime assurance | Advanced behavior is bounded by a separately trusted monitor/controller and recoverable safe path | The agent and candidate cannot replace or authorize the safety envelope observing them; autonomy expands only inside verified limits |
| TUF and SLSA | Separated/threshold update trust, freshness and rollback defenses; immutable build provenance | Provenance and signatures inform admission but do not prove behavior; keep ultimate trust and known-good recovery outside online agent credentials |
| SPIFFE/SPIRE | Short-lived workload identity, explicit trust domains and directional federation relationships | Authentication establishes peer identity; Aegis and each owner still decide resource/action authority and disclosure |
| Raft/SWIM/CRDT research | Consensus ordering, scalable membership/suspicion, and formally convergent replicated data types | Select the mechanism per logical authority; membership is not leadership and eventual delivery does not make arbitrary state mergeable |
| Zanzibar/Macaroons/OAuth RAR | Relationship/context authorization, attenuation, structured fine-grained requests | Aegis grants remain opaque and owner decisions are revalidated; UI confirmation is not authority |
| W3C PROV/in-toto/SLSA | Entity/activity/agent lineage, planned steps, materials, subjects, attestations | Provenance supports verification but does not make generated claims true or actions authorized |
| LSP/Tree-sitter/editor research | Editor-language separation, incremental analysis, versioned edits, semantic navigation | Analysis is derived and stale-able; coordinates, authority, persistence, and collaboration remain Vellum-owned |

## Capability priorities

### P0 — preserve truth and expose relationships

1. Define the typed relationship projection and stable journey reference.
2. Select the persistence owner and implement the versioned role-activation and
   transition contract: human/agent principal type, role, work mode, objective,
   model/tool profile, resource/action scope, delegation, budgets, grounding,
   accountable owner, policy epoch, start/expiry, rationale and prior activation.
   Project it consistently in UI, TUI and headless views/actions; role display
   never grants authority and historical activations remain auditable.
3. Add owner/freshness/revision/uncertainty labels everywhere aggregate status is shown.
4. Compose Moira's peer-qualified agent participation, session, event, prompt,
   permission, cancellation, reattach, and deliberation surfaces.
5. Build the unified attention view as references, not copied semantic state.
6. Provide owner-native “next safe action” and action preview contracts.
7. Define the lifetime taxonomy and machine contracts for attach, detach,
   disconnect, reattach, provider/kernel reattach, cancel, close and shutdown.

### P1 — close the cornerstone journey

1. Version a task-to-agent workflow template with task/room/session/artifact links.
2. Add explicit Agora-message-to-agent-prompt admission.
3. Show Daimon event/budget/tool state and Aegis permission requests in the journey.
4. Add Vellum proposal/diff/review and durable receipt links.
5. Qualify lost replies at every cross-peer boundary and complete-replay with no
   duplicate provider call, edit, task, or message.
6. Add authorized existing-resource/session catalogues, dynamic TUI/CLI attach,
   clean presentation detach, cursor continuation and CLI↔TUI handoff without
   creating, cancelling or draining owner work.
7. Select a bounded work-mode registry owner and qualify J34's register,
   activate, version, migrate, deprecate, retire and tombstone lifecycle with
   human-agent and UI/TUI/headless parity before custom modes are advertised.

### P2 — operational and epistemic depth

1. Add grounding detail, diff, claim/evidence/dissent inspection.
2. Add saga graph, evidence lattice, compensation, and reconciliation UX.
3. Add Hermes lag/gap/retention/schema views.
4. Add global search with bounded owner query contracts.
5. Add support/audit export with explicit gaps and nonclaims.
6. Add the analysis/query/graph workbench with catalogue, plan preview, bounded
   execution, typed results, visualization, reproducible artifacts and lineage.
7. Compose Dendrite's peer-qualified query and three-depth SME surfaces;
   implement profile-owned ambient loading, fold activation, coverage and
   influence inspection while preserving the narrative-search gap.
8. Bring those analysis and SME journeys to headless parity with plan/execute,
   durable run references, attach/watch/detach, challenges and recovery lookup.
9. Add event/log/metric/trace catalogues, query/live-follow, signal pivots,
   dashboards, SLO/alert inspection, telemetry-pipeline health and saved
   investigation artifacts with TUI/headless parity.
10. Compose Moira methodical deliberation with Dendrite fold/surface/lattice
    evidence into a productively adversarial stakeholder-SME review: contextual
    cohort planning, independent reports, coverage/orthogonality, dissent,
    finding disposition, human-advocate gate requests, named-human presentation/
    response, review invalidation and outcome feedback with headless parity that
    cannot synthesize the human decision.

### P3 — product profiles

1. Chatbot, workflow, and tool-using agent profiles with distinct invariants.
2. Recurring maintenance and event-triggered workflow profiles.
3. Human takeover/handback and cross-device session continuation.
4. Break-glass, separation-of-duties, and regulated review profiles.
5. Selected VCS/worktree/PR owner rather than ambient Git in Pantheon.
6. Nominal operations and incident-observability profiles with explicit signal,
   alert, incident, recovery and retrospective boundaries.
7. After the build, deployment, observation and recovery owners pass the
   implementability gate, qualify a reflexive-development profile: isolated
   worktree, role transitions, reproducible candidate/provenance, independent
   review, sandbox/shadow/canary, guarded component replacement, attachable live
   comparison, rollback and retrospective. Begin with low-risk local changes;
   qualify stateful and authority/control-plane targets separately.

### P4 — federated deployment profiles

1. Establish global authority/resource/instance/incarnation namespaces and a
   signed, versioned, expiring authority directory.
2. Select authenticated remote transport, workload identity, trust-domain
   federation, discovery, negotiation and route-pinning profiles.
3. Compose the existing Agora and Vellum single-home/replica precedents and
   Chronos federation admission into Pantheon's typed journey projection.
4. Qualify remote Moira peer effects, Daimon worker placement/reattachment,
   Hermes relay/replication and Aegis cross-domain revalidation under partitions.
5. Add distributed Operations views and guarded instance replacement, logical
   failover, rehome, rolling upgrade and disaster-recovery workflows.
6. Add authenticated remote TUI/CLI attachment, multi-client observation/control,
   resumable event cursors and service-manager-backed shell detachment.
7. Run cross-host, cross-site and cross-trust-domain fault campaigns covering
   lost replies, partitions, stale routes, split-brain attempts, clock skew,
   version skew, revocation delay, replica gaps and capacity exhaustion.
8. Qualify reflexive rolling upgrades across federation: mixed-version windows,
   cohort/route identity, quorum and home constraints, independent observation,
   trust-root freshness, rollback ordering and site-loss recovery.

### P5 — broader deployment and ecosystem profiles

Fleet-scale automation, end-to-end room encryption, additional remote providers,
signed distribution, more platforms, organization-wide policy, multi-region
active-active profiles and third-party federation should ship only with selected
identity, consensus/convergence, credential, trust, discovery, data residency,
retention, support, availability, upgrade and incident-response contracts.

## Journey acceptance catalogue

Every aggregate journey should include at least these scenarios:

1. **Happy path:** all owners accept and every final receipt is independently observed.
2. **Stale witness:** resource changes between preview and confirmation; command refuses without retargeting.
3. **Denied authority:** Aegis/owner refuses; no downstream dispatch occurs.
4. **Pre-dispatch failure:** safe retry with the same identity is allowed by the owner contract.
5. **Lost accepted reply:** fresh client/process recovers the original receipt without a second effect.
6. **Outcome unknown:** automatic progress fences; unrelated later work is not sent.
7. **Conflicting evidence:** UI retains conflict and requires owner/operator resolution.
8. **Duplicate exact input:** matching replay returns the original result.
9. **Duplicate changed input:** same operation identity with changed semantics conflicts.
10. **Cancellation race:** completion winning the race remains completion; cancellation does not rewrite it.
11. **Revocation race:** current revalidation blocks future dispatch; already completed effects remain attributable.
12. **Retention gap:** client resnapshots/reconciles and never interprets a gap as no events.
13. **Capacity exhaustion:** admission refuses before partial mutation; existing retained identities still replay.
14. **Peer restart:** exact durable state reopens under a new transport/session identity.
15. **Aggregate restart:** reverse drain, residue, and replacement preserve per-peer receipts and uncertainty.
16. **Privacy/telemetry sink failure:** domain result is unchanged; diagnostics drop/saturate visibly.
17. **Accessibility:** complete journey is keyboard operable with visible focus and announced status.
18. **Adversarial content:** prompt injection in room/document/event cannot grant tools or escape content boundaries.
19. **Budget exhaustion:** execution stops/fences according to policy and retains partial evidence without success.
20. **Audit reconstruction:** exported manifest verifies all available links and explicitly marks missing/redacted/unverifiable evidence.
21. **Divergence without commitment:** brainstorming branches and recombines ideas without admitting tasks or granting effect authority.
22. **Promotion boundary:** moving a prototype or insight into delivery creates explicit requirements, owners, controls and acceptance evidence.
23. **Alternative preservation:** a strategy or architecture choice retains rejected options, dissent, assumptions and revisit triggers.
24. **Production-readiness refusal:** a successful demonstration lacking security, test, package or operational evidence cannot be presented as ready to deliver.
25. **Learning closure:** an incident improvement is evaluated against its intended outcome rather than considered effective when its task closes.
26. **Altitude consistency:** IC, team, portfolio and executive views derive from the same owner records and expose freshness, scope and transformation rather than disagreeing silently.
27. **Executive exception:** a material deviation produces a concise decision request with evidence and drill-down, not a dashboard that requires reconstructing the issue manually.
28. **External least privilege:** a stakeholder can complete the invited review or acceptance but cannot discover or operate unrelated internal resources.
29. **Disclosure lifecycle:** external access expiry and revocation are enforced, prior disclosures remain auditable, and no claim is made that received exports were recalled.
30. **Affected-person redress:** an affected person can receive notice, correct relevant facts, request human review and track an appeal without needing employee-level access.
31. **Remote semantic parity:** local and authenticated remote clients produce the same owner outcomes and refusals while exposing topology-specific latency, trust and uncertainty.
32. **Partial discovery:** one unavailable or malicious discovery source cannot invent authority; cached routes expire and retain provenance.
33. **Protocol skew:** instances with no safe common version/profile refuse without downgrade, mutation or redirect loop.
34. **Network partition:** each operation follows its declared refusal, stale-read, queue, quorum or convergence policy and clearly exposes incompleteness.
35. **Split-brain attempt:** stale home/leader/incarnation/lease epochs cannot accept or publish a second authoritative mutation.
36. **Replica gap and divergence:** a replica detects the condition, stops claiming freshness, and repairs only from an authenticated authority snapshot/frontier.
37. **Instance failover:** promotion follows the selected consensus/home-transfer law and recovers retained identities without duplicating accepted effects.
38. **Logical rehome:** stale clients preserve request identity through authenticated redirect/re-resolution; old and new homes cannot both mutate.
39. **Trust revocation during partition:** future cross-domain actions fail closed at the selected freshness bound while already accepted effects remain attributable.
40. **Federated aggregate:** search, inbox, portfolio, executive and audit projections name unreachable/redacted authorities and never present partial results as complete.
41. **Reproducible query:** pinned sources, schema/profile, query, parameters and environment reproduce the retained result or explain exact source/environment drift.
42. **Bounded analysis:** row/byte/time/fan-out/cost limits and cancellation stop work without presenting truncated output as complete.
43. **Federated partial result:** unreachable, unauthorized and redacted sources remain distinct and the analysis states whether its conclusion survives missing populations.
44. **Graph semantic safety:** node/edge types and direction remain visible; “no path” reports searched snapshot/visibility and a path is never relabeled as causation.
45. **Natural-language query review:** an agent-generated query cannot execute until selected sources, compiled plan, limits and access scope are inspectable under the chosen risk policy.
46. **Ambient SME activation:** a known context tag deterministically routes the pinned roles/surfaces and records exact fold/corpus identities and consultation evidence.
47. **SME change during work:** a changed grounding digest follows explicit pin, refresh or review policy; the new content never silently replaces consulted advice.
48. **Advisory boundary:** SME retrieval or agreement cannot authorize an effect, satisfy an approval, prove a claim or complete a task/workflow.
49. **Attach-existing:** TUI and CLI attach to the exact existing resource/session and cannot fall back to creating a replacement when resolution or authorization fails.
50. **Clean detach:** subscriptions and client-owned presentation resources end while task, room, saga, agent, document, query and external owner processes remain unchanged.
51. **Abrupt disconnect:** in-flight mutation remains unknown where appropriate, connection-local handles expire, and no cancel/acknowledge/detach receipt is invented.
52. **Cursor reattach:** a fresh client resumes from an exact frontier or detects a retention gap and resnapshots without omitting or duplicating semantic events.
53. **Concurrent clients:** multiple read-only observers coexist; exclusive control follows an owner lease/takeover contract and stale controllers are fenced.
54. **Presentation handoff:** CLI→TUI→CLI continuation carries stable resource/cursor references but no credentials, grants, confirmation state or implicit assignment.
55. **Headless parity:** every selected interactive capability produces equivalent owner requests, refusals, receipts and recovery states under its machine contract.
56. **Noninteractive confirmation:** exact challenge/response refuses stale target, revision, scope, consequence or expiry and never defaults an ambiguous choice.
57. **Detached long-running work:** the initiating shell exits and a fresh client later observes the same durable run/session without provider or effect duplication.
58. **External-owner exit:** exiting Pantheon never drains, stops or replaces independently operated attached peers.
59. **Hosted-owner handoff:** leaving Pantheon-hosted peers running succeeds only through a durable supervisor ownership transfer; failed handoff requires guarded drain or explicit residue.
60. **Headless content safety:** prompts, queries, documents and secrets stay out of argv, URLs, process listings, diagnostics and default machine output.
61. **Signal separation:** one occurrence represented as domain event, integration event, log, metric contribution, span and alert retains each signal's distinct identity and nonclaims.
62. **Clock and arrival skew:** event/source and observed/ingest timestamps remain visible; delayed or skewed records do not create a false causal order.
63. **Counter reset and gap:** rate calculation recognizes restart/reset, collection gaps and temporality rather than reporting a negative spike or continuous population.
64. **Histogram aggregation:** compatible distributions aggregate correctly; incompatible buckets/schemas or client summary quantiles refuse misleading combination.
65. **Cardinality bound:** unbounded user/resource/session/content values are rejected or transformed before metric labeling without losing required owner evidence.
66. **Telemetry-pipeline failure:** dropped, sampled, rejected or delayed signals are visible while the underlying domain result remains unchanged.
67. **Live-follow detach:** TUI/CLI log/event/metric watch detaches cleanly, preserves a supported cursor, and neither acknowledges delivery nor stops producers/collectors.
68. **Alert/incident distinction:** alert firing can request incident admission and resolution can update evidence, but neither automatically opens/closes the incident or proves impact/recovery.
69. **Missing-data honesty:** absent logs, traces or series and zero values remain distinct; incomplete federated sources cannot produce a falsely complete green view.
70. **Recovery verification:** an incident requires owner-native and user-visible verification; improved telemetry alone cannot settle uncertain effects or declare recovery.
71. **Role parity without impersonation:** human and agent participants can perform the selected work mode through equivalent owner contracts while identity, authorship, delegation and accountable owner remain distinct.
72. **No authority by role change:** an agent switching from analyst to engineer, operator or reviewer receives no new capability until current policy admits the new bounded delegation.
73. **No self-approval:** an agent-authored candidate cannot satisfy its own independent review, risk acceptance, deployment authorization or exception gate.
74. **Isolated reflexive work:** changing Pantheon occurs in a pinned worktree/sandbox; production source, state, credentials and data cannot leak in or be mutated outside admitted scope.
75. **Reproducible candidate:** the exact source, builder, dependencies, invocation, tests and immutable output digest are retained; a mutable branch/tag/path cannot be deployed as identity.
76. **Compatibility refusal:** absent or failed wire/state/event/policy/telemetry compatibility and downgrade evidence stops rollout before the affected traffic/state boundary.
77. **Effect-free shadow:** mirrored inputs cannot issue real external effects or expose disallowed production data; shadow differences retain baseline/candidate versions and coverage.
78. **Bounded canary:** exact cohort, exposure, duration, budgets, stop conditions and baseline are visible; cohort expansion is a new authorized transition, not automatic proof from quiet telemetry.
79. **Guarded hot replacement:** selected code-upgrade, drain/restart, rolling or route-switch mechanism preserves owner invariants and reports exact old/new version, state transfer and residue.
80. **In-flight continuity:** retained operations, sessions, leases, cursors and uncertain effects survive or reconcile under their owners; the replacement cannot silently duplicate, abandon or relabel them.
81. **Independent observation:** safety gates include a signal/probe path outside the candidate; candidate loss, spoofing or instrumentation regression cannot produce a false healthy promotion.
82. **Human intervention:** an authorized human can inspect, pause, abort, take over and invoke recovery through a path that does not depend on the candidate agent/component.
83. **Rollback honesty:** restoration separately reports code/configuration, durable state, messages, external effects and user impact; partial or uncertain recovery remains an incident.
84. **Trust-root protection:** changes to Aegis, deployment, audit, telemetry, supervisor or rollback paths cannot replace the independent authority/monitor/recovery root in the same unreviewed rollout.
85. **Reflexive learning:** a failed or successful rollout produces attributable retrospective evidence and proposed improvements without editing prior decisions, suppressing dissent or self-authorizing the next attempt.
86. **Open work-mode parity:** a newly registered/custom mode supplies the semantic contract and can be performed by a human or agent without being coerced into one of the illustrative modes or losing owner semantics.
87. **Contextual ensemble routing:** the same work under changed purpose, affected population, risk or lifecycle state produces a review plan reflecting the new context and exact Dendrite fold/surface identities.
88. **Coverage before multiplicity:** many same-concern reviewers cannot satisfy a missing required stakeholder/safety concern; missing, excluded and unavailable lenses remain explicit.
89. **Correlated reviewers:** shared model, provider, prompt, grounding, evidence or organization is disclosed and agreement is not counted as independent corroboration.
90. **Stakeholder non-impersonation:** an agent lens cannot claim lived experience, consent, mandate or acceptance for an affected person/group; authorized real stakeholders can challenge the framing and findings.
91. **Independent first pass:** where policy selects it, one reviewer cannot anchor another before initial report commitment; later cross-examination preserves original and revised positions.
92. **Productive dissent:** reviews retain strengths, counterevidence, alternatives, tests and mitigations alongside objections; non-convergence or disagreement cannot be flattened into consensus or automatic refusal.
93. **Finding disposition:** every material finding has an attributable status, rationale, owner and follow-up; review agreement or an accepted disposition cannot substitute for approval, authority or effect evidence.
94. **Material-change invalidation:** changes to pinned work, assumptions, evidence, affected stakeholders, risk or effect boundary invalidate the relevant review scope and refuse stale review reuse.
95. **Outcome feedback:** observed outcomes link to prior review predictions and update utility/coverage evidence without rewriting reports, grading dissent by popularity or automatically mutating Dendrite doctrine.
96. **Role-activation recovery:** after process/client restart, the exact current and prior human/agent role activations reopen with principal type, scope, delegation, budgets, grounding, policy epoch, rationale and expiry; no role or authority is reconstructed from display state.
97. **Role-projection parity:** UI, TUI and headless clients show and transition the same persisted activation revision and owner outcomes; a stale client cannot overwrite a newer role transition.
98. **Declarative mode registration:** executable content, bearer material, hidden routes, unbounded schema or an unauthorized namespace is refused before registry mutation; successful registration grants no instance/effect authority.
99. **Exact mode resolution:** an instance pins one admitted mode version/digest; unknown, retired, ambiguous or unavailable versions refuse without nearest-name fallback or silent downgrade.
100. **Compatible mode upgrade:** activating a compatible revision affects only declared new use; existing instances remain pinned and semantically unchanged unless an explicit authorized migration succeeds.
101. **Breaking mode revision:** an incompatible contract uses a distinct major identity and can coexist; old transitions, evidence, projections and links are never decoded under the new meaning.
102. **Mode migration uncertainty:** a lost migration reply recovers the owner receipt by stable identity; unknown outcome fences retry and neither registry version is presented as the instance's settled truth.
103. **Mode deprecation:** policy can block new instances while existing pinned work remains observable and can complete, recover, migrate or export under an explicit horizon.
104. **Mode retirement and tombstone:** retirement accounts for every retained instance and preserves non-executable identity/schema/provenance needed for audit and broken links without retaining authority or permitting new use.
105. **Catalogue rollback honesty:** disabling a bad mode revision stops selected future use but does not erase registration, reinterpret or delete created instances, reverse owner effects or resurrect an older contract implicitly.
106. **Reflexive owner gate:** J30 refuses before source mutation unless qualified build, deployment, observation and recovery owners are all resolved, ready, authorized and sufficiently independent of the candidate/authoring agent.
107. **Promotion separation:** observer evidence, accountable-principal request, Aegis authorization, accepted promotion-decision record and steward execution retain separate identities/receipts; none can synthesize another or promote alone.
108. **Recovery-policy separation:** changing the recovery manifest or last-known-good designation requires the named administrator, exact authorization and versioned policy receipt; the rescue executor cannot author policy and promotion cannot designate itself known-good.
109. **Assurance-label honesty:** a same-user trusted-development profile describes SQLite history, unwritable paths and process separation as application/convention safeguards only; hostile-write resistance is claimed only after OS-enforced identity/filesystem/credential/process boundaries are qualified.
110. **Build-platform eligibility:** candidate production refuses without an unexpired record of exact repository visibility, service plan, attestation availability/privacy, trusted workflow identity, runner, registry and verifier profile; an ineligible private/internal GitHub plan follows the qualified alternative rather than omitting provenance.
111. **Final rollout human go/no-go:** promotion to the authoritative/default production population refuses without an exact presentation receipt and explicit current response from a named human principal; silence, timeout, observer pass, ensemble agreement, policy default or agent response cannot promote.
112. **Human-advocate gate:** a supported, non-frivolous human-advocate finding pauses the exact material decision for named-human presentation; the owner cannot bypass it through `out_of_scope`, relabeling or automatic disposition, and the advocate gains no veto/effect authority.
113. **Changed human-gated decision:** a changed candidate, scope, evidence, affected population, consequence or decision revision invalidates the prior response and requires a fresh presentation rather than replaying consent.

## Metrics that do not corrupt semantics

Measure these by explicit population and owner; never use them as completion or
authorization evidence:

- intake-to-admission, ready-to-claim, claim-to-start, prompt-to-first-progress,
  permission wait, execution duration, review wait, settlement duration;
- work in progress, throughput, cycle time, work-item age, blocked age,
  uncertain-effect age, lease/grant expiry pressure;
- task/saga/session/document/event success, refusal, failure, uncertainty,
  conflict, cancellation, compensation, and recovery rates;
- provider/tool token/call/time/byte utilization and hard-budget violations;
- grounding freshness, source coverage, unsupported-claim rate, citation/
  attribution fidelity, contradiction and abstention rates;
- event lag, retention headroom, gap frequency, redelivery/deduplication,
  acknowledgement and compaction latency;
- permission request/deny/expiry/revocation and excessive-scope findings;
- peer readiness/freshness, restart/drain outcomes, residue, recovery time;
- authority/home/quorum availability, route age/churn/failure, instance
  membership/suspicion, failover/rehome duration, split-brain refusal;
- replica frontier/lag/gap/divergence/repair, cross-site queue depth and expiry,
  partition duration, trust-bundle/revocation freshness and protocol-version skew;
- query plan/queue/execution latency, rows/bytes/fan-out/cost, cancellation,
  truncation, cache/snapshot age, unavailable populations and reproducibility;
- SME route/consult/reference depth, context and corpus budget, unmatched tags,
  coverage/orthogonality gaps, adopted/rejected/material-change feedback and
  stale-digest review—never raw retrieval count as proof of value;
- ensemble intended/admitted/missing lens coverage, disclosed reviewer
  correlation, independent-first-pass completion, evidence-supported finding and
  duplicate-objection rates, material dissent/non-convergence, finding
  disposition/age/re-review and predicted-versus-observed concern outcomes—never
  agreement, objection volume or reviewer activity as a quality proxy;
- human-gate request/admission/refusal, time-to-presentation, named-human
  go/no-go/revise/defer outcome, stale-response invalidation and decision wait;
  never optimize for approval rate, fast consent or fewer escalations;
- catalogue/resolve/attach latency and refusal, attached-client/controller count,
  disconnect/detach causes, cursor lag/gaps/resnapshot, takeover/fencing,
  reattach/recovery duration and abandoned connection-local handles;
- headless/TUI capability parity, challenge expiry/staleness, stream
  backpressure/disconnect, handoff success and accidental cancel/close/drain
  invariants (which must remain zero);
- telemetry volume/cost, schema/type/unit/version skew, label cardinality,
  sampling/drop/reject/parse/redaction rates, timestamp skew, ingest/query lag,
  retention, trace/log/metric correlation coverage and exemplar validity;
- alert evaluation freshness, pending/firing/resolution/no-data, noise/duplicate/
  inhibition, acknowledgement and incident-conversion; SLI population coverage,
  error-budget burn and objective-window completeness;
- human/agent role activation and transition, delegation refusal/expiry,
  persistence/recovery/projection skew, stale-transition refusal, self-approval
  refusal, takeover/handback, intervention and escalation;
- work-mode registration/validation/refusal, version adoption and skew,
  instantiation, migration/recovery, deprecation horizon/exception, retained-
  instance disposition, retirement residue and tombstone/broken-link resolution;
- candidate build/test/provenance reproducibility, compatibility-gate result,
  sandbox/shadow/canary duration and exposure, baseline/candidate semantic delta,
  readiness/progress stop, promote/hold/rollback and recovery verification;
- build-platform eligibility freshness/refusal, promotion decision/authorization/
  execution correlation, recovery-policy revision/last-known-good designation,
  rescue execution and declared-versus-enforced assurance level;
- mixed-version population/window, state-transfer and drain duration, in-flight
  reconciliation, stale-instance fencing, rollback residue and independent
  monitor/known-good-path availability;
- accessibility task completion, focus/keyboard defects, and screen-reader
  announcements tested with representative assistive technology.

Safety invariants are not error budgets. A lower error rate obtained by dropping
requests, hiding unknowns, or excluding slow failures is invalid. Percentiles,
censoring, sampling, dropped observations, and population boundaries must be
visible.

## Product-definition decisions still required

The [reflexive ownership decision brief](reflexive-ownership-decision-brief.md)
provides recommended defaults for decisions 38, 39, 43, 44 and 48. They remain
open until accepted by an accountable decision owner and qualified with concrete
implementations; a recommendation is not a selected owner.

1. Which personas and role separations are primary in each deployment profile:
   individual developer/operator, local team, external collaboration, or
   regulated organization? The current qualified profile is single-user local.
2. Is the “ticket” always a Chronos task, or is an external issue tracker a
   first-class source with its own sovereign adapter/owner?
3. Who owns typed relationship durability and access control: a new bounded
   Pantheon projection store, Moira, or adapter-owned links? It must not become a
   hidden semantic authority.
4. What is the selected VCS/review owner for worktree, commit, PR, CI, merge, and
   deployment journeys?
5. Which Agora capabilities are product-critical in Pantheon beyond current room
   create/speak/read: threads, topics, search, revisions, reactions, moderation?
6. Which actions require human confirmation versus policy-only authorization,
   and which require separation of duties?
7. What retention and disclosure policy applies to room content, model output,
   prompts, tool I/O, grounding sources, audit manifests, and traces?
8. Which remote identity, transport, credential, secret, sandbox, network-egress,
   and data-residency profiles will be supported first, and in what sequence?
9. What workflow-definition format and migration policy is selected: bounded
   native algebra, Rune-authored machines, a BPMN subset, or adapters among them?
10. What evidence is required before an agent-produced artifact may be reviewed,
    saved, committed, merged, deployed, or communicated externally?
11. What is the canonical federation namespace for trust domains, logical
    authorities, shards, resources, instances and incarnations, and who operates
    its directory without becoming a semantic owner?
12. Which component/resource classes use single home, read replicas, consensus,
    sharding, convergent active-active data, worker pools or gateways?
13. Which remote transport, mutual-authentication, workload-identity, channel
    binding, protocol negotiation and credential-rotation profiles are selected?
14. How are home discovery, route freshness, redirect, rehome, shard rebalance
    and disaster recovery authorized and made safe from split brain?
15. What consistency, availability, queueing and user-visible behavior applies
    to every command and observation during host, site and trust-domain partition?
16. Which data classes may replicate or execute in which sites/jurisdictions,
    and what evidence supports residency, deletion and external disclosure?
17. Which per-component availability, durability, recovery-point,
    recovery-time, clock-quality and revocation-propagation objectives are
    promised, and against which correlated failure domains?
18. What cross-version compatibility window, rolling-upgrade order, downgrade
    policy and fleet conformance evidence are required for federation?
19. Who owns saved analysis/notebook artifacts, query execution, result
    retention and scheduled refresh, and which kernels/languages are supported?
20. Which tabular, event/time-series, text and graph sources are composed first;
    what exact SQL/GQL/PGQ/SPARQL/native profiles and interchange formats apply?
21. What is the authoritative schema for journey, provenance, dependency,
    authority and topology graphs, and which inferred edges may be persisted
    only after human or owner review?
22. Which Dendrite corpus/fold and SME depths are selected by each agent/workflow
    profile; which consultation points are mandatory, advisory or user-controlled?
23. Who governs SME authorship, review, promotion, supersession, stable
    recommendation IDs, utility feedback and incident correction across trust
    domains without turning Dendrite into a decision authority?
24. Who owns the authorized catalogue of existing resources/sessions and their
    current homes, capabilities, cursors and attachment modes without becoming
    owner of those resources?
25. Which resource/session kinds survive detach, for how long, under what quota,
    checkpoint and idle/expiry policy, and how are owners/users notified before
    irreversible retirement?
26. What versioned JSON/framed-stream schemas, exit/refusal codes, pagination,
    cursor, challenge/response and content-input conventions define headless
    parity?
27. Which component controls permit concurrent clients, which require a
    controller lease or explicit takeover, and how are stale controllers fenced?
28. What durable supervisor/service-manager accepts lifecycle ownership when a
    Pantheon shell detaches from peers it started, and what happens if handoff is
    refused or uncertain?
29. Which presentation state—layout, filters, drafts, selections, cursors and
    worksets—is local, synchronizable or handoff-capable, with what privacy and
    stale-state rules?
30. Which currently interactive-only or one-shot controls move first to the
    common headless attach/watch/control/recovery contract, and what evidence
    establishes parity rather than merely similar output?
31. Which owners/backends provide domain-event, log, metric, trace, profile,
    alert and SLO catalog/query/follow contracts, and which signals remain local
    or are permitted to cross trust domains?
32. Which exact OpenTelemetry, Prometheus/OpenMetrics, CloudEvents and query
    language/profile versions are supported, with what translation loss and
    compatibility policy?
33. What content, identifier, attribute, cardinality, sampling, redaction,
    retention, residency and deletion policies apply per signal and persona?
34. Who owns dashboards, recording rules, SLIs/SLOs, alert rules, routing,
    inhibition, acknowledgement and their review/retirement lifecycle?
35. Which alert-to-incident mappings and recovery checks are automated, which
    require human admission/closure, and what prevents telemetry from becoming
    domain authority?
36. What source-time/observed-time and clock-quality model supports federated
    correlation without claiming a false total order, and how are gaps and late
    arrivals represented?
37. Which roles and actions are participation-symmetric for humans and agents in
    each deployment/risk profile, and which accountability decisions remain
    reserved for a named human, institution or independently controlled quorum?
38. What is the versioned role-activation/transition contract—objective, work
    mode, principal, model/tool profile, scope, budgets, grounding, accountable
    owner, policy epoch and expiry—and which owner persists it?
39. **Blocking J30:** Which sovereign VCS, isolated worktree, build, test,
    package, SBOM, provenance, signature and artifact-registry owners form the
    reflexive development path, and what qualified commands/receipts connect them?
40. Which change risk classes may progress automatically from sandbox to shadow
    or canary, and which require independent human/multi-party review at build,
    deploy, cohort expansion, promotion and rollback boundaries?
41. Which exact hot-replacement mechanism is qualified per component and change
    class: OTP release/code change, application restart, Pantheon process reexec,
    rolling instance replacement, blue/green routing, or configuration/feature
    activation? What changes explicitly require downtime?
42. What compatibility declaration and test matrix governs wire protocols,
    persistent state, events, policies, telemetry, relationships, mixed versions,
    upgrade order and downgrade/forward-fix behavior?
43. **Blocking J30:** Which accountable promotion principal, promotion-decision
    record owner, deployment executor and observation owner provide traffic
    mirroring, effect suppression, canary cohort selection, state transfer,
    baseline comparison, readiness, progress deadlines and semantic/user probes;
    what exact records separate observe, decide, authorize and execute; and how
    are privacy and production-data constraints enforced?
44. **Blocking J30:** Which recovery-policy administrator, versioned recovery-
    policy/last-known-good record owner, rescue executor, supervisor, authority,
    telemetry, invariant monitor, protected artifact/configuration and operator
    channel form the independent recovery root; which changes require separate
    approval; and how are they kept outside the candidate's failure/credential
    domain under the claimed assurance level?
45. How are in-flight tasks, workflows, agent sessions, rooms, documents,
    events, leases, cursors and uncertain effects drained, checkpointed,
    transferred, fenced or reconciled for each replacement type?
46. How are code rollback, data restoration, event/message reconciliation,
    external-effect compensation and affected-user remediation authorized and
    reported separately when recovery is partial or impossible?
47. What stronger, out-of-band and separation-of-duties process governs
    reflexive changes to Aegis, deployment/supervision, audit, telemetry/safety
    monitors, artifact trust and rollback itself?
48. Who owns the extensible work-mode registry/schema, compatibility and
    lifecycle so custom modes gain human-agent parity without making Pantheon a
    universal semantic owner or permitting unsafe executable extensions?
49. Which outputs/transitions are “material” in each deployment profile, what
    risk/context changes trigger stakeholder-ensemble review, and when may a
    prior review be reused by exact digest and applicability?
50. Which Moira methods, claim/evidence/dissent schemas, fuel bounds, resolution
    kinds and disposition states define the initial stakeholder-review profiles?
51. How does Dendrite's fold/lattice vocabulary represent stakeholder concerns,
    affected-person and public-interest lenses, required coverage, exclusions
    and cross-domain corpus governance without claiming representativeness?
52. How are reviewer independence and correlation declared across model,
    provider, prompt, grounding, evidence, organization, incentive and prior-
    report exposure, and which reviews require blinded/independent first passes?
53. Which contexts require participation by actual users, affected people,
    accountable domain owners, legal representatives or external assessors
    rather than an agent SME lens, and how are consent, accessibility, privacy,
    compensation, mandate, conflict and safe dissent handled?
54. Who may disposition, defer, accept risk for or close a material finding; how
    do unresolved findings reach a decision owner without turning reviewers,
    Moira, Dendrite or Pantheon into the effect authority?
55. Which versioned review rule admits a human-advocate gate request as supported,
    non-frivolous and material; who may challenge that classification; how are
    the exact decision, alternatives, affected people, evidence, dissent,
    uncertainty and consequences presented accessibly; and what prevents
    timeout, relabeling, agent impersonation or disposition from bypassing the
    named-human response?

## Source hierarchy and research notes

The synthesis prioritizes:

1. current component requirements, specifications, public clients, live gates,
   and `composition-status.json`;
2. normative standards from IETF, W3C, OMG, Unicode, NIST, and protocol owners;
3. foundational peer-reviewed research for distributed systems, authorization,
   workflow, provenance, logic, and multi-objective choice;
4. official product documentation for industry journey precedents;
5. secondary sources only for discovery, not as the basis of normative claims.

Important local sources include the sibling `requirements.md` and `README.md`
files for Aegis, Agora, Chronos, Daimon, Hermes, Keystone, Moira, and Vellum;
Dendrite's `req/REQUIREMENTS.md`; Moira's
`docs/methodical-agent-harness.md`; and Pantheon's
[`docs/composition/prior-art.md`](composition/prior-art.md). Federation-specific
repository evidence includes `agora-federation`, `vellum-federation`, Chronos's
`docs/federation-integration-v1.md`, and the remote/federation qualification
notes in Aegis, Moira, Daimon and Hermes. Those owner documents remain
authoritative when this blueprint and an implementation detail diverge.

Pantheon's active delivery and goal-reconciliation rules are maintained in
[`delivery-operating-contract.md`](delivery-operating-contract.md). That linked
Vellum artifact is normative for how this blueprint is executed, reviewed and
evolved; it may strengthen this blueprint from evidence but may not narrow or
replace its promised operational journeys.
