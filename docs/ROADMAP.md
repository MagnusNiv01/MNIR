# MNIR development roadmap

> **Informative, not normative.** This roadmap describes intended development direction and milestones. It does not define MNIR semantics. Normative behavior is defined only by the documents in [`docs/specification/`](specification/README.md).

## Versioning model

MNIR uses separate versioning tracks for the overall project and for individual specification documents:

- The MNIR project is currently in the `0.0.x` development series.
- `MNIR 0.1.0` is the first planned integrated end-to-end release.
- Each specification document has its own independent version, such as Program Model 0.1.
- Specification versions do not need to match the MNIR product or release version.
- A future MNIR release may therefore combine different versions of individual specifications.

Current Cargo package versions are implementation metadata and are not renumbered by this roadmap.

## MNIR 0.1.0 release goal

MNIR 0.1.0 is intended to be the first release that demonstrates the core MNIR concept end-to-end:

```text
AI / Semantic API
        ↓
       MNIR
        ↓
Semantic Verification
        ↓
Canonical Serialization
        ↓
       Git
        ↓
Deserialize MNIR
        ↓
      EasyH
        ↓
Human-readable representation
```

The release is a conformance and architecture milestone, not a production-ready executable programming-language runtime. It is intended to demonstrate that MNIR can:

- represent a meaningful program semantically;
- be manipulated without textual source code;
- perform semantic verification and provide machine-readable diagnostics;
- express initial security and system-engineering semantics;
- be serialized canonically, stored in Git, and deserialized without semantic loss;
- render human-readable EasyH and parse it back into MNIR;
- support semantic round-trip;
- support AI-oriented semantic mutation;
- provide an initial semantic diff.

Detailed behavior for planned capabilities will be established by future normative specifications before implementation.

## A. Semantic Core — complete

Completed specification increments:

- Program Model 0.1
- Type System Foundations 0.1
- Functions and Parameters 0.1
- Expressions and Basic Function Bodies 0.1
- Arithmetic Expressions 0.1
- Semantic Verification and Diagnostics 0.1

Together they establish:

- Program, Module, Function, and Parameter;
- stable semantic identities;
- revisions and transactions;
- intrinsic types and Function signatures;
- Function bodies and Expressions;
- arithmetic;
- structural validity;
- semantic verification and machine-readable diagnostics;
- revision-bound `VerifiedProgram` evidence.

## B. Basic Programming Model

Planned sequence:

1. Comparison Expressions
2. Conditional Control Flow and an initial control-flow graph
3. Local Values and Bindings
4. Function Calls

The milestone is the ability to represent non-trivial multi-Function logic with conditions and local computation.

For example, MNIR should eventually be able to represent a program conceptually equivalent to:

```text
max(a: Int32, b: Int32) -> Int32 {
    if a > b {
        return a
    } else {
        return b
    }
}
```

This example is illustrative pseudocode, not normative EasyH syntax.

## C. Safe Domain Model

Planned work includes:

- domain-defined types;
- `TypeId`;
- initial `Text` and `Bytes` support where required;
- explicit semantic distinction between domain types that share an underlying representation.

Illustrative domain types include:

```text
CustomerId
InvoiceId
Percentage
UserName
EmailAddress
```

The design goal is explicit domain meaning. Domain semantics are not inferred from names.

## D. Security Foundations

An initial security model is planned so that future specifications can express concepts such as:

```text
Secret<T>
Untrusted<T>
Validated<T>
Credential<T>
PersonalData<T>
PasswordInput
PasswordHash
```

High-level goals:

- represent security semantics explicitly;
- avoid inferring security from variable, field, or type names;
- make unsafe information flows difficult to represent;
- build security verification on `mnir-verify`.

This roadmap does not define the semantics of these concepts.

## E. Effects and Contracts

### Effects

Initial effect representation is planned for concepts such as:

```text
database.write
network.http
logging
```

The goal is to make side effects explicit and machine-verifiable.

### Contracts

Initial contract representation is planned for concepts such as:

```text
requires
ensures
invariant
```

MNIR 0.1.0 does not need full theorem proving or SMT integration. The milestone is first-class representation with useful initial verification.

## F. Policies, Profiles, and Patterns

An initial system-engineering layer is planned around:

```text
Policy
Profile
Pattern
```

Its intended purpose is to:

