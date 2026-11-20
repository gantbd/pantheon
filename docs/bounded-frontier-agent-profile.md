# Bounded frontier agent profile

This profile is the admission boundary for a Pantheon-owned frontier session.
It does not grant a provider a shell, filesystem, inherited environment,
ambient network, undeclared MCP server, peer credential or rollout authority.

## Required composition

1. Moira retains the task-agent journey and accountable human.
2. Chronos supplies one exact Running task attempt. Every tool callback
   revalidates that attempt.
3. Tellus supplies a pinned SME plan. `review_required` is visible and cannot be
   restyled as complete stakeholder coverage.
4. Daimon launches an exact provider/profile under a digest-bound deny-default
   sandbox. ACP client filesystem read/write remain disabled.
5. Aegis grants only the command capabilities named by the execution. Daimon
   reauthorizes the exact command-specific capability at callback time.
6. The ACP terminal catalogue contains only closed gateway programs. It never
   contains a shell, interpreter, package manager, generic `curl`, editor or
   arbitrary Pantheon CLI entry point.

There is deliberately no `pantheon.*` umbrella authority. Each component and
operation plane is admitted independently. For example, a session may hold
Chronos `tasks.workflow.observe.v9` and Tellus `tellus.sme-plan.v1` without
receiving `tasks.mutate.v1`, Vellum `workspace.inventory.v1`, Agora
`messages.speak.v2`, or any
other Pantheon-composed capability. Component admission still routes through
the component's owner client and does not transfer sovereignty to Pantheon.

The initial authorities are distinct:

- `rooms.read.v2` and `messages.read.v2`: separate Agora gateways for one exact
  room summary or one message page of at most 64 items. The deployment fixes
  the owner socket, participant claim and transport profile. Exact-room lookup
  scans at most eight internally consistent owner pages and emits no catalogue;
  absence refuses without substituting a visible room. Message reads bind one
  room and cursor and intentionally disclose body text, author, logical time and
  addressees. Neither read grants room creation, speaking, membership,
  moderation, task mutation or agent execution.
- `messages.actor-publication.observe.v1` and
  `messages.actor-publication.execute.v1`: separate durable Agora publication
  tools. A private, owner-only request document fixes a fresh nonzero operation
  identity, exact room revision, logical time, body and a sorted bounded
  addressee set. Execute additionally requires an exact confirmation bound to
  the SHA-256 digest of the complete request bytes. Output retains only bounded
  facts and the owner receipt, never body or addressee content. Unknown delivery
  forbids blind retry; the observation tool reconciles the same operation and
  intent to owner-retained `accepted` or authoritative `absent` evidence. The
  deployment pins the private request path and confirmation in the command;
  the model receives no path or arbitrary filesystem argument. Pantheon opens
  only one owner-private, non-symlink, single-link regular file capped at 64
  KiB. A Vellum-backed staging adapter may supply that immutable effect document
  later without changing the Agora owner contract.
- component-owned capability IDs such as `tasks.workflow.observe.v9` and
  `tellus.sme-plan.v1`: each exact catalog entry fixes the Pantheon subcommand
  and deployment arguments; the model cannot append a different operation or
  infer a sibling component/control capability;
- `product-alignment.observe.v1`: the deployment fixes one Chronos alignment
  owner directory and one exact scope. The model supplies no arguments. The
  shared owner client returns content-free current goal, control-task, cohort,
  budget, evidence, dissent, human-gate, proposal and currentness facts. It
  exposes neither the opaque owner configuration nor Vellum credentials and
  grants no proposal creation/application, task mutation, reprioritization,
  acceptance, sign-off, role activation or execution authority.
- `agent.web.fetch.v1`: the `pantheon agent-web-fetch` gateway with the policy
  path fixed in the command prefix. Only the URL is model-supplied, and the
  gateway revalidates it against exact hosts and bounds.
