# AI semantic authoring with MNIR

> **Informative, not normative.** This document describes a long-term architectural vision for AI-assisted authoring with MNIR. It does not define MNIR semantics, public API commitments, transport protocols, or conformance requirements. Existing behavior is defined only by the normative documents in [`docs/specification/`](specification/README.md).

## Core principle

> **AI should not need to memorize MNIR.**

An AI authoring system should understand user intent and work through discoverable, typed semantic capabilities. It should primarily manipulate program semantics, not synthesize textual source and hope that a parser reconstructs the intended meaning.

The intended long-term flow is:

```text
User intent
    ↓
AI
    ↓
Capability discovery
    ↓
Semantic inspection
    ↓
Begin MNIR transaction
    ↓
Typed semantic operations
    ↓
Structural enforcement
    ↓
Semantic verification
    ↓
Structured diagnostics
    ↓
AI repair loop
    ↓
Policies / Security / Patterns
    ↓
Commit
```

EasyH is intended primarily as the human-facing representation. It should not be required for an AI to inspect, create, or modify MNIR.

## Architectural relationship

The envisioned responsibilities are distinct:

```text
MNIR Specification
    defines semantics

MNIR Core
    stores canonical semantic Program state

Semantic API
    exposes safe inspection and controlled operations

AI Semantic Authoring
    uses discovery, introspection, and semantic operations

Verifier
    provides machine-readable correctness feedback

Policies / Patterns
    encode reusable engineering constraints and knowledge

Package Ecosystem
    distributes reusable semantic capabilities

Canonical Serialization
    persists MNIR

EasyH
    provides the human-oriented language view
```

No layer in this diagram changes the authority of the normative specification. In particular, AI behavior, presentation names, package metadata, and transport formats do not independently define program semantics.

## Semantic operations

AI tooling should interact with typed operations that express semantic intent. Illustrative operation names include:

```text
CreateModule
CreateFunction
AddParameter
CreateFunctionBody
CreateExpression
SetReturn
```

These names are conceptual examples, not proposed or normative API names. The supported operations will evolve with specified MNIR capabilities.

Semantic operations modify MNIR, but the operations are not themselves the persisted program. The relationship is similar to database commands modifying database state: commands describe controlled changes, while the resulting state is authoritative. For MNIR, the resulting Program revision is the canonical semantic state.

Operations should preserve structural invariants, typed identifier boundaries, transaction isolation, and revision semantics. They should expose failures explicitly rather than leaving partially modified canonical state.

## Transactional authoring workflow

The preferred AI workflow is:

```text
inspect
    ↓
begin transaction
    ↓
apply semantic operations
    ↓
inspect working state
    ↓
verify
    ↓
diagnostics?
    ├── yes → inspect affected semantic IDs → repair → verify again
    └── no  → commit
```

Long-term authoring tools should follow these principles:

- inspect relevant semantic state before modifying it;
- preserve existing semantic identity where possible;
- use controlled mutation instead of unrestricted object mutation;
- inspect the working state as operations accumulate;
- verify before commit;
- use structured diagnostics to guide repairs;
- derive behavior from semantic structure, not presentation names;
- report specification gaps instead of inventing missing semantics.

The precise relationship between verification and an active transaction remains subject to future specification and API design. The diagram expresses the desired workflow, not a commitment to a particular transaction or verifier API.

## Semantic introspection

AI authoring requires read APIs that reveal the semantic neighborhood relevant to a task. Illustrative queries include:

```text
GetModule
GetFunction
GetExpression
ListFunctions
ListParameters
GetExpressionDependencies
FindReferences
GetDiagnostics
```

Future semantic queries may answer higher-level questions such as:

```text
What effects does this Function have?
Why is this Program not verified?
What security classification does this value have?
What can write to this resource?
```

These examples are future vision, not current API commitments. Their underlying concepts must be specified before they become MNIR behavior.

Introspection should support focused retrieval. An AI should be able to request the relevant entities, dependencies, references, and diagnostics without placing a complete MNIR Program dump into model context. Stable semantic identities make that selective retrieval possible.

## Capability discovery

The long-term semantic interface should be machine-discoverable. An AI should be able to ask conceptually:

```text
What operations are supported by this MNIR version?
What operations are valid for this entity?
What verification rule sets are available?
What packages or patterns provide this capability?
```

Discovery results should be machine-readable and should describe typed inputs, outputs, applicability, and relevant semantic constraints. This reduces dependence on memorized prompts, hard-coded operation catalogs, or undocumented conventions.

The capability model should be transport-independent. JSON-RPC, MCP, HTTP, or another protocol could carry such interactions, but this document does not select a transport or define a wire format.

## Verification as the correction loop

`mnir-verify` and structured diagnostics are intended to provide the primary correction loop for AI-authored changes. For example:

```text
AI creates:
    Int32 + Bool

Verifier returns:
    MNIR-DIAG-...

AI:
    inspect affected ExpressionId
    inspect dependencies
    correct semantic structure
    verify again
```

This example is illustrative; the applicable normative specification defines actual diagnostic codes and payloads.

The AI should not be expected to prove every correctness property by itself. MNIR structure should enforce structural invariants, while verifier rule sets, policies, and security rules should provide explicit machine-checkable constraints. Diagnostics should identify semantic subjects and structured facts so repair does not depend on parsing prose messages.

## Patterns, policies, and encoded engineering knowledge

AI systems should express intent and reuse established semantic knowledge instead of manually reproducing expert implementations. Illustrative future capabilities include:

```text
PasswordAuthentication
TransactionalOutbox
AuthenticatedEndpoint
RetryWithBackoff
```

These examples do not imply that the concepts exist in current MNIR or establish their future semantics.

> Established engineering knowledge should be encoded in reusable MNIR patterns, policies, profiles, and packages so that AI and developers do not need to reproduce every implementation detail from memory.

Policies may constrain acceptable structures, while patterns and profiles may package established solutions and organizational expectations. Their actual representation and enforcement require future normative specifications.

## Package-first reuse

> **Prefer discovering and reusing suitable verified or approved packages, patterns, and semantic capabilities over generating equivalent functionality from scratch.**

The intended decision flow is:

```text
Need capability
    ↓
Search available packages / capabilities
    ↓
Suitable approved capability exists?
    ├── yes → import / reuse
    └── no  → implement application-specific semantics
```

Reuse can reduce duplicated work and make review, provenance, compatibility, and security knowledge available to both developers and AI systems. The package vision is described in [`PACKAGE-ECOSYSTEM.md`](PACKAGE-ECOSYSTEM.md).

## Future operational authoring guide

This document describes architecture and principles. It is intentionally distinct from a future operational guide:

```text
AI-SEMANTIC-AUTHORING.md
    long-term architecture and principles

future semantic-authoring-guide
    concrete instructions for agents using a released MNIR API
```

An operational guide should be written only when a sufficiently complete, released semantic API exists. It can then document real capability discovery, transaction, inspection, verification, and repair procedures without turning conceptual examples from this vision into accidental API commitments.
