# J1 UADD Tellus SME review refusal and resolution

Date: 2026-11-20

Status: initial authenticated request refused; owner taxonomy repair and live
SME-plan routing qualified; no ensemble-fit answer, stakeholder standing,
design admission, acceptance or effect authority

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

## Qualified owner resolution

The owner audit found that the canonical UADD path selects four previously
unregistered roles, not only the first role named by the refusal. Their exact
scope-derived registrations are:

- `security`: `application-security / threat-modeling`;
- `security.stride`: `threat-modeling / stride-categorization`;
- `adversarial-tester`: `security-testing / adversarial-validation`; and
- `live-verification`: `empirical-verification / production-reality`.

The three security roles intentionally share Axon's `risk-protection` facet;
they are complementary parts of one security argument, not independent
convergence. Live verification shares `epistemic-discipline` with Rigor and
Auditor. The focused Axon gate requires all five negative orthogonality facts
and still requires the established independent cross-cutting pairs to pass.

Revised identities:

- Axon concern lattice SHA-256:
  `6408299fb38ff953718317d1ae682dbb195ce7f5c78a1beb4559b5aeb0dfed53`;
- security grounding SHA-256:
  `38318e39003f6ee8f459fdfa487f28274a3f3352604020855dbdd31101a16083`;
- STRIDE grounding SHA-256:
  `fdd171c22e995525f1b5be8ff167a2e8f007fb87c72f326d20530d605c5e6313`;
- adversarial-tester grounding SHA-256:
  `35b8982fcd305c3749355917b29dac5393feae326b2cf8f6fa95bdc64215a322`;
- live-verification grounding SHA-256:
  `474f745c3da80663e07a61da2e8b0101bceb941de107426856d04bfc3d1698e7`;
- exact eight-role request SHA-256 remains
  `ec15ed2ffa04ccaf88a7d5fe36f5e32eb5dd6d2b82619934a201df0fe3108c28`;
  and
- state-local security request SHA-256:
  `36309269f66d8cfb45e976b7ddffd222eb9ba78cb4326e0fef11084b586bad6e`.

`tests/live/j1_uadd_sme_plan.sh` now cold-loads the real sources into an
isolated authenticated Dendrite owner, verifies the temporary token is mode
`0600`, and invokes Pantheon's shipped client twice. The unchanged J1 request
returns all eight roles with only User Experience/Humanism correlated. The
security-stage request returns security, STRIDE, adversarial-tester and live
verification with exactly the three security-role correlations. Both plans are
honestly `review_required`, retain “do we have the right ensemble?”, and state
that SME routing is not stakeholder equity, authority, approval or proof.
