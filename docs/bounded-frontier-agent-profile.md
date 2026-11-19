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

Example Daimon terminal fragment (identifiers and paths are deployment-owned):

```json
{
  "capability": "00000000000000000000000000000000",
  "boundary_program": "/absolute/daimon-process-boundary",
  "commands": [
    {
      "command": "pantheon-status",
      "capability": "11111111111111111111111111111111",
      "program": "/absolute/pantheon",
      "argument_prefix": ["headless", "--workspace-profile", "/absolute/profile.json"],
      "allow_additional_arguments": false
    },
    {
      "command": "web",
      "capability": "22222222222222222222222222222222",
      "program": "/absolute/pantheon",
      "argument_prefix": ["agent-web-fetch", "/absolute/web-policy.json"],
      "allow_additional_arguments": true
    }
  ],
  "environment_names": [],
  "max_handles": 2,
  "max_output_bytes": 1048576
}
```

The compatibility scope capability must still be a valid configured tool but
does not authorize either command when command-specific capabilities are
present. Deployments retain the mapping from each opaque Aegis/Daimon tool ID
to its stable component capability ID; wildcard peer names and aggregate
`pantheon` grants are invalid profile shapes.

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
