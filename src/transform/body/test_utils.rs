//! Shared test utilities for body transformation tests.

use std::rc::Rc;

use crate::{
    transform::context::TransformContext,
    types::{Duration, KeySignature, Meter, Mode, PitchClass},
};

/// Create a standard test context with C major, 4/4 time, and 1/8 unit length.
pub(super) fn create_test_context() -> TransformContext<'static> {
    let key = KeySignature {
        tonic: PitchClass::C,
        accidental: None,
        mode: Mode::Major,
        explicit_accidentals: Vec::new(),
        clef: None,
        transpose: None,
        octave_shift: None,
        middle: None,
        stafflines: None,
    };

    TransformContext::new(
        Rc::new(key),
        Meter {
            numerator: 4,
            denominator: 4,
        },
        None,
        Duration::new(1, 8),
    )
}
