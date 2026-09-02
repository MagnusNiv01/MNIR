# MNIR package ecosystem

> **Informative, not normative.** This document describes a long-term vision for reusable MNIR functionality. It does not specify package representation, identifiers, manifests, resolution rules, registry protocols, trust mechanisms, or MNIR semantics. Those details require future normative specifications.

## Core hierarchy

The conceptual distinction is:

```text
Program
    contains Modules

Package
    distributable reusable unit
    may contain multiple Modules

Registry
    discovery and distribution system for Packages
```

`Program` and `Module` already have meanings in the normative MNIR specification. `Package` and `Registry` are future ecosystem concepts whose exact representation is not yet specified.

A Package is intended to group reusable semantic contents for distribution and dependency management. It should not become a competing program model or redefine Module ownership through documentation alone.

## Ecosystem goals

A future package and dependency system should support:

- reusable functionality;
- semantic dependencies;
- stable package identity;
- versioning;
- dependency resolution;
- dependency locking;
- integrity verification;
- package discovery;
- local and Git dependencies;
- future registry distribution;
- AI-oriented capability discovery.

These are design goals, not current conformance requirements. Their detailed semantics, failure behavior, and data representations remain open.

## Reusable semantic contents

MNIR Packages should eventually be able to export more than Functions. Potential export categories include:

```text
Functions
Domain Types
Security Types
Effects
Contracts
Policies
Profiles
Patterns
```

Not all of these concepts currently exist in MNIR. The list illustrates the broader goal: packages should distribute reusable semantics and established engineering knowledge, not only executable code.

Package exports should remain semantic entities. Human-readable names may help discovery and presentation, but they should not replace stable semantic identity or silently define compatibility.

## Security and reviewed functionality

Packages are important to safe AI authoring because they can make reviewed capabilities preferable to newly improvised implementations.

Instead of:

```text
AI invents password storage
```

the preferred future flow is:

```text
discover approved password capability
    ↓
add dependency
    ↓
reuse reviewed semantic implementation / pattern
```

Illustrative future capabilities include:

```text
password authentication
OAuth
JWT validation
cryptography
authorization
audit logging
transactional outbox
retry policies
```

These examples neither define those capabilities nor imply that they exist today. Future security and policy specifications must define what review metadata means and which guarantees a package may claim.

## Package identity and semantic references

Imported semantics should ultimately resolve to stable semantic identities instead of remaining string-based source references. Conceptually:

```text
Package identity
    ↓
Module identity
    ↓
exported Function / Type / Pattern identity
```

Stable identity can support dependency resolution, semantic inspection, cross-package references, diffing, provenance, and AI capability discovery. This document intentionally does not define identifier formats, global uniqueness rules, import identity behavior, or reference encodings.

## Versioning and semantic compatibility

A SemVer-style package version such as:

```text
1.2.3
```

is a likely design direction, and dependency constraints may eventually express acceptable versions. This is not yet a normative decision.

MNIR may eventually reason about semantic compatibility in addition to source compatibility. Semantic diff tooling could identify changes such as:

```text
Function return type:
    Int32 → Int64

Policy:
    new requirement added
```

Such facts could inform breaking-change analysis more precisely than text-only comparison. Rules for classifying compatibility, however, must be specified rather than inferred from these examples.

## Package manifests and dependency locks

The ecosystem will need concepts equivalent to:

```text
package manifest
dependency declarations
lock file
```

A manifest could describe package identity, version, exports, dependencies, compatibility, and package metadata. A lock could record resolved dependency identities, versions, and integrity information for reproducible use.

Illustrative filenames might be:

```text
mnir-package.toml
mnir.lock
```

These names and any implied syntax are non-normative placeholders. No final file format is selected here.

## Package sources and the MNIR 0.1.0 direction

A complete public package registry is not required for MNIR 0.1.0. A first implementation may be limited to sources such as:

```text
local package
Git package
```

A minimal workflow can still prove package identity, dependency declaration, resolution, locking, integrity, semantic references, and reuse across Program boundaries. The exact MNIR 0.1.0 scope remains subject to future specification.

A centralized registry and its publication, indexing, availability, moderation, and federation behavior may come later.

## Package discovery for AI

Future AI tooling should be able to search for semantic capabilities rather than only package names. Conceptually:

```text
search packages for:
    "password authentication"

inspect:
    exported capabilities
    version
    security/review metadata
    compatibility
```

The subsequent workflow might be:

```text
add dependency
inspect exports
use exported semantic entities
```

The actual discovery, dependency, and import APIs are undefined. Search terms and presentation metadata should aid discovery without becoming semantic identity.

This package-first model complements the AI authoring principles in [`AI-SEMANTIC-AUTHORING.md`](AI-SEMANTIC-AUTHORING.md).

## Supply-chain and integrity direction

A future package system should consider:

- package integrity;
- immutable or versioned artifacts;
- dependency locking;
- provenance;
- signatures or other trust metadata;
- approved package sources;
- security-review metadata.

This document does not select a signature format, trust root, provenance protocol, review authority, or policy model. Those choices affect security guarantees and require explicit future design and specification.

## Relationship to canonical serialization

Package distribution will depend on the future canonical MNIR serialization model:

```text
Package
    ↓
canonical MNIR representation
    ↓
distribution / Git / registry
```

The architectural direction is to avoid a second competing canonical program representation. Package artifacts should use or compose the canonical MNIR representation so that identity, semantic round-trip, integrity, and verification operate on the same semantic foundation.

Manifest and registry metadata may surround canonical MNIR artifacts, but it should not silently replace or reinterpret their specified semantics.

## Open design work

Future specifications and architecture decisions will need to define at least:

- package identity and ownership;
- Module export and import semantics;
- dependency reference and resolution behavior;
- version and compatibility rules;
- conflict handling and lock semantics;
- package artifact and integrity representation;
- local, Git, and registry source behavior;
- trust, provenance, review, and approval semantics;
- interaction with canonical serialization and semantic verification.

Until those decisions exist, the concepts in this document remain architectural direction rather than MNIR behavior.
