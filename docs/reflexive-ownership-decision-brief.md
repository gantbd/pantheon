# Reflexive ownership decision brief

> **Reuse-audit amendment (2026-11-17):** the earlier recommendation to create
> `pantheon-control` is superseded by
> [ADR 0001](decisions/0001-local-control-plane-registries.md). Moira already
> owns qualified governed-agent participation; Rune, Dendrite and Moira are the
> first work-mode capability audit. References below to new Participation and
> Work-Mode registries remain option analysis and requirements, not an accepted
> implementation plan. Pantheon must compose owner-native records before a new
> service can be admitted.

Status: proposed recommendations; no owner is selected or qualified by this
document

Date: 2026-11-17

Scope: the P0 role-activation owner, the custom work-mode registry, and the four
owner classes that block J30 reflexive development.

## The short version

Pantheon should start with a **single-user, local, externally supervised**
profile. It should not begin with Kubernetes, arbitrary plugins, or Pantheon
replacing itself without an outside safety path.

That first profile provides **operational separation for trusted candidate
code**: a crash or bad configuration should not destroy observation or recovery.
Processes running as the same OS user are not a security boundary against a
malicious or fully compromised candidate. Security-, authority- or recovery-
sensitive self-modification requires a distinct OS identity and filesystem
ownership, a sandbox/VM/container boundary, or an equivalently qualified
isolation mechanism.

| Decision | Recommended first owner/pattern | Why |
| --- | --- | --- |
| Role activation | A Participation Registry with an append-oriented, revisioned record | One honest place to answer “who is acting in which role, under what delegation?” without confusing role with permission |
| Custom work modes | A logically separate declarative Work-Mode Registry | Allows new modes and versions without loading arbitrary code or making Pantheon own their domain semantics |
| Build and provenance | GitHub Actions as the candidate build platform; digest-addressed native bundles in an OCI registry | Pantheon already uses pinned GitHub Actions and locked Rust builds; OCI digests keep storage portable |
| Deployment | Moira owns the rollout decision record; a named human makes the final production go/no-go; a small external `pantheon-steward` executes it under Aegis authority | Separates advocate/review, decide, authorize and execute while preserving Pantheon drain/readiness/receipts |
| Observation | A separate `pantheon-observer` using owner-native read clients plus an OpenTelemetry Collector | Combines semantic probes with vendor-neutral telemetry; candidate telemetry alone cannot judge the candidate |
| Recovery | The host-supervisor profile owns a recovery-policy ledger; a minimal rescue path applies it to protected last-known-good slots | Names who may designate recovery state and survives failure of Pantheon and the deployment steward |

These are logical owners, not a request for one daemon per noun. The first local
implementation should run the Participation and Work-Mode registries as two
typed contracts/ledgers in one small `pantheon-control` process, using separate
stores/namespaces, capabilities and receipts. Split them into deployment units
only when fault isolation, scaling, retention, independent upgrade or authority
evidence requires it. Steward/observer/rescue may share an installer, but their
runtime identities and update paths remain separable. Sharing a package or
process is not permission to merge semantic ownership.

## Decision rules

The recommendations follow six rules:

1. **Records have one semantic owner.** Pantheon may project them but does not
   reinterpret them.
2. **A role is not a permission.** Role activation refers to Aegis evidence; it
   never mints or extends authority.
3. **Definitions are not executable extensions.** A work-mode registration may
   reference an owner-native implementation, Rune method or Moira workflow, but
   the registry does not load supplied code.
4. **Promotion uses immutable identity.** A candidate is a digest plus verified
   provenance, not a branch, tag, mutable path or “latest” label.
5. **The candidate cannot grade or rescue itself.** In trusted-development,
   paths and credentials are not intentionally exposed to the candidate, but
   this is convention and accidental-failure resistance only. The isolated
   profile must enforce that at least one semantic probe and the known-good
   recovery path are outside its filesystem, credential, process and signal
   authority.
6. **Start with the topology Pantheon actually has.** Today that is a local Rust
   composition over Unix processes/sockets with bounded stdio management—not a
   container fleet.

## Decision 1 — who owns role activation?

### What needs an owner

The owner persists a statement such as: “agent session A is acting as security
reviewer for work W, under delegation D, from revision 7 until expiry T.” It
also retains transitions, changed scope, rationale and exact replay/conflict.
It does not decide whether the principal may perform an effect.

