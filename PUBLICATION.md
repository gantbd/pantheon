# Public qualification gate

Pantheon's source repository is public, but its qualification is
dependency-closed.
Public CI must check out the exact recorded revisions of every peer source used
by `Cargo.lock`, including Agora and Tellus. Do not replace those sources with
stubs, remove peer surfaces, or weaken the source receipt merely to make an
isolated checkout green.

Do not claim a qualified release until the Agora suite is public and qualified,
every pinned peer revision is reachable, and `scripts/qualify-source.py` has
recorded the resulting exact closure in `release/source.json`.

The public source receipt records the dependency-closed revisions qualified by
Pantheon. Any peer advance must update that receipt and the matching immutable
CI checkout before the composition is described as qualified again.
