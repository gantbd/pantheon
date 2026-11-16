# Publication gate

Pantheon is source-ready but its public qualification is dependency-closed.
Public CI must check out the exact recorded revisions of every peer source used
by `Cargo.lock`, including Agora and Tellus. Do not replace those sources with
stubs, remove peer surfaces, or weaken the source receipt merely to make an
isolated checkout green.

Publish after the Agora suite is public and qualified, Tellus has a public
qualified revision, and `scripts/qualify-source.py` has recorded the resulting
exact closure in `release/source.json`.

As of the rename qualification, the local closure is intentionally blocked:
Chronos selects `rustix` 1.1.6 while the concurrently remediated Vellum source
still pins 1.1.4. Resolve that at the owning peer before qualifying Pantheon;
Pantheon must not patch around incompatible peer locks.