- `workspace.snapshot.v1`: the `pantheon vellum-document-read` gateway with the
  socket, optional credential and byte ceiling fixed in the command prefix.
  Only one locator and an optional UTF-8 byte range are model-supplied. Vellum resolves it against current
  workspace membership, coalesces absent and foreign locators, selects and
  snapshots the exact document, then releases the witness before Pantheon
  emits bounded content. The locator and witness are never returned; ranged
  reads retain whole-document revision, byte length and digest evidence.
- `verification.result-envelope.validate.v1`: the deployment fixes one
  owner-private request document and supplies no model argument. The closed V1
  schema binds a fresh operation identity, exact Chronos task incarnation and
  attempt, Vellum workspace/document revision and digest, governed test
  capability/profile digest, Daimon execution/evidence/output digests, bounded
  output size, duration and a disposition-consistent exit code. Unknown fields,
  insecure links/permissions, wrong owners, zero identities and inconsistent
  pass/fail claims refuse before any owner connection. Success is explicitly
  `validated_not_published`: supplied references remain claims until their
  owners are re-observed. The pending Moira composition must either govern the
  test from dispatch through its existing continuation/output lifecycle or
  admit a distinct versioned external-Daimon receipt; it may not manufacture
  provenance by replaying an already executed test as a new continuation.
- `moira.task-agent.catalogue.v1` and `moira.task-agent.observe.v1`: separate
  read-only Moira gateways. The deployment fixes the owner socket; the former
  takes no model argument and the latter takes one opaque owner identity and
  journey sequence. Neither admits preview, instantiation, progression, peer
  effects or reconciliation.
- `moira.deliberation.observe.v1`: one exact Moira stakeholder deliberation and
  its revision-stable review. The deployment fixes the owner socket and the
  model supplies one opaque deliberation identity. It discloses the
  owner-published stakeholder standing, coverage/equity, grounding, evidence,
  dissent, correlation and pending or settled human-gate facts. It grants no
  ensemble-fitness inference, independence claim, gate response, role
  activation, task mutation, agent execution, approval, sign-off or rollout
  authority. Absence refuses without selecting a different ensemble.
- `task-conversation.v1`: the exact relationship-observation gateway. The
  deployment fixes the adapter socket and the model supplies either one task
  reference or one conversation reference. Its result is relationship truth;
  room activity is not promoted to task progress.
- `ledger.events.disclose.v1`: bounded Hermes event disclosure for one stream,
  start and limit, retaining the command's exact disclosure confirmation.
  Acknowledgement and compaction remain separately guarded and unadmitted.
- `agents.session.inspect.v1`: the `pantheon daimon-agent-observe` gateway for
  one exact agent-session identity against a deployment-fixed Daimon socket. It
  reuses Daimon's stable inspect/events/inspect owner algorithm and discloses
  only session facts plus content-free bounded event metadata. Assistant text,
  thoughts, terminal output, tool details and file paths remain undisclosed;
  observed provider capabilities grant no execution or tool authority.
- `runtimes.observe.v1`, `artifacts.observe.v1`, and `artifacts.catalog.v1`:
  separate Hestia gateways for one exact runtime, one exact artifact, or the
  deployment catalogue capped at 128 entries. They preserve owner revision,
  incarnation, lifecycle, readiness basis/time, placement posture, model
  identity, immutable source revision, license, expected size/digest and
  admission progress while withholding executable paths, arguments,
  environment, registry locators, downloader details, storage paths and process
  handles. Observation grants no acquisition, start, drain, provider
  qualification, inference, or task authority.

The catalogue describes independently admissible commands, not a default role
bundle. A deployment may admit any strict subset, and must deny every omitted
command even when another command belongs to the same component.

