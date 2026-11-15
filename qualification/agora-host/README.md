# Agora owner qualification

This nested Cargo workspace qualifies `NativeHost::start_agora` against the
real Agora local daemon and public Unix client. Its authority port,
authentication, and clock are explicitly test-only deployment inputs.

The separate lock closure is deliberate. Workbench's default CLI enables its
native Chronos owner, whose current standalone graph selects a vendored
`keystone-storage`. Agora selects the sibling Keystone package with the same
name and version, and Cargo refuses those identities in one lockfile. This
qualifier disables the optional native-Chronos owner so Agora's lifecycle can
be tested without falsely claiming that the all-three deployment graph builds.

Run:

```sh
cargo test --manifest-path qualification/agora-host/Cargo.toml --all-targets --locked --offline
```
