<p align="left">
  <img src="assets/mnir-icon.png" alt="MNIR icon" width="180">
</p>

# MNIR

MNIR stands for **Magnus Nivinger Intermediate Representation**. The project is experimental and in its earliest design phase; it is not yet a usable programming language.

**Implemented through Conditional Control Flow 0.1**

MNIR is intended to become a canonical semantic representation of programs. Future tools and human-oriented languages may project from or compile to that representation. EasyH is planned as a human-readable presentation/frontend language, not as the canonical representation.

Program Model 0.1 is implemented as the foundation for Program and Module identity, revisions, snapshots, forks, presentation metadata, and controlled mutation. Type System Foundations 0.1 adds the closed intrinsic type set `Int32`, `Int64`, `Bool`, and `Unit`. Functions and Parameters 0.1 adds Functions and ordered Parameters. Expressions and Basic Function Bodies 0.1 adds optional bodies, intrinsic literals, Parameter references, and Return terminators. Arithmetic Expressions 0.1 adds checked `Add`, `Subtract`, `Multiply`, `Divide`, and `Remainder` semantics as non-evaluated Expression structures. Comparison Expressions 0.1 adds equality and signed-integer ordering comparisons. Conditional Control Flow 0.1 extends Function bodies to finite acyclic multi-Block control-flow graphs with entry Blocks and Branch terminators. Semantic verification rule sets V0.1 through V0.3 provide revision-bound verification evidence and machine-readable Expression, Return, and Branch diagnostics. No evaluator, loops, local bindings, Function invocation, EasyH support, or usable end-user tooling has been implemented. Development proceeds from the normative specification, and implementation must not silently fill in unspecified behavior.

## Vision and roadmap

- [Development roadmap](docs/ROADMAP.md) — planned milestones toward MNIR 0.1.0.
- [AI semantic authoring](docs/AI-SEMANTIC-AUTHORING.md) — long-term AI authoring architecture and principles.
- [Package ecosystem](docs/PACKAGE-ECOSYSTEM.md) — long-term package, dependency, and reusable-capability vision.

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
  mnir-core/    Canonical Program, type, Expression, and acyclic CFG model
  mnir-verify/  Semantic verification V0.1–V0.3 and machine-readable diagnostics
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

## License

MNIR is licensed under the [Apache License 2.0](LICENSE).
