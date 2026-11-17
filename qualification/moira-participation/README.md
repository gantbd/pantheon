# Moira participation qualification

This nested workspace proves Pantheon's P0 participation projection against
Moira's real Unix daemon and durable SQLite owner. It observes a current
participation through headless and TUI surfaces, drops and reopens the owner,
and separately preserves not-found, authorization-refusal, and unavailable
transport states. Pantheon never copies or mutates the semantic record.

Run:

```sh
sh qualification/moira-participation/qualify.sh
```
