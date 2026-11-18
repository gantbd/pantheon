# Pantheon terminal experience

Pantheon should feel like one operations workbench without becoming one
semantic engine. The shell owns discovery, navigation, focus, workflow chrome,
and a bounded activity timeline. Each peer still owns its observations,
configuration schema, guards, confirmations, authority, effects, receipts, and
uncertainty.

The selected product is a single user operating one local machine. GUI,
multi-user, remote, and fleet management are later profiles.

## Precedents

The interaction model deliberately borrows proven ideas rather than copying a
particular screen:

- [Ratatui's application patterns](https://ratatui.rs/concepts/application-patterns/)
  separate input, state transitions, actions, and immediate-mode rendering.
  Its component template likewise treats event handling and application state
  as application responsibilities rather than widget behavior.
- [K9s](https://github.com/derailed/k9s) combines resource views, command mode,
  filtering, contextual hotkeys, breadcrumbs, confirmations, read-only policy,
  and continually refreshed state. The important lesson is that navigation and
  action discovery use the same resource vocabulary.
- [Lazygit's keybinding and custom-command model](https://github.com/jesseduffield/lazygit/blob/master/docs/Custom_Command_Keybindings.md)
  makes actions contextual, searchable, prompt-driven, and visible in generated
  help. Long-tail actions do not each need a permanent global key.
- [bottom](https://github.com/ClementTsang/bottom) demonstrates adaptable
  information density, focusable widgets, search, zoom, and a lightweight help
  path without turning the terminal into an imitation desktop GUI.

Pantheon adds a stricter rule: no shell action may bypass the exact peer-owned
public client or guard merely because a convenient command palette exists.

## Instrumentation requirement

Instrumentation is part of the workbench contract, not a post-launch add-on.
The shell must measure its own orchestration without claiming peer internals:

- action dispatch and settlement latency, grouped by stable action and peer;
- refresh request age, last observed freshness, and work currently in flight;
- accepted, refused, failed, cancelled, and outcome-unknown dispositions;
- fences entered, explicit recovery attempts, and durable receipt recovery;
- bounded event and counter retention suitable for one user on one machine.

Instrumentation is acceptance evidence, not decoration. Pantheon must correlate
each guarded submission from shell dispatch through the peer's explicit
terminal disposition, receipt, and refreshed projection. A fast enqueue is not
a fast operation. The live sheet therefore exposes in-flight work, full
round-trip latency, outcome-unknown settlement, and budget violations. Missing
correlation or settlement is a capability defect; measurements that expose a
blocking call, stale projection, queue growth, or slow path require a code fix
or an explicit, tested budget change.

Pantheon is an integration boundary, so this requirement is end-to-end rather
than TUI-only. One user operation receives a content-free local correlation;
every participating subsystem gets a distinct child span under that same
correlation. The retained stage ledger must prove the path through
`shell_dispatch`, `peer_pending`, any explicit peer receipt or recovery fence,
and `terminal_disposition` only after the final projection or durable receipt
that the workflow promises. Whole-suite refresh and topology lifecycle work
therefore remain one trace even though their peer spans settle independently.
Peer-internal stages may be joined only through an owner-published correlation
field; idempotency keys, grant handles, prompts, documents, and payload text are
not telemetry. Where an owner protocol cannot carry or return correlation,
Pantheon records the exact observable boundary and treats deeper attribution as
an explicit integration gap rather than inventing it.

The current owner-contract audit constrains the next cross-process work:

| Owner | Existing identity/correlation | Pantheon disposition |
| --- | --- | --- |
| Hermes | Explicit authority-free correlation on event envelopes | Safe to join when an observed operation already publishes a matching envelope; never rewrite historical correlations |
| Chronos | Audit/effect correlation in engine-owned command and journal contracts | Join only through a public client API that accepts and returns it; the TUI must not reach into the engine journal |
| Agora | Session-scoped client request sequence | Use for exact request settlement only; it is not a caller-selected distributed trace parent |
| Moira | Workflow/saga identities and exact receipts | Retain as semantic evidence beneath the local span; do not replace workflow identity with a trace ID |
| Daimon and Aegis | Exact idempotency, session, revision, epoch, and receipt identities | Preserve for safety and replay; never repurpose them as telemetry |
| Vellum and Tellus | No uniform Pantheon-selected trace field on the composed path | Measure through the client terminal projection and record deeper correlation as unavailable |

The initial single-machine budgets are deliberately concrete:

- local input routing, palette work, and shell dispatch: 50 ms hard ceiling;
- guarded local-peer submission to first explicit terminal disposition: 2 s;
- reference 120×40 palette workload (1,000 searches and 100 full renders): 2 s;
- at most 32 correlated peer operations, 64 completed/in-flight trace records
  with eight stages each, 64 action metric series, 64 activity entries, and a
  64-byte minibuffer query retained in memory;
- no transport wait or peer computation on the render/update thread.

CI runs the reference workload through the same instrumentation and fails on a
budget violation. Component and live tests must additionally exercise accepted,
refused, failed, and outcome-unknown paths; recovery is a new explicit attempt,
never an invisible continuation. Agora, Moira, Daimon, Hermes, Aegis, and
Tellus now publishes a minimal peer-owned read-settlement signal, allowing
Pantheon to correlate palette and direct-key requests through a current or
failed projection without parsing presentation text. Chronos publishes changed
projection settlement through its hub; Vellum's synchronous service and poll
paths are timed directly. Explicit age/freshness budgets for every projection
remain a delivery requirement where the peer protocol exposes that evidence.
Presentation commands use the same correlation discipline: input classification,
minibuffer resolution, dispatch, user-visible feedback, the first rendered
feedback frame, and terminal disposition are one bounded trace. Peer spans
caused by that command reuse its correlation. A command is not considered
settled merely because its handler returned; the orienting success/refusal and
next step must have reached both Activity and a visible frame.

The activity/receipt sheet is the human projection of this model. Structured
export must be optional, local-first, credential-free, bounded, and routed
through an explicit adapter; telemetry must never contain prompt/document
content, authority handles, provider locators, or opaque secrets. Instrumenting
an action never grants permission to retry it or reinterpret uncertainty.

## Interaction law

### Cardinality law

One item is a safe default, not a product ceiling. When an owner contract can
act on more than one domain item, Pantheon must expose N through a bounded
range, batch, or explicit multi-selection. The workflow must show the selected
count and limits, preview the aggregate effect, keep selection separate from
submission, and retain per-item or exact-range receipts. Partial completion and
uncertainty remain visible per child operation; the host never flattens them
into one optimistic success.

The interaction model follows established terminal precedents: Ratatui's
[stateful table example](https://www.ratatui.rs/examples/apps/table/) pairs a
scrollable selection with detail; Gum's
[bounded multi-select](https://github.com/charmbracelet/gum) separates toggling,
limits, and final Enter submission; Lazygit's
[range selection](https://github.com/jesseduffield/lazygit/blob/master/docs/keybindings/Keybindings_en.md)
uses an explicit visual range before a bulk action. Pantheon standardizes these
as arrows for fine movement, PageUp/PageDown for coarse movement, Home/End for
bounds, Space for sparse selection when appropriate, and a separate arm/confirm
phase for effects.

| Workflow class | N disposition |
| --- | --- |
| Suite refresh and hosted-topology drain | Already N: one parent correlation with independently settled peer spans and receipts |
| Hermes disclosure | Bounded 1–16 batch, selected-event list/detail navigation, and a fixed 256-byte preview per event |
| Hermes acknowledgement | Exact cursor range from current next sequence through any observed published sequence |
| Hermes compaction | Exact retained range with fine/coarse/bounds movement and post-receipt projection reconciliation |
| Chronos task submission | Bounded 1–128 row composer backed by one owner-atomic `SubmitGraph`; visible count/current byte bound and no per-task request loop |
| Moira reconciliation | Bounded sparse selection of 1–16 sagas; just-in-time owner challenge per child, exact durable receipt per success, stop-on-unknown, and explicit resume of not-sent children |
| Daimon session control, prompting, and permissions | Bounded explicit local workset of 1–16 exact sessions; sparse selection; fresh connection, execution and idempotency identity per child; challenge-native permission review with a reject-first default and independent option per session; returned session projection per applied child; content-free prompt/challenge envelope evidence; and stop-on-unknown with later children visibly not sent |
| Chronos task cancellation | Bounded marked set on the current 64-row owner page with focused-task fallback; exact V3 receipt per child, owner-revision reobservation between children, stop-on-unknown, and later children visibly not sent; never presented as atomic |
| Chronos task eligibility | The same marked/current workset drives pause and not-before; V5 overview plus exact eligibility is reobserved before every child, unrelated eligibility fields are preserved, each disposition is visible, and uncertainty stops later sends |
| Vellum document editing | Bounded sparse selection of 1–16 documents; literal query/replace preview with exact revisions and match counts; one owner-atomic in-memory generation; exact retained-request recovery; persistence remains explicitly separate |
| Vellum document persistence | Bounded sparse selection of 1–16 documents; exact revision re-observation, independent durable child receipts, stop-on-unknown, exact lookup, and explicit resume; no atomic filesystem-batch claim |
| Aegis epoch revocation | Already semantically N: one owner-atomic epoch operation invalidates the bounded authority set; individual grant actions remain exact and singular |
| Resource and message creation | Singular where the owner contract creates one domain identity; repeated creation is not mislabeled as one atomic batch |

1. **Search first.** `Ctrl-P` finds every action by peer, verb, capability,
   plane, and description. A novice need not memorize keys.
2. **Context accelerates.** Frequent actions receive local keys, but those keys
   and palette entries resolve to the same typed action.
3. **Observe before control.** Guarded actions begin from current peer state and
   show the exact proposal or confirmation document before submission.
4. **One request in flight.** A workflow cannot silently queue a second guarded
   effect while the first result is unresolved.
5. **Receipts stay visible.** Accepted, refused, failed, and uncertain outcomes
   enter a bounded activity timeline. Uncertainty is never restyled as success.
6. **Disabled is informative.** Unsupported actions remain searchable with a
   precise reason: missing peer, missing selector, read-only launch, absent
   authority, or interactive workflow not yet composed.
7. **Editors keep their text.** Global punctuation does not steal ordinary text
   from Vellum. Universal workbench keys use control/function keys.
8. **Small terminals remain usable.** Every overlay is bounded and every core
   operation remains reachable without a mouse or color perception. Mouse
   capture is scoped to the workbench and currently owns only explicit shell
   navigation; peers remain keyboard-driven until their APIs expose pointer
   intents. The
   overview deliberately folds into a compact signal list instead of crushing
   desktop-shaped cards into a short viewport.

## Information architecture

The persistent frame has four layers:

- brand and current surface;
- configured surface tab rail (`[` / `]` sequentially, stable `Ctrl-0`–`Ctrl-9`
  for Overview through Operations, `Ctrl-,` for Configuration, equivalent
  `Alt` numbers, and mnemonic `Alt` shortcuts for every first-class surface);
  clicking a tab or wheeling over the rail provides pointer navigation; at
  narrow widths it preserves identity with numbered glyphs;
- peer-owned content region;
- current outcome plus `Ctrl-P actions`, `F1 help`, `Ctrl-G activity`, and safe
  exit hints.

Overlays never acquire peer authority:

Keyboard behavior crosses one host-owned `BehaviorFacade`. It classifies each
key once using explicit precedence—safe exit, active overlay, global
workbench command, host refresh, surface navigation, then surface-local
workflow/peer input. Overlay and peer handlers consume behaviors rather than
re-recognizing global chords, so a binding cannot silently change meaning as
new tabs or sheets are added. Peer-native translators remain the final
surface-local boundary and therefore retain their own editing semantics.
Palette outcomes are normalized by the same facade into `ExecuteCommand`
behaviors. F1, Ctrl-G, semantic surface shortcuts, Vellum open/save, and their
palette entries therefore converge on one command implementation instead of
maintaining parallel shortcut and palette paths.

Workspace profiles select the closed `pantheon` or `emacs` keybinding contract;
the Configuration sheet shows the effective choice and a change requires the
existing guarded shell restart. The Emacs profile is prefix-stateful and routes
to the same semantic commands as the palette:

| Emacs input | Current semantic command |
| --- | --- |
| `M-x` | Open command minibuffer |
| `C-g` | Keyboard quit: cancel prefix or dismiss the active overlay |
| `C-x C-f` | `vellum.document.open` when Vellum owns the active surface |
| `C-x C-s` | `vellum.save` when Vellum owns the active surface |
| `C-x b` | Open the minibuffer prefiltered to Pantheon surfaces (the shell's buffer-scale navigation unit) |
| `C-x Left`, `C-x Right` | Previous/next Pantheon surface |
| `C-x C-c` | Guarded Pantheon exit |
| `C-h b`, `C-h ?` | Generated binding/action help |
| `C-h l` | Bounded activity/receipt history, analogous to lossage inspection |
| `C-a`, `C-e`, `C-b`, `C-f`, `C-n`, `C-p`, `C-d` | Vellum line/cursor motion and forward delete through Vellum's existing actions |
| `C-s` | Vellum's bounded literal-find prompt; incremental isearch remains a visible owner gap |

In this profile `C-p`, `C-q`, `C-o`, Ctrl/Alt-number and the bracket rail keys
are not stolen for Pantheon palette, exit, file-open or surface navigation.
Core Vellum cursor keys are normalized to Vellum's existing actions; Pantheon
does not implement editing semantics. Unknown `C-x`/`C-h` chords refuse with an
explicit unbound result instead of falling through as text. This is not a claim
of full Emacs parity. Vellum still needs owner-native word motion, incremental
search, mark/region, kill/yank and undo bindings; those must remain Vellum
actions rather than a second editor implementation in Pantheon.

The work-item journey follows that same route. `Shift-N` and its minibuffer
entry both resolve to `workitem.agent.new`; the journey captures only the
bounded text typed into the Chronos form, waits for the task receipt, prepares
the selected local Daimon session's prompt, and stops on refusal or uncertainty.
Agent dispatch retains the ordinary review/arm/confirm guard. Returning to the
board requests a fresh Chronos projection rather than showing optimistic state.
The execution profile freezes one explicit task key/incarnation/attempt across
prompt, permission and reattach; each actual operation still receives fresh
execution and idempotency identities. Configuration shows that task envelope,
harness profile, context/tool counts and budgets, while observation-only
headless output projects the same facts and names unpublished participation
fields instead of inferring them.
The Daimon event timeline is the current conversation surface. A genuinely new
per-task session and comments embedded in a Chronos ticket remain explicit
contract gaps: Pantheon will not infer Daimon's dynamic task binding or invent
a Chronos comment mutation.

- **Minibuffer:** `Ctrl-P` opens a bottom-anchored Emacs-style `M-x` prompt
  without obscuring the active surface. It provides strict command completion,
  C-p/C-n candidate motion, Tab completion, C-g cancellation, cursor/editing
  chords, and M-p/M-n history over the same available/unavailable registry.
- **Workflow sheet:** peer-owned fields, current witnesses, confirmation, and
  one exact submit/recover path.
- **Help:** generated from the same registry, so documentation cannot drift from
  runtime actions.
- **Activity & receipts:** bounded newest-first outcomes with explicit severity
  and uncertainty.

Every command result is phrased as an observable state (`READY`, `QUEUED`,
`OPENED`, `DONE`, `REFUSED`, or `OUTCOME UNKNOWN`), preserves the owner-returned
reason, and includes a concrete `Next:` orientation. Refusal never disappears
with the minibuffer, and uncertainty never becomes a retry suggestion.

The visual language is intentionally calm but not anonymous. A near-black
canvas and raised graphite panels provide continuity; cyan is navigation,
green is settled evidence, amber is guarded attention, red is degradation or
uncertainty, and each surface carries one restrained accent. Rounded containers
and sparse glyphs create rhythm without turning every fact into a box. Like a
good oscilloscope, motion and color must reveal signal rather than decorate
noise: Pantheon uses gauges and tables only for observed facts, never invented
telemetry. The active workspace pane receives a strong focus ring, while peer
components retain their own native information grammar inside it.

## Coherent shell, fit-for-purpose surfaces

Pantheon must not confuse coherence with visual uniformity. The shared shell
owns the interaction laws—action search, focus, help, confirmation grammar,
receipts, recovery, instrumentation, accessibility, and navigation. Each peer
owns the representation that best matches its work and evidence.

| Concern | Native surface grammar to explore | Avoid |
| --- | --- | --- |
| Chronos | Agenda, outline, dependency board, deadlines, and narrow task forms; an Org-like hierarchy may fit parts of this work | Flattening temporal/dependency work into a generic resource table |
| Vellum | Document tree, editor, revision/persistence state, and contextual service actions; outline affordances may fit structured documents | Treating text editing as a dashboard of cards |
| Agora | Room/conversation navigator, message timeline, composer, participants, unread/freshness cues | Forcing conversation into an outline merely for consistency with Chronos |
| Moira | Saga/evidence timeline, reconciliation proposal, authority evidence, and durable receipts | Hiding uncertainty in a generic success/failure list |
| Daimon | Selected-session transcript/evidence, current turn/challenge, and explicit guarded-control sheet | Mixing presence monitoring with provider-control authority |
| Aegis | Authority ledger, grant ancestry/scope, decisions, epochs, and an isolated administration session | Rendering authority as ordinary editable settings |
| Hermes | Stream/cursor timeline, retention window, disclosure, acknowledgement, and compaction boundaries | Losing ordering or exact cursor evidence in generic pagination |
| Tellus | Provenance, source graph, digest/freshness, and disclosure state | Inventing controls before an owner-selected contract exists |
| Operations | Topology and lifecycle state, dependency/start order, health/freshness, drain/restart receipts | Reusing a document or conversation metaphor for process control |

The design test for every surface is therefore two-part: it should feel native
to the concern, and a user who learned one Pantheon workflow should still know
how to discover actions, verify authority, confirm effects, recover uncertainty,
inspect instrumentation, and return home in every other surface.

Chronos and Vellum are first-class full-frame surfaces. Version-one profiles
that explicitly select `workspace` retain the qualified side-by-side legacy
composition, but new profiles should select `chronos` and `vellum` separately.
Configuration is likewise a persistent surface rather than a palette-only
sheet; `Alt-,` opens its bounded redacted launch facts from anywhere.
Agora, Moira, Daimon, Tellus, Aegis, and Hermes also retain distinct
operational landing layers when disconnected: each describes its real
workflows, exact profile connection contract, and discovery path instead of
collapsing into a generic empty-state card. Once attached, the same tab hands
its body to the peer-owned renderer.

## Capability coverage

| Area | Current interactive depth | Next composition step |
| --- | --- | --- |
| Navigation and discovery | Searchable action palette, surface rail, contextual help, direct Agora/Moira detail entry, and Aegis paging | Recent-action ranking and breadcrumbs |
| Observability | Every configured peer component; palette refresh for current/all peers; cross-peer Overview distinguishing unobserved, observing, current-with-age, degraded, busy, and outcome-uncertain channels from actual settlement/control evidence | Add revision drill-down and protocol-derived resource summaries where peers expose them |
| Workspace work | Typed palette entry into Chronos's bounded 1–128 task atomic-graph composer, exact cancellation confirmation, and scheduling surface; guarded create-task → selected-local-agent prompt → conversation → refreshed-board journey; Vellum per-document edit state, owner-authorized Ctrl-O file admission, marked-set owner-atomic query/replace, and bounded independently receipted durable-save worksets | Compose Daimon session-open with dynamic exact Chronos task binding, then add task comments only when Chronos publishes that mutation |
| Moira | Saga inspection/evidence/receipts plus bounded sparse 1–16 selection across every reconciliation disposition: keep-uncertain, fail-terminal, cancel, compensate, effect-bound failed/successful evidence admission, and retry; explicit bounded domain-time entry; exact visible proposals; just-in-time challenges; per-child receipts; stop-on-unknown recovery/resume; instrumentation; durable reopen evidence; exact governed-participation inspection; and independent deliberation presentation with stakeholder standing plus exact pending/settled named-human gate evidence in TUI/headless modes. Only pending gates enter the shared operator agenda, retaining owner identity/revision and no implied authority; gates lacking resolvable decision material are visibly blocked rather than actionable | Publish and verify owner-resolvable decision-packet/rationale references, then add Pantheon's separately selected, command-scoped Aegis-authorized human response control with exact confirmation and lost-reply recovery; compose Dendrite concern routing and independent member execution afterward |
| Daimon | Presence plus a bounded configured 1–16 session workset; sparse selection; phase-aware owner-state candidates that explicitly retain the separate authority requirement; protocol-derived all/conversation/action/decision/usage event filters; exact observed-window counts and input/output usage totals across the complete current workset; multiline prompting, challenge-native per-session permission choices, and exact detached-session reattachment backed by one reusable credential-free execution-policy profile; independently settled prompt/permission/reattach/advance/cancel/close runs with owner-returned projections, post-receipt refresh, stop-on-unknown, retained evidence, exact sent-session recovery inspection without retry, mixed-state blocker diagnostics, and exit protection | Add owner-published cost/time/resource evidence when the protocol exposes it, without importing provider policy |
| Aegis | Bounded authority/grant/decision observation; exact-current-epoch global revocation; and one private bounded managed session for 1–16-scope issue/attenuate, retained ancestry review, and authorization. Exact owner confirmations, arm/confirm, one attempt, uncertainty fencing, redacted facts, instrumentation, and live durable-daemon qualification keep handles inside the owner component | Add owner-defined bulk grant import/reconciliation only if Aegis publishes such a contract |
| Hermes | Ledger, selected stream and cursor observation; 1–16 event disclosure with bounded per-event previews and list/detail navigation; exact selectable acknowledgement and compaction ranges with fine/coarse/bounds movement, owner re-observation, arm/confirm, uncertainty fencing, receipt-to-refreshed-projection traces, and exit protection | Add protocol-derived consumer-safety evidence for compaction without inferring it from aggregate backlog |
| Agora | Conversation-shaped room sidebar and message timeline; bounded revision-bound room creation; multiline message composition with a canonical 0–64 recipient set, owner revision/sequence receipts, one in-flight request, uncertainty fencing, and exact replay | Add member administration and reply/thread workflows only through peer-owned contracts |
| Tellus | Provenance and freshness observation | Remain observation-only until Dendrite selects a disclosure/control contract |
| Configuration | Profile validated before startup; redacted live sheet shows selected surfaces, attached peers, and launch-time Chronos authority; bounded local profile-path editor reuses the typed parser and exact diff planner; explicit arm/confirm preserves current authority flags across process replacement; SHA-256 binds confirmation and replacement startup to the exact preflighted bytes | Add saved/recent profile discovery without scanning the machine or exposing credential paths |
| Operations | Live local-host lifecycle and hosted-peer census; guarded two-step reverse-order topology drain on a worker; retained exact per-peer receipts; pending/outcome-unknown and dirty/indeterminate exit fencing; preflighted restart disconnects peer clients, requires a clean owned-topology drain, then replaces the process and proves the replacement configuration | Add per-peer restart selection only where the typed host transaction can return replacement clients to the running terminal safely |
| Instrumentation | Bounded human activity/receipt stream; content-free action and frame metrics; explicit in-flight/overdue and budget-violation state; bounded operation-stage ledger; shared parent correlation for multi-subsystem work; correlated guarded round trips and reads across all asynchronous peer surfaces; timed Chronos projection and Vellum service paths; instrumented reference performance gate | Add owner-published cross-process correlation where peer protocols support it, protocol-derived projection-age budgets, export, and live full-suite performance qualification |

The palette already lists every qualified guarded route. Routes without an
interactive workflow remain disabled and explain that gap; discovery is not a
claim of authority or completion.

## Delivery sequence

1. Establish workbench chrome, palette, help, activity, navigation, refresh,
   focus, and one real guarded action. **Implemented:** Vellum service save is
   the first guarded vertical slice.
2. Give Chronos and Vellum first-class typed action bridges and workflow forms.
   **In progress:** Chronos atomic task-batch creation, bounded marked-set
   independently receipted cancellation, marked-set eligibility controls, and scheduling entry now reuse its peer-owned forms and launch guards. Vellum now retains
   per-document cursor/dirty/selection state, composes a marked-set literal
   query/replace through one owner-atomic generation, and provides independently
   receipted authenticated durable saves with exact recovery.
3. Compose peer control components in recovery-value order: Moira, Daimon,
   Aegis, Hermes, then Agora. **In progress:** Moira's first exact
   reconciliation disposition and Daimon's exact-state advance/cancel/close
   controls are composed and preserve uncertain-outcome fencing. Keep
   Tellus observation-only.
4. Add configuration inspection, diff/preflight, and local lifecycle operations
   using the already-qualified management substrate. **Implemented:** the
   interactive workbench validates a bounded next-profile path, renders the
   exact restart boundary and change set, explicitly arms, fences unresolved or
   dirty work, drains owned topology, and replaces the process while preserving
   the original authority flags.
5. Polish adaptive layouts, accessibility, mouse affordances, themes, command
   history, and end-to-end live qualification only after the action model is
   complete. **In progress:** the shared shell now has an adaptive named/glyph
   tab rail, direct tab navigation, semantic dark theme, responsive compact
   overview, live-signal table and gauge, focused-pane framing, and cohesive
   palette/help/activity sheets. Mouse affordances and command history remain.
   **Breadth complete:** tmux qualification now drives the real binary across
   every first-class surface and global presentation keybinding; the
   operational gate separately mounts the real Moira, Daimon, Hermes, Aegis,
   and Tellus peers.
