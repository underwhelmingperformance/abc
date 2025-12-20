//! Transformation from AST to public types.
//!
//! This module handles the conversion from the internal AST representation
//! to the public, validated types that users work with. The transformation
//! includes:
//!
//! - Resolving context-dependent values (pitches, durations)
//! - Validating structure (required fields, voice references)
//! - Computing absolute timing information
//! - Tracking musical context changes
//!
//! Transformation is implemented via `TryFrom` traits, making conversion
//! idiomatic and straightforward.

pub(crate) mod body;
pub(crate) mod context;
pub(crate) mod tune;
