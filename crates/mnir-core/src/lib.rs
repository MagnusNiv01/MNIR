#![forbid(unsafe_code)]

//! Ownership boundary for the future canonical MNIR data model.
//!
//! This crate will eventually contain semantic program structures, stable
//! identifiers, types, expressions, functions, metadata, and related core
//! concepts. None of those semantics are implemented yet.
//!
//! This crate must not depend on EasyH or on `mnir-verify`, and it should
//! remain independent of concrete execution backends.
