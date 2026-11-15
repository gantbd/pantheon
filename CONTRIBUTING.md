# Contributing

Keep this repository a product composition root. It may own terminal lifecycle,
layout, focus, and routing, but it must not absorb Chronos scheduling semantics,
Vellum editing semantics, or either peer's persistence authority.

Required local gate:

```sh
cargo fmt --check
cargo test --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline
```

Keep failures content-safe, bounds explicit, and uncertain outcomes typed. Do
not publish, configure remotes, or add ambient branch dependencies as part of a
source change.
