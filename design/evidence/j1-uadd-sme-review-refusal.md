# J1 UADD Tellus SME review refusal

Date: 2026-11-20

Status: authenticated live request refused; no SME plan, ensemble-fit answer,
stakeholder standing, design admission, acceptance or effect authority

## Bound inputs

- SME-plan request SHA-256:
  `ec15ed2ffa04ccaf88a7d5fe36f5e32eb5dd6d2b82619934a201df0fe3108c28`
- Rune trigger fold SHA-256:
  `14fad3a0af935394c7484e88304d7e2d9636ff76ff7517301808c638b4d31be8`
- Axon concern lattice SHA-256:
  `2ff1bddf17d36f1e202e68073e9df8298a34b8f8b453a2e6ac1f00758d46d508`
- adversarial-tester grounding SHA-256:
  `f4e34ad0f7e38a524052e20a21943a5adab8ad378d6de2caeadfbdd79b2528f0`

## Live boundary exercised

An isolated loopback Dendrite owner loaded the exact real Rune fold and Axon
lattice. Its own Auth process issued a short-lived opaque token to a temporary
mode-`0600` file. Pantheon's shipped `tellus-sme-plan` command sent the exact
request through the authenticated `tellus.sme-plan.v1` boundary. The shared
Dendrite process and its credentials were not read, stopped or mutated.

The first sandboxed client call was not sent because loopback networking was
denied. The same exact read-only command was then run with loopback permission;
Tellus returned HTTP 503 and Pantheon correctly projected a typed `refused`
result with no roles. A direct authenticated read of the same owner endpoint
returned only its content-safe `sme_plan_unavailable` reason. No parser,
authentication rule or response contract was weakened to obtain more detail.

The canonical deterministic tools isolate the owner cause:

```text
architecture-flow-review: architecture, network, identity, data
adversarial-security-validation: adversarial-tester
user-acceptance-review: user-experience
human-impact-decision: humanism
claiming-completion: live-verification

summary: 5 tag(s) checked, 8 unique role(s) fold in: architecture, network,
identity, data, adversarial-tester, user-experience, humanism, live-verification

lattice_check: role 'rune:adversarial-tester'
(../rune/sme/adversarial-tester.grounding.rune) has no (concern …) clause —
not registered in the orthogonality system
```

The disposable owner was stopped and its generated token, port file and
fixture were removed.

## Resolution boundary

The request is correctly scoped at the existing eight-role ceiling. Removing
`adversarial-security-validation`, substituting a less relevant role, raising a
limit or inventing a concern pair merely to obtain a plan would weaken the
review. The Rune grounding owner and Axon lattice owner must jointly register
the adversarial-tester's real concern, or explicitly decide that the role is
ineligible and revise the review design with preserved dissent. After that
owner change is qualified, this exact plan must be rerun against the new source
digests.

Even a successful future SME plan remains advisory. Moira must separately
admit an equitable, independently attributable stakeholder/owner ensemble
beginning with “do we have the right ensemble?”, and the named human authority
must review the exact Rune-author request before candidate bytes may exist.
