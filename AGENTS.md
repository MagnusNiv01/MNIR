# Guidance for coding agents

This repository uses specification-driven development. These instructions apply to the entire repository.

## Authority and scope

- The normative MNIR specification in `docs/specification/` is authoritative.
- Do not invent unspecified MNIR semantics.
- When required behavior is undefined or contradictory, report a **SPECIFICATION GAP** and do not resolve it automatically.
- Do not expand work beyond the requested specification section.
- Do not introduce speculative abstractions for functionality that has not been specified.
- Prefer small, explicit, testable changes.
- Every implemented normative **MUST** or **MUST NOT** requires tests where technically applicable.
- Never modify the normative specification merely to match an implementation bug.
- If implementation and specification disagree, correct the implementation unless the task explicitly changes the specification.
- Do not create backward-compatibility behavior unless the specification requires it.

## Architecture boundaries

- Preserve the crate dependency boundaries documented in the root `README.md` and crate documentation.
- `mnir-core` must never depend on EasyH or on `mnir-verify`.
- `mnir-core` should remain independent of concrete execution backends.
- Human-readable source code is a presentation or frontend form; it is not intended to become the canonical program representation.
- Avoid dependencies unless there is a clear technical reason.

## Safety and quality

- Security checks must not be weakened merely to make tests pass.
- Avoid unsafe Rust unless an approved specification rule or architecture decision record explicitly requires it.
- Do not suppress compiler warnings or Clippy findings without documented justification.
- Write code and code comments in English.
- Documentation may be written in English unless a future repository convention states otherwise.
- Before finishing a task, run formatting, linting, compilation, and tests:

  ```text
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo check --workspace
  cargo test --workspace
  ```

## Reporting a specification gap

Stop the affected implementation work and report the gap using this form:

```text
SPECIFICATION GAP

Location: <specification section or rule>

Problem: <what is undefined or contradictory>

Required decision: <what needs to be decided before implementation can continue>
```

Continue only with work that does not depend on the missing decision. Do not choose a behavior, edit the specification, or encode a temporary semantic assumption unless the task explicitly authorizes that decision.
