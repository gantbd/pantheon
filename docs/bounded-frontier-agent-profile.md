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

- component-owned capability IDs such as `tasks.workflow.observe.v9` and
  `tellus.sme-plan.v1`: each exact catalog entry fixes the Pantheon subcommand
  and deployment arguments; the model cannot append a different operation or
  infer a sibling component/control capability;
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
- `moira.task-agent.catalogue.v1` and `moira.task-agent.observe.v1`: separate
  read-only Moira gateways. The deployment fixes the owner socket; the former
  takes no model argument and the latter takes one opaque owner identity and
  journey sequence. Neither admits preview, instantiation, progression, peer
  effects or reconciliation.
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

The catalogue describes independently admissible commands, not a default role
bundle. A deployment may admit any strict subset, and must deny every omitted
command even when another command belongs to the same component.

`pantheon agent-tools` also returns a machine-readable `unavailable` collection.
Agora, Hestia, and Oculus are currently named there with the missing
owner/client boundary and next admissible seam. `aggregate_fallback=false` is
normative: neither an agent nor a launcher may replace a missing component tool
with Pantheon's aggregate headless observation. Aegis authority and human-gate
responses are deliberately absent from this gap list because they are guarded
controls, not autonomous-agent conveniences.

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
      "command": "web",
      "capability": "22222222222222222222222222222222",
      "semantic_capability": "agent.web.fetch.v1",
      "program": "/absolute/pantheon",
      "argument_prefix": ["agent-web-fetch", "/absolute/web-policy.json"],
      "allow_additional_arguments": true
    }
  ],
  "environment_names": [],
  "max_handles": 4,
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