W3C PROV is useful precedent: it distinguishes an agent's association and role
in an activity from responsibility delegation. Pantheon should preserve that
separation rather than treating a role label as RBAC. See
[PROV-DM](https://www.w3.org/TR/prov-dm/).

| Option | Pros | Cons |
| --- | --- | --- |
| Keep it in each UI/session | Very small initial change | Lost on detach/restart; clients disagree; presentation becomes hidden authority |
| Put it in Aegis | Aegis already handles subjects and policy | Collapses participation into authorization; encourages “role implies permission” |
| Put every activation in Moira | Strong durable participation/deliberation evidence and recovery | Forces freeform rooms, analysis and direct owner work into a workflow; expands Moira beyond coordination |
| Let every domain owner invent its own record | Strong sovereignty; no new service | No portable parity or cross-mode history; duplicated schemas and inconsistent recovery |
| Create a bounded Participation Registry | One versioned contract and projection; durable human/agent parity; clean audit | New store/service; must resist becoming a universal identity, HR or authorization system |

### Recommendation

Create a **Participation Registry** as a narrow sovereign logical owner. For the
first profile, implement it as one ledger in the shared `pantheon-control`
process, with its own SQLite store/namespace, typed contract and headless
protocol. Its record contains only stable principal/resource references,
principal type, declared role/work mode, delegation/authority references,
bounds, grounding, accountability, policy epoch, time, rationale and revision.

“Append-oriented” is an application transition invariant, not a tamper-evidence
claim. In trusted-development, a process with the same OS identity can rewrite
or replace the SQLite file. The product may claim replayable history and detect
invalid transitions through its own API; it may not claim hostile-write
resistance. Stronger integrity requires the isolated profile's enforced file/
credential ownership plus a separately verified hash/signature chain, external
anchor or equivalent qualified mechanism.

It should use optimistic revision, stable operation identity, exact-input
replay, changed-input conflict and historical lookup. Aegis remains the only
authority decision source; Chronos, Agora, Moira and other semantic owners remain
authoritative for their resources. A future federated registry may shard by
journey home, but P0 does not need global consensus.

Moira participation records should link to the registry activation when Moira
coordinates the work; they should not be duplicated or reinterpreted. The
registry is a new logical peer boundary even though its first deployment shares
the `pantheon-control` process with the Work-Mode Registry.

Do not use organizational job titles as activations. “VP,” “architect,” and
“operator” are context, not ambient permission.

## Decision 2 — who owns custom work-mode definitions?

### Useful precedents

Kubernetes custom-resource definitions show a helpful lifecycle: structural
schemas, multiple served versions, explicit storage versions, conversion,
incremental migration, deprecation and removal only after stored objects have
moved. Pantheon should borrow the lifecycle, not Kubernetes itself. See
[CRD versioning](https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definition-versioning/)
and the [Kubernetes deprecation policy](https://kubernetes.io/docs/reference/deprecation-policy/).

Temporal's worker versioning provides the complementary runtime lesson: long-
running executions stay pinned to a build/version instead of silently changing
underneath. See [Temporal worker deployment guidance](https://docs.temporal.io/best-practices/worker).

| Option | Pros | Cons |
| --- | --- | --- |
| Hard-code a Rust enum | Simple, exhaustive, easy to test | Every new mode requires a Pantheon release; contradicts open-ended modes |
| Treat Rune/Moira as the registry | Reuses methods and workflow machinery | Forces freeform/domain-native modes into workflow; confuses authored method with product registration |
| Load plugins/scripts supplied by a mode | Maximum flexibility | Turns registration into code execution, dependency and authority injection; unsafe foundation |
| Declarative Work-Mode Registry referencing sovereign implementations | Extensible, inspectable, versionable and safe to project | New lifecycle/store; compatibility and retirement need real work |

### Recommendation

Create a **Work-Mode Registry** logically separate from the Participation
Registry but initially deployed as the second ledger in `pantheon-control`. A
registration is canonical bounded data: namespace/name/version, semantic owner,
roles, inputs/outputs, state/outcome vocabulary, allowed transition names,
evidence/nonclaims, authority boundaries, review triggers, recovery, retention,
disclosure and presentation schemas.

The registration may refer to:

- a Rune method and Dendrite grounding digest;
- a Moira workflow/deliberation profile;
- owner-native commands and observation projections; or
- a freeform mode with no executable workflow.

It never embeds executable code, credentials or arbitrary endpoints. Store
accepted definitions by digest, retain every served/stored version, pin each
instance, and make migration a separate owner command. Compatible additions may
share a major version; changed meaning requires a new major identity. Retirement
leaves an audit tombstone.

Git review is a good authoring and change-control path, but Git files alone are
not the runtime owner: the registry must return admission, conflict, migration
and retirement receipts.

## Decision 3 — who builds and identifies a candidate?

### Current Pantheon position

Pantheon's GitHub workflow already uses a pinned checkout action, exact sibling
repository commits, Rust 1.95, locked dependencies and `make check`. That is a
strong CI starting point. It currently verifies source; it does not produce,
attest, register or promote a runnable candidate.

[SLSA provenance](https://slsa.dev/spec/v1.2/build-provenance) describes the
source, build platform, invocation, dependencies and output subjects needed to
verify where an artifact came from. GitHub Actions can generate provenance and
SBOM attestations for binaries or container images, although private/internal
repository attestation availability depends on the GitHub plan. See
[GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations).
OCI registries store objects by digest and attach referrers such as attestations;
they need not force the local runtime to use containers. See the
[OCI Distribution Specification](https://specs.opencontainers.org/distribution-spec/).

| Option | Pros | Cons |
| --- | --- | --- |
| Agent runs `make` and deploys its local binary | Fast, offline, excellent for development | Same principal authors/builds/deploys; weak provenance; mutable workspace; not promotable |
| GitHub Actions produces attested native bundles | Extends current CI; isolated hosted builder; good provenance and review integration | External dependency, plan limits, runner trust and service availability |
| Self-host a hermetic build platform | Maximum control, offline/private operation, customizable isolation | Highest operational burden; the builder itself becomes critical infrastructure |
| Require two independent builders | Strong reproducibility/tamper signal | Costly; byte reproducibility across macOS toolchains may be difficult initially |

### Recommendation

Use **GitHub Actions as the first candidate build owner**, conditional on the
repository plan supporting the required private attestation workflow. Keep local
agent builds explicitly `development-only`.

This condition is a concrete pre-build gate, not a footnote. Before enabling the
release workflow, the Moira-owned release/rollout record must retain an approved
`BuildPlatformEligibilityV1` observation containing at least:

- repository owner/name and exact visibility (`public`, `private` or `internal`);
- GitHub plan and organization/account supplying Actions;
- whether GitHub-hosted artifact attestations are available for that exact
  visibility/plan at the observed time;
- public or private Sigstore/transparency-log behavior and its disclosure impact;
- trusted reusable-workflow identity/digest, OIDC issuer/subject policy and
  runner class;
- OCI registry/referrers support and verifier version/profile; and
- observer, evidence source, timestamp, expiry/recheck trigger and approval.

GitHub currently offers attestations for public repositories on current ordinary
plans, while private/internal repository use requires GitHub Enterprise Cloud;
see [attestation availability](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).
If the observed repository/plan combination is ineligible, GitHub-native
attestation is refused: the profile must select and qualify the Sigstore/self-
hosted alternative before candidate production. A plan name in static config is
not sufficient; eligibility is observed and expires because service offerings
can change.

The release workflow should:

1. accept only a reviewed immutable source commit and exact sibling commits;
2. run the locked checks and security/advisory gates;
3. build platform-specific native bundles, not containerize by default;
4. produce a manifest, SBOM, test evidence and SLSA/in-toto provenance;
5. publish bundle and attestations to an OCI registry by digest;
6. sign/attest through short-lived OIDC identity where available; and
7. verify the digest and trust policy before handing it to deployment.

The trusted release workflow must be protected independently of the candidate
change—for example, a pinned reusable workflow in a separately controlled
repository or a protected workflow path requiring independent review. Otherwise
an agent could modify the build rules and then cite the resulting provenance as
independent evidence. Provenance faithfully reports which builder and definition
ran; it does not make an untrusted definition trustworthy.

GitHub's documentation is explicit that attestations link an artifact to its
build but do not prove the artifact is secure. Pantheon must keep testing,
review and admission as separate gates. If GitHub's plan or availability is
unacceptable, preserve the same candidate manifest and verifier contract while
substituting a self-hosted builder and [Sigstore identity-based signing](https://docs.sigstore.dev/cosign/signing/overview/).

## Decision 4 — who deploys the candidate?

### Useful precedents

Host service managers keep processes alive: `launchd` manages persistent macOS
services and systemd supports restart, watchdog and rate-limit behavior. They do
not understand Pantheon peer drains, uncertain effects or semantic receipts.
[Apple Service Management](https://developer.apple.com/documentation/appkit/managing-ongoing-background-processes-in-your-mac)
and [systemd service behavior](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)
are therefore substrate, not the whole deployment owner.

Kubernetes Deployments supply rollout history, gradual replacement, pause,
progress deadlines and rollback. That is strong precedent for the later fleet
profile, but it is excessive for Pantheon's current single-host Unix topology.
See [Kubernetes rolling updates](https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/).

| Option | Pros | Cons |
| --- | --- | --- |
| Let `pantheon manage` replace itself | Reuses current validated re-exec path | Candidate and deployer share failure domain; no artifact admission, cohort or external rollback root |
| Let launchd/systemd replace the binary directly | Mature process ownership and restart | No Pantheon-aware preflight, drain, state/effect reconciliation or semantic receipt |
| Add a small external Pantheon steward above the service manager | Understands candidate admission and Pantheon management protocol; remains outside candidate | New security-sensitive component; must be simpler and updated separately |
| Move immediately to Kubernetes/GitOps | Mature fleet rollout ecosystem | Large topology/operations jump; does not solve peer semantics; poor first local profile |

### Recommendation

Use a **Moira rollout saga as the promotion-decision record owner** and build a
minimal **`pantheon-steward`** as the local deployment executor. Moira does not
choose a preferred outcome by itself. The deployment profile names an
accountable rollout decision principal. Automated policy may request or execute
qualified intermediate hold, abort, rollback or bounded exposure reductions,
but the final go/no-go that promotes a candidate to the authoritative/default
production population must be presented to and explicitly decided by a named
human principal. Silence, timeout, prior approval, ensemble agreement, observer
`pass`, or policy default is not a go decision. Aegis authorizes that exact
principal, candidate, target/cohort and transition; the steward executes only a
current accepted Moira decision.

Human gating also applies before other material decisions when the selected
stakeholder-SME method admits a supported, non-frivolous human-advocate finding
that a reasonable human should review the decision. Such a finding creates a
`HumanGateRequestV1`; it does not itself approve, deny or gain effect authority.
The work owner may answer the concern, but cannot disposition it away as
`out_of_scope`, substitute an agent persona, or continue through an automated
default. Moira presents the exact decision, alternatives, affected people,
evidence, dissent, uncertainty, consequence, reversibility and deadline to a
named human gate principal. That human records go, no-go, revise or defer with
rationale under current Aegis authority. A changed decision payload or material
evidence invalidates the response and requires presentation again.

The versioned `RolloutDecisionV1` record contains the candidate and prior digest,
target instances/incarnations and cohort, risk/profile, observation window,
`pantheon-observer` evaluation/evidence digest, unresolved findings, requested
`hold`, `expand`, `promote`, `rollback` or `abort`, accountable decision
principal and principal type, any `HumanGateRequestV1` references, human
presentation/response receipt for final promotion, Aegis decision/epoch,
expected rollout revision, expiry and rationale. Moira returns an exact
acceptance/refusal/replay receipt. The observer supplies evidence but cannot
write this record; Aegis allows or denies but does not own rollout state; the
steward cannot invent or widen the decision. Headless automation may prepare,
request and observe the gate, but cannot synthesize the human response.

Run the steward as a per-user launch agent on macOS and user service on Linux for
trusted-development unless a system-wide/isolated profile is explicitly
required. The host service manager owns the steward's lifetime; the steward owns
only verified candidate admission and execution of the accepted rollout.

On macOS, `SMAppService` is the recommended way to register a visible,
user-controllable persistent background item. That is lifecycle integration,
not candidate isolation. A launch agent running as the same user provides no
hostile-code boundary. The isolated macOS profile needs separately qualified
sandbox/XPC mediation, a distinct service identity where appropriate, or a VM/
container boundary; it must not infer isolation from successful `SMAppService`
registration. See [Apple's background-service guidance](https://developer.apple.com/documentation/appkit/managing-ongoing-background-processes-in-your-mac).

The steward should:

- accept a digest and rollout plan, never a mutable path;
- independently verify provenance/trust policy;
- stage into versioned read-only slots;
- run offline profile/capability/compatibility preflight;
- use the existing `pantheon manage` protocol to observe and drain;
- select drain/re-exec, blue/green or rolling replacement only where qualified;
- bind the operation, old/new digest, target/incarnation and observation window;
- wait for an independent observer evaluation and a separate authorized
  promotion decision, including the explicit named-human go/no-go for final
  production promotion; and
- return exact deploy/hold/promote/fence/rollback receipts.

For a stateful single-home peer, the first implementation should use guarded
drain and replacement—not pretend that a local process restart is a canary.
Rolling/canary deployment begins only when two valid serving instances can
coexist under the peer's authority law.

## Decision 5 — who observes the candidate?

### Why telemetry alone is insufficient

The candidate can emit useful telemetry and still be wrong, omit a signal or
fail before emitting anything. Observation therefore needs two channels:

1. telemetry collected outside the candidate process; and
2. semantic probes using the same public read clients and user-visible paths
   that real users rely on.

The OpenTelemetry Collector provides a vendor-neutral receiver/processor/
exporter boundary and can run as a separate agent or gateway. Its documentation
also calls out the extra complexity of multi-tier deployments, so the first
local profile should use one external collector rather than a fleet. See
[Collector deployment patterns](https://opentelemetry.io/docs/collector/deploy/).
Prometheus's black-box pattern is useful precedent for probing a service from
the outside rather than trusting its self-reported internals. See the
[multi-target exporter pattern](https://prometheus.io/docs/guides/multi-target-exporter/).

| Option | Pros | Cons |
| --- | --- | --- |
| Candidate evaluates its own logs/metrics | Cheapest and richest internal detail | Cannot detect lying, missing or broken instrumentation; circular promotion gate |
| External collector only | Separates pipeline credentials/config; standard signals | Still observes mostly what the candidate emits; telemetry is not semantic proof |
| External collector plus independent owner-native probes | Detects instrumentation failure and user-visible/semantic regressions | More engineering; requires bounded read authority and explicit probe oracles |
| Full commercial observability platform as owner | Fast dashboards, alerting and retention | Vendor coupling; dashboard health can be mistaken for semantic authority |

### Recommendation

Create **`pantheon-observer`** as a read-only rollout observation owner. Run an
OpenTelemetry Collector beside it, not inside the candidate. The observer pins
baseline/candidate populations and combines:

- service-manager/process/resource evidence;
- candidate-emitted OTLP signals with gap/drop/pipeline health;
- owner-native health, revision and effect-knowledge reads;
- selected end-to-end synthetic/user-visible probes;
- cohort and topology identity; and
- explicit privacy, sampling and observation-window bounds.

The observer returns `pass`, `fail`, `inconclusive` or `observation_unknown`
with evidence. It never deploys, promotes or declares a domain effect complete.
Its binary, configuration, credentials and storage must not be writable by the
candidate. A Prometheus-compatible backend is a sensible first metric store, but
the observation contract should not depend on one dashboard vendor.

For the first same-user development profile, “not writable” is an operational
layout invariant and sandbox rule, not containment against hostile native code.
A stronger profile must run the candidate under a separate OS identity or
isolation boundary. Linux systemd can combine a dedicated/dynamic user with
read-only filesystem protection and explicit writable directories; see
[systemd execution environments](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html).
On macOS, XPC/launchd patterns provide process lifecycle and privilege
separation, but Pantheon would need a deliberately packaged/sandboxed service
profile rather than assuming that two ordinary user processes are isolated; see
[Apple XPC](https://developer.apple.com/documentation/xpc).

## Decision 6 — who owns recovery?

### The recursion problem

If the steward deploys a bad Pantheon build, the steward can roll it back. If a
later change breaks the steward, observation pipeline or rollback logic, a lower
and simpler layer must still recover them. Recovery therefore forms a hierarchy,
not one universal rollback button.

| Option | Pros | Cons |
| --- | --- | --- |
| Candidate rolls itself back | Minimal components | Fails exactly when the candidate is broken or compromised; reject |
| Steward is also the final recovery root | Practical for ordinary Pantheon updates | Cannot safely update or recover the steward itself; one failure domain |
| Host service manager plus protected rescue helper and known-good slots | Independent, simple, works without Pantheon | Platform adapters and careful filesystem/credential ownership required |
| Kubernetes control plane/GitOps reconciliation | Strong fleet recovery and declarative desired state | Too heavy for first local profile; control-plane recovery still needs an outer layer |

### Recommendation

Make the first recovery root a **host-supervisor profile**:

- `launchd` on macOS and systemd on Linux own the long-running steward,
  observer and Pantheon services;
- a host-supervisor-owned Recovery Policy Ledger retains the authorized recovery
  manifest and last-known-good designation;
- a minimal `pantheon-rescue` helper can verify and select protected A/B slots,
  stop/fence the active candidate, start the pinned known-good profile and emit
  a local receipt;
- in the isolated profile, candidate and steward identities cannot write or
  signal the rescue binary, trust policy, recovery ledger or known-good slot;
- at least two known-good revisions and their compatibility metadata are retained;
- recovery remains possible without the TUI, Pantheon, network or telemetry
  backend; and
- data restore, message/effect reconciliation and user remediation remain
  separate from binary rollback.

The deployment profile names a **recovery-policy administrator** as the
accountable decision principal. Changing the recovery manifest, trust policy or
last-known-good designation requires an exact Aegis authorization and, for the
initial profile, independent human approval. The Recovery Policy Ledger returns
a `RecoveryPolicyRevisionV1` receipt binding prior/new revision, permitted and
last-known-good digests, compatibility/state-schema bounds, slot identities,
rescue-helper/trust-root digests, retention, approvers, authority epoch,
rationale and expiry/review trigger. A candidate does not become last-known-good
merely because it was promoted; designation follows its required soak/observation
window and a separate accepted recovery-policy change.

`pantheon-rescue` is an applier and receipt producer, not the policy author. Its
`RecoveryExecutionReceiptV1` records policy revision, from/to digest, fencing,
process/start evidence and residue. In trusted-development, the ledger's write
restrictions are convention and accidental-failure resistance because all
processes may share one user. Only the isolated profile may claim enforced
write/credential/process separation.

There are therefore two initial assurance levels:

- **Local trusted-development:** steward, observer and candidate may run under
  one user for rapid qualification. It protects against ordinary crashes,
  malformed candidates and operator error, not malicious native code.
- **Local isolated:** candidate runs under a different OS identity or VM/
  container; supervisor, observer trust policy, rescue helper and known-good
  storage are owned outside it. This is the minimum profile for security,
  authority, telemetry or recovery-root changes.

Routine Pantheon rollback may be executed by the steward. Updating the steward,
observer, rescue helper or trust root requires a different maintenance profile,
out-of-band verification and normally human approval. TUF's separated roles,
version/freshness checks and rollback/freeze protections are the right precedent
for that update channel; see the
[TUF specification](https://theupdateframework.github.io/specification/latest/).

## Recommended first topology

| Layer | Recommended owner | Trust and mutation boundary |
| --- | --- | --- |
| Definitions | Work-Mode Registry | Immutable declarative definitions and lifecycle receipts only |
| Participation | Participation Registry | Role activations/transitions only; no domain effects or grants |
| Source/review | GitHub repository and protected review policy | Selects immutable source revision; does not create a runnable candidate by itself |
| Build eligibility | Moira release/rollout record | Retains observed repository visibility/plan, attestation availability and approved build trust profile |
| Candidate build | GitHub Actions release workflow | Produces native bundle, manifest, SBOM and provenance |
| Artifact storage | OCI registry | Serves objects and attestations by digest; tags are navigation only |
| Promotion decision | Named human rollout principal recorded by Moira under Aegis authorization | Owns final production go/no-go; bounded automation may request intermediate hold/abort/rollback but cannot synthesize final promotion |
| Deployment execution | `pantheon-steward` | Verifies, stages, drains and executes only the accepted current rollout decision |
| Observation | `pantheon-observer` plus external OTel Collector | Read-only semantic probes and telemetry evaluation; no promotion authority |
| Authorization | Aegis and each semantic owner | Allows exact commands under current facts; no role or signature shortcut |
| Recovery policy | Host-supervisor Recovery Policy Ledger; named recovery-policy administrator under Aegis | Owns recovery manifest and last-known-good designation with versioned receipts |
| Recovery execution | launchd/systemd plus `pantheon-rescue` and protected known-good slots | Applies current policy and can fence/recover without candidate, steward UI, network or telemetry backend |

This gives the critical flow:

```text
reviewed source -> build-eligibility gate -> builder -> digest -> steward -> candidate
candidate -> observer -> evidence/evaluation -> Moira rollout record
human advocate gate request -> named human presentation/response -> Moira rollout record
named human rollout principal -> Aegis authorization -> Moira rollout record
Moira accepted decision -> steward execution
recovery-policy administrator -> Aegis -> recovery ledger -> rescue -> slots/processes
```

The observer informs a bounded policy/owner decision; it does not command the
candidate directly. The rescue edge is intentionally outside the normal rollout
path.

## What not to decide yet

- Do not require containers for the local profile. OCI is an artifact envelope,
  not a runtime mandate.
- Do not select Kubernetes until remote identity, peer federation, replica laws
  and fleet operations are qualified.
- Do not make an observability vendor the semantic observation owner.
- Do not allow arbitrary work-mode plugins or conversion webhooks in P0/P1.
- Do not claim reproducible builds until a second build actually reproduces the
  declared artifact under a qualified comparison.
- Do not hot-upgrade Aegis, the steward, observer or recovery root in the same
  rollout whose safety they govern.

## Recommended decision sequence

1. **Accept or revise the six logical-owner recommendations.** Record each as an
   ADR with scope, non-goals and replacement criteria.
2. **Build P0 truth first.** Participation Registry and UI/TUI/headless role
   projection; no deployment work depends on inferred roles.
3. **Build the declarative Work-Mode Registry and qualify J34.** Keep executable
   implementations behind owner-native clients.
4. **Promote CI into a candidate builder.** Produce and verify one native bundle,
   manifest, SBOM and provenance without deploying it.
5. **Build observer before deployer.** Prove baseline/candidate comparison,
   missing-data behavior and independent probes against manually started builds.
6. **Build steward and rescue together.** Fault-inject lost replies, bad startup,
   stuck drain, corrupt candidate, missing telemetry and failed rollback.
7. **Run J30 first on a low-risk, stateless presentation change.** No Aegis,
   steward, observer, persistence-schema or recovery-root self-update.
8. **Only then qualify stateful peers and federation.** Add mixed-version,
   authority-home, partition and site-loss tests per owner.

## Decision criteria for approval

Before these recommendations become selected owners, the productively
adversarial stakeholder-SME review should answer:

- Does the local profile work on the current macOS development host without
  requiring a cluster?
- Can every owner be observed and controlled headlessly with stable identities
  and receipts?
- Can an operator recover when Pantheon, the candidate, network and telemetry
  backend are unavailable?
- Does every promotion name the accountable principal, Aegis decision, Moira
  rollout record and steward receipt without letting observer or steward decide?
- Does every final production rollout preserve an exact presentation receipt
  and explicit named-human go/no-go, with no timeout, policy or agent substitute?
- Can a supported human-advocate finding trigger a human gate for any material
  decision without becoming an agent veto, and can no owner bypass that gate by
  relabeling or automatically dispositioning the finding?
- Does every recovery-policy change name its administrator, Aegis decision,
  prior/new manifest, last-known-good evidence and receipt?
- Is the profile labeled trusted-development or isolated, and do its claims
  match what OS identity, filesystem, credential and process/signal boundaries
  actually enforce?
- Does the candidate manifest capture every sibling repository commit and build
  input used by the current path-based Cargo workspace?
- Does the unexpired build-eligibility record name exact GitHub repository
  visibility, plan, attestation availability, privacy/transparency behavior,
  workflow identity and verifier profile?
- Can the same contract substitute another CI, OCI registry, service manager or
  telemetry backend without changing journey semantics?
- What is the smallest change safe enough for the first real J30 qualification?

Until those questions are answered with concrete qualification evidence, the
recommendations remain proposed architecture—not implemented capability.