Not every admitted tool is a Pantheon CLI gateway. `owner_tools` records
component-native routes separately. Daimon's `verification.test.run.v1` is one
operator-named ACP terminal catalogue entry
whose command-specific Aegis capability and stable semantic capability are
revalidated at callback time. Its program and argument prefix are
deployment-owned, `allow_additional_arguments=false` makes that prefix the
complete argv, the workspace/executables are canonicalized, the environment is
explicitly allowlisted, and output is bounded. The model may select the entry;
it receives no shell, arbitrary flags, path expansion or Pantheon gateway by
implication. Pantheon does not restyle this as its own command.

The same owner-tool shape now catalogues three deny-default design-proof
capabilities: `rune.machine-path.validate.v1`,
`smith.compatibility.inspect.v1`, and `smith.presentation.prepare.v1`. Each
entry fixes the exact workspace, executable, source, target and mandatory-stage
arguments in the deployment-owned complete argv; the model appends nothing.
Rune path checking does not declare semantic completion. Smith compatibility is
read-only, and presentation bytes remain bounded process output with no output
path or Vellum publication authority. Catalogue presence grants none of these
commands; Daimon and Aegis still admit and reauthorize each one independently.
`rune.source.validate.v1` remains unavailable: current Scry success emits the
complete typed IR, and merely raising an output bound would not provide a
content-minimal validation verdict.

`pantheon agent-tools` also returns a machine-readable `unavailable` collection.
It currently names Rune's content-minimal source-validation verdict, Oculus's
owner/query runtime, Smith executable projection, and Vellum revision-bound
change-set proposal with their next admissible seams. Agora's read,
durable-publication observation, and durable-publication execution tools remain
independent grants.
`aggregate_fallback=false` is
normative: neither an agent nor a launcher may replace a missing component tool
with Pantheon's aggregate headless observation. Aegis authority and human-gate
responses are deliberately absent from this gap list because they are guarded
controls, not autonomous-agent conveniences.

An unavailable entry is keyed by both `component` and `surface`; it does not
withdraw or weaken independently admitted sibling surfaces on that component.

## Typed development toolchain target

The intended development profile replaces general shell/filesystem access with
independently admissible semantic tools wherever the work is expressible:

1. Tellus asks Dendrite for a pinned, provenance-bearing stakeholder/SME plan;
   Moira retains equitable participation, dissent and any required human gate.
2. Rune admits and validates the exact versioned machine/plan, including its
   obligations and refusal states. A model cannot weaken the machine to make a
   later target compile.
3. Smith accepts only that admitted Rune/typed-IR identity and emits a
   compatible projection or a named refusal. It receives no arbitrary output
   path and cannot publish directly into the host filesystem.
4. Vellum exposes an exact bounded workspace snapshot and receives the Smith
   projection as a revision-bound proposed change set. Vellum owns validation,
   diff, conflict handling and atomic publication; “Smith forges into Vellum”
   is composition through those owner contracts, not Smith acquiring editor or
   filesystem authority.
5. Daimon and Aegis bind each invocation to one current task attempt, semantic
   capability, budget and effect authority. Oculus correlates events, logs,
   metrics and traces with the same lineage without treating telemetry as the
   outcome oracle.

The minimum independently admitted capability classes are therefore Tellus SME
planning, Moira deliberation, the catalogued Rune path validator, Smith
compatibility/presentation preparation, Vellum workspace snapshot and future
change-set proposal, governed verification and lineage observation. The three
Rune/Smith V1 identities are now stable catalogue entries and pass the focused
catalogue qualification. That does not mean a Daimon provider continuation has
admitted or executed them. Content-minimal Rune source validation, executable
Smith projection, Vellum change-set publication and Oculus lineage observation
remain explicit unavailable surfaces; Pantheon must not invent a generic
gateway in the interim.

This profile can eliminate shell access for the covered development algebra,
not by assertion for every possible engineering operation. Dependency
acquisition, target-toolchain execution, packaging, signing and deployment need
their own closed owner tools or must refuse as unsupported. There is no fallback
from an unavailable semantic capability to shell, interpreter, arbitrary
Pantheon command or ambient file write.