- encode established engineering and security practices;
- let AI systems and developers express intent without manually rebuilding every pattern;
- enable organization-specific policies in later releases.

Illustrative future concepts include:

```text
SecureService
NoSecretsInLogs
TransactionalOutbox
```

Their normative behavior is intentionally not defined here. One or two working patterns are sufficient for MNIR 0.1.0 to demonstrate the architecture.

## G. Persistence and Git

Canonical MNIR serialization is planned with these goals:

- deterministic serialization;
- semantic round-trip;
- a versioned format;
- stable semantic IDs;
- preservation of identity allocation history where required;
- a Git-friendly representation;
- independence from Rust `HashMap` iteration order;
- canonical representation of equivalent MNIR state.

The intended flow is:

```text
MNIR in memory
    ↓
canonical serialization
    ↓
*.mnir
    ↓
Git
```

Serialized MNIR is intended to become the canonical persisted program representation. EasyH is a human interface, not the source of truth.

## H. AI and Tooling Interface

An explicit semantic mutation interface is planned for AI systems and other tools. Conceptual operations may include:

```text
CreateModule
CreateFunction
AddParameter
CreateExpression
SetReturn
```

These are examples, not normative API names.

The intended workflow is:

```text
AI
 ↓
semantic operations
 ↓
MNIR transaction
 ↓
verification
 ↓
commit
```

AI tooling should not need to generate EasyH source to create or modify an MNIR program.

## I. Semantic Diff

An initial semantic diff capability is planned, conceptually:

```text
mnir diff revision-a revision-b
```

The goal is to describe changes through stable semantic identities rather than only text-line differences. Potential categories include:

- added or removed Functions;
- changed signatures;
- changed Expressions;
- changed contracts;
- changed security classifications;
- changed policies.

The exact CLI and output format remain undefined.

## J. EasyH Human Interface

Planned sequence:

1. EasyH renderer
2. EasyH parser

### Renderer

```text
MNIR
 ↓
EasyH
```

EasyH is intended to provide a complete human-readable representation of supported general-purpose MNIR semantics.

### Parser

```text
EasyH
 ↓
MNIR
```

The important round-trip objective is semantic:

```text
MNIR
 ↓ render
EasyH
 ↓ parse
MNIR'

semantic_equivalent(MNIR, MNIR') == true
```

Textual identity is not required.

## K. MNIR 0.1.0 Conformance Demo

The integrated milestone is an end-to-end demonstration of:

```text
AI
 ↓
semantic API
 ↓
MNIR
 ↓
verify
 ↓
canonical serialize
 ↓
Git
 ↓
deserialize
 ↓
verify
 ↓
render EasyH
 ↓
parse EasyH
 ↓
MNIR
```

The demo will use a small but meaningful program exercising several core ideas:

- Functions;
- control flow;
- arithmetic;
- domain types;
- security classification;
- effects;
- contracts;
- at least one policy or pattern.

The exact demo application will be chosen later.

## Explicitly outside MNIR 0.1.0

The following capabilities may be considered in later releases but are not required for the first integrated release:

- production-ready runtime;
- WASM backend;
- LLVM backend;
- native code generation;
- optimizer;
- async/await;
- threads and advanced concurrency;
- package manager;
- LSP or IDE integration;
- debugger;
- garbage collector;
- full generics system;
- full trait or interface system;
- advanced collections;
- full exception system;
- complete pattern library;
- advanced theorem proving;
- production HTTP or database runtime;
- production authentication implementation.

This release boundary does not imply that these capabilities will never be developed.

## Current status

| Area | Status |
| --- | --- |
| Program Model | Done |
| Type System Foundations | Done |
| Functions and Parameters | Done |
| Expressions and Basic Function Bodies | Done |
| Arithmetic Expressions | Done |
| Semantic Verification and Diagnostics | Done |
| Comparison Expressions | Next |
| Conditional Control Flow | Planned |
| Local Values / Bindings | Planned |
| Function Calls | Planned |
| Domain Types | Planned |
| Security Foundations | Planned |
| Effects | Planned |
| Contracts | Planned |
| Policies / Profiles / Patterns | Planned |
| Canonical Serialization | Planned |
| Semantic Mutation API | Planned |
| Semantic Diff | Planned |
| EasyH Renderer | Planned |
| EasyH Parser | Planned |
| MNIR 0.1.0 Conformance Demo | Planned |
