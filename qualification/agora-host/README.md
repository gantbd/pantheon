# Agora owner qualification

This nested Cargo workspace qualifies one `NativeHost` process owning Chronos,
Vellum, and a real Agora local daemon while serving Agora's public Unix client.
Its Agora authority port, authentication, and clock are explicitly test-only
deployment inputs. While all owners are live it invokes Workbench's reusable
headless aggregation API and requires current Agora, Chronos, and Vellum
observations. It then renders Agora's real room and message projections through
the reusable component and Workbench shell before reverse shutdown.

A second scenario starts Agora's dedicated composition profile through the
same `NativeHost` lifecycle seam and returns the concrete public
`ScheduledLocalClientV1`. It publishes one scheduled message and observes the
exact retained acceptance through fresh actor-bound sessions before clean
shutdown. The scenario proves that standalone and Workbench composition can
select the same public client; its accepting authority, authentication, and
clock remain explicitly test-only and establish no production grant policy.

The separate lock closure is deliberate. Workbench's default CLI enables its
native Chronos owner, whose current standalone graph selects a vendored
`keystone-storage`. Agora selects the sibling Keystone package with the same
name and version, and Cargo refuses those identities in one lockfile. This
qualifier derives both local paths and supplies a narrowly scoped Cargo source
patch at build time. The resulting single lockfile and executable use the
shared package identity without changing either peer's standalone source
policy or committing a machine-specific absolute path.

Run:

```sh
sh qualification/agora-host/qualify.sh
```