Every accepted projection must join one signed, immutable lineage bundle:
canonical machine/version/digest, typed IR, Rune/Scry/Smith build identities,
target and executable digest, compatibility report, Vellum input/output
revisions, test and instrumentation identities, task/agent attempt and runtime
telemetry identity. The signature attests lineage and authorization under its
stated policy; it does not prove ensemble adequacy, design correctness or
runtime fitness. Missing or mismatched lineage is inconclusive and cannot
satisfy review, completion or rollout gates.

This target is shared by existing Chronos work rather than a new umbrella item:
`pantheon-frontier-capability-sandbox` owns deny-default agent admission,
`rune-compiler-supervisor-semantic-parity` owns non-weakening projection and
lineage, and Vellum's existing bounded workspace/change-set items own editor
publication. Their dependency linkage must remain visible before the frontier
worker can become operational.

Rune `16615a0` now dogfoods this target as the canonical
`bounded-agent-development` machine. Its 11 states and 18 transitions make
ensemble fitness, design admission, machine validation, fail-closed projection,
Vellum proposal/publication reconciliation, independent verification and human
acceptance explicit. Focused qualification proves every authored context is
fold-resolvable, mandatory stages dominate terminal acceptance, Markdown
projection is losslessly recoverable, and the current Rust executable refuses
before bytes because precondition and provenance semantics are still missing.
That refusal is the intended current result, not a failed demonstration.

## Role admission planning

`pantheon agent-role-plan ROLE [SEMANTIC_CAPABILITY ...]` evaluates a candidate
least-authority tool set for the four immediate Operations workers:
`product-manager`, `sre`, `frontier-collaborator`, and `verification-worker`.
The arguments are semantic capability identifiers from `agent-tools`, not
opaque grants. Unknown, duplicate, wildcard and aggregate `pantheon*`
capabilities refuse; known capabilities outside the selected role's minimum
set remain visibly extraneous rather than silently broadening the role.

The result separately reports missing candidate tools and structural owner
surfaces. It is a complete plan evaluation, never an activation receipt:
`operational` remains false even when every semantic tool is present. Actual
operation still requires progression from Moira's existing replay-safe
task-agent journey into current opaque Aegis/Daimon command grants, one exact
Running Chronos attempt, an owner-bound worker lifecycle and a completed-work
receipt. The existing journey already retains its versioned template/profile,
exact task evidence, work mode, budgets, grounding and authority-policy
digests, initiating principal, accountable human, objective/acceptance
evidence, TTL, consequence confirmation and replay receipts. Pantheon must add
only the missing current delegation, expiry and effect progression; it must not
create another role registry. Human gates and authorities the role must never
exercise are machine-readable parts of each profile.

The current profiles intentionally fail closed. In particular, the SRE still
requires Oculus's catalogue/query owner, a durable Hermes trigger subscription,
a typed Chronos incident-admission proposal and the shared role/worker owners;
Hermes disclosure, shell output or Pantheon aggregate observation cannot stand
in for Oculus. The product manager now has independently admissible alignment
and stakeholder-ensemble reads, including exact pending human-gate references;
changed Vellum-backed proposal authoring remains unavailable. Frontier workers
now require all three fixed Rune/Smith design-proof tools, but retain their
provider continuation and Aegis gaps. The verification
worker now reuses Daimon's allowlisted exact-argv runner and has a closed
pre-publication result envelope, but still lacks the Moira publication adapter
and the replay-safe task-to-session composition that drives Pantheon's already
retained Moira/Chronos worker. Chronos scheduling, grant, lease, completion and
uncertainty mechanics are not missing and must not be reimplemented. These are executable
blocker inventories, not claims that named roles are running.

Example Daimon terminal fragment (identifiers and paths are deployment-owned):

