# MNIR specification

Documents in this directory form the MNIR specification. Specification documents may contain normative `MNIR-*` rules that define what conforming implementations must do. Implementation follows those rules and must not silently define missing semantics.

Normative requirements use **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY**. Future rules should receive stable identifiers such as `MNIR-CORE-001`, `MNIR-TYPE-001`, `MNIR-SEC-001`, or `MNIR-PATTERN-001`. Tests should reference the corresponding rule identifier where practical.

If required behavior is absent, ambiguous, or contradictory, report a `SPECIFICATION GAP` using the process in the root `AGENTS.md`. Do not automatically resolve the gap in implementation.

## Documents

- [`00-introduction.md`](00-introduction.md) — Project principles and foundational terminology.
- [`01-program-model.md`](01-program-model.md) — Normative Program Model 0.1 specification covering Program, Module, identity, revisions, controlled mutation, structural validity, and presentation metadata.
- [`02-type-system-foundations.md`](02-type-system-foundations.md) — Normative Type System Foundations 0.1 specification defining the initial intrinsic type set and its foundational semantics.

Later specification increments will define additional MNIR semantics.
