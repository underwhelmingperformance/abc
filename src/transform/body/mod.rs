//! Body transformation from AST to public types.
//!
//! This module implements the transformation of tune body elements from the AST
//! into resolved public types with pitch and timing values.

mod duration;
mod dynamics;
mod event;
mod fields;
mod lyrics;
mod pitch;
mod ties_slurs;
mod transformer;
mod voice;

pub(crate) use transformer::BodyTransformer;
pub(super) use transformer::LastNotePosition;

#[cfg(test)]
mod test_utils;
