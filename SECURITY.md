# Security policy

This composition root is unreleased. Report suspected vulnerabilities privately
to the repository owner rather than opening a public issue containing workspace
paths, source text, credentials, or service locators.

The workbench is a trusted local application, not an authorization boundary. It
must preserve each peer's identity, capability, revision, persistence, and
uncertainty rules. Focus and layout never grant cross-engine authority, and the
host must not retry or reinterpret an uncertain peer effect.

Before release, review STRIDE and LINDDUN for terminal input/output, service
attachment, credential loading, focus routing, diagnostics, and shutdown; also
qualify dependency provenance and terminal control-sequence handling.