```json
{
  "capability": "00000000000000000000000000000000",
  "boundary_program": "/absolute/daimon-process-boundary",
  "commands": [
    {
      "command": "chronos-task",
      "capability": "11111111111111111111111111111111",
      "semantic_capability": "tasks.workflow.observe.v9",
      "program": "/absolute/pantheon",
      "argument_prefix": ["chronos-task-observe", "/absolute/chronos-service"],
      "allow_additional_arguments": true
    },
    {
      "command": "vellum-read",
      "capability": "33333333333333333333333333333333",
      "semantic_capability": "workspace.snapshot.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["vellum-document-read", "/absolute/vellum.sock", "/absolute/vellum.credential", "262144"],
      "allow_additional_arguments": true
    },
    {
      "command": "daimon-agent-observe",
      "capability": "44444444444444444444444444444444",
      "semantic_capability": "agents.session.inspect.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["daimon-agent-observe", "/absolute/daimon-agent.sock"],
      "allow_additional_arguments": true
    },
    {
      "command": "hestia-runtime-observe",
      "capability": "55555555555555555555555555555555",
      "semantic_capability": "runtimes.observe.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["hestia-runtime-observe", "/absolute/hestia.sock"],
      "allow_additional_arguments": true
    },
    {
      "command": "hestia-artifact-observe",
      "capability": "66666666666666666666666666666666",
      "semantic_capability": "artifacts.observe.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["hestia-artifact-observe", "/absolute/hestia.sock"],
      "allow_additional_arguments": true
    },
    {
      "command": "hestia-artifact-catalog",
      "capability": "77777777777777777777777777777777",
      "semantic_capability": "artifacts.catalog.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["hestia-artifact-catalog", "/absolute/hestia.sock"],
      "allow_additional_arguments": false
    },
    {
      "command": "web",
      "capability": "22222222222222222222222222222222",
      "semantic_capability": "agent.web.fetch.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["agent-web-fetch", "/absolute/web-policy.json"],
      "allow_additional_arguments": true
    }
  ],
  "environment_names": [],
  "max_handles": 7,
  "max_output_bytes": 1048576
}
```

The compatibility scope capability must still be a valid configured tool but
does not authorize any command when command-specific capabilities are
present. Deployments retain the mapping from each opaque Aegis/Daimon tool ID
to its stable component capability ID; wildcard peer names and aggregate
`pantheon` grants are invalid profile shapes.

The Chronos entry deliberately invokes `chronos-task-observe`, not Pantheon's
aggregate `headless` snapshot. Its only model-supplied argument is one task ID;
the service directory is deployment-fixed and the response cannot disclose the
board or sibling tasks. An aggregate headless route is a human/operator
overview, not an admissible component tool.

## Web policy

The policy is strict JSON:

```json
{
  "version": 1,
  "curl_program": "/usr/bin/curl",
  "curl_sha256": "sha256:REVIEWED_EXECUTABLE_DIGEST",
  "allowed_hosts": ["example.com"],
  "max_response_bytes": 262144,
  "timeout_seconds": 10,
  "user_agent": "Pantheon bounded frontier/1"
}
```

Hosts are exact lowercase DNS names, not suffix patterns. IP literals,
userinfo, non-default ports, fragments and redirects are refused. Content is
returned with source URL, host, byte length and SHA-256 plus
`trust=untrusted_external_content`. Retrieval is evidence, not an instruction
or permission to follow links.

## Qualification still required

Before a frontier row can report `ready`, a real provider journey must prove:

- the selected Pantheon command and web fetch succeed under their distinct
  grants and retain terminal/tool evidence;
- swapping or omitting either capability is refused before process dispatch;
- arbitrary command, file read, file write, shell and direct provider network
  attempts fail;
- restart reopens exact receipts without provider redispatch;
- prompt/context, tool arguments and content never enter default logs, metrics
  or traces;
- Claude remains no-tool/unavailable until its adapter has an equivalent
  host-tool continuation; it must not inherit ACP evidence by analogy.
