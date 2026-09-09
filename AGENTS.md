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

## Direct external dependency governance

An agent MUST NOT add, remove, upgrade, downgrade, or change features of a
direct external dependency unless explicitly mandated by the task or an
authoritative specification.

If a direct external dependency change appears technically necessary but is
not explicitly authorized, the agent MUST report:

```text
DEPENDENCY DECISION REQUIRED
Dependency:
Purpose:
Scope:
Alternatives:
Security/portability implications:
```

The agent MUST then stop before making that dependency change. Transitive
`Cargo.lock` changes caused by an already-authorized direct dependency change
do not require separate approval, but MUST be reported.

## Documentation freshness before commits

Before every Git commit, analyze whether the following documents accurately
describe the repository's complete current state:

- `README.md`;
- `docs/ROADMAP.md`;
- `docs/specification/README.md`.

This review MUST consider the repository as a whole, including all implemented
capabilities and specifications, and MUST NOT be limited to the changes included
in the intended commit. Update any document that is stale or misleading about
implementation status, supported capabilities, repository structure,
development milestones, or the specification document inventory, regardless
of which earlier change caused it to become stale. Do not create the commit
until all three documents are up to date. If no update is required, leave the
documents unchanged.

This review does not authorize changing normative specification semantics to
match an implementation. Normative specification documents remain subject to
the authority and scope rules above.

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
