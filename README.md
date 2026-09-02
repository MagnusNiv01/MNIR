# MNIR

MNIR stands for **Magnus Nivinger Intermediate Representation**. The project is experimental and in its earliest design phase; it is not yet a usable programming language.

**Program Model 0.1: DONE**

MNIR is intended to become a canonical semantic representation of programs. Future tools and human-oriented languages may project from or compile to that representation. EasyH is planned as a human-readable presentation/frontend language, not as the canonical representation.

Program Model 0.1 is implemented as the foundation for Program and Module identity, revisions, snapshots, forks, presentation metadata, and controlled mutation. Type System Foundations 0.1 adds the closed intrinsic type set `Int32`, `Int64`, `Bool`, and `Unit`. Functions and Parameters 0.1 adds Functions and ordered Parameters. Expressions and Basic Function Bodies 0.1 adds optional single-Block bodies, intrinsic literals, Parameter references, and Return terminators. Arithmetic Expressions 0.1 adds checked `Add`, `Subtract`, `Multiply`, `Divide`, and `Remainder` semantics as non-evaluated Expression structures. Semantic Verification and Diagnostics 0.1 adds revision-bound verification evidence and machine-readable arithmetic and Return type diagnostics. No evaluator, general control flow, invocation, EasyH support, or usable end-user tooling has been implemented. Development proceeds from the normative specification, and implementation must not silently fill in unspecified behavior.

## Development model

```text
Design decision
      ↓
Normative MNIR specification
      ↓
Acceptance criteria
      ↓
Implementation
      ↓
Tests
      ↓
Verification
      ↓
Next specification increment
```

The specification is authoritative. Normative requirements use **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY**, and future rules receive stable identifiers such as `MNIR-CORE-001` or `MNIR-TYPE-001`. Tests should reference applicable rule identifiers where practical. Undefined or contradictory behavior is reported as a `SPECIFICATION GAP`, not selected by the implementation.

## Repository structure

```text
crates/
  mnir-core/    Program Model, intrinsic types, Functions, and basic bodies
  mnir-verify/  Semantic verification and machine-readable diagnostics
  easyh-render/ Future deterministic EasyH renderer
  mnir-cli/     Future command-line development tools
docs/
  specification/ Normative MNIR specification
  rationale/     Non-normative explanations of design decisions
  adr/           Architecture decision records
tests/            Future repository-level acceptance fixtures and tests
```

The intended dependency direction is:

```text
mnir-cli ───────→ mnir-verify ───→ mnir-core
    │                                  ↑
    └──────────→ easyh-render ─────────┘
```

`mnir-core` never depends on EasyH or on the verification layer and should remain independent of execution backends.

## Development commands

With stable Rust installed:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace
cargo test --workspace
```
