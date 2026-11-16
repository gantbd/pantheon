# Selected local release

Status: qualified unsigned local artifact for `aarch64-apple-darwin` on the
recorded Darwin 25.6.0 arm64 host. This is not a remote, registry, signed,
notarized, universal-macOS, Linux, Windows, or GUI distribution claim.

## Release unit

The release unit is the `pantheon` executable plus its operator profile
example, source receipt, Pantheon license texts, security and change records,
an exact selected-target package/license inventory, available third-party legal
texts and their package mapping, and a SHA-256 file manifest. The
source qualifier reconstructs Pantheon and every direct peer from the full Git
commits in `release/source.json`, separately seeds the lock-pinned Keystone Git
identity from its exact declared object, switches Cargo offline, tests the
archive closure, and builds the optimized executable from that closure.

Run the immutable candidate gate with a full Pantheon commit:

```text
python3 scripts/qualify-source.py FULL_COMMIT --local-release \
  --artifact /tmp/pantheon-aarch64-apple-darwin.tar.gz
```

The output receipt names the artifact digest, target, qualified host, reviewed
package count, profile schema, and replacement result. The artifact is unsigned
and is suitable only for deliberate local installation on the selected host.

## Install and replace

Extract the archive into an operator-owned directory, verify `SHA256SUMS`, and
place `bin/pantheon` at the desired local executable path. Installation
does not create services, credentials, peer data, profiles, shell startup
entries, or filesystem authority. Those remain deployment inputs.

Replacement is an atomic binary-file operation performed by the deployment.
The gate performs an initial install and same-version replacement, runs the
installed binary's profile check after both operations, and proves that the
external operator profile is byte-identical afterward. Keep the prior binary
until the replacement passes its profile check so rollback is a file rename,
not a peer-state migration.

## Upgrade and migration boundary

This is the initial supported Pantheon artifact, so there is no earlier
released Pantheon state schema to migrate. Pantheon owns no durable engine
state: Chronos, Vellum, Agora, Moira, Daimon, Hermes, Aegis, and Dendrite retain
their own storage, migration, authentication, and recovery contracts.

Pantheon owns only the versioned operator profile contract. Schema version 1
is accepted without rewriting it; an unknown schema is refused before peer
connection. A future release that changes the profile schema must add an
explicit old-to-new migration and rollback fixture before extending this claim.

## Legal and support boundary

Every package in the lock-selected Cargo graph must carry one of the exact
reviewed SPDX expressions enforced by `scripts/qualify-source.py`. The artifact
records package name, version, expression, and source identity in
`THIRD_PARTY_LICENSES.tsv`. `THIRD_PARTY_NOTICES.tsv` maps every selected-target
package to its bundled legal text. Internal path packages declared under the
dual license but carrying no nearer notice use the bundled Apache-2.0 text as
the selected distribution license. A new expression or a dependency without a
distributable legal text fails qualification.

The selected support boundary is local Apple-silicon macOS on the exact host
profile above. Remote source reachability, crates.io publication, automatic
updates, service-manager installation, code signing, notarization, and wider
platform support remain unavailable.
