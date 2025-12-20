//! Type definitions for ABC notation parsing.
//!
//! This module contains all type definitions used throughout the ABC parser,
//! organised by purpose: pitch representation, duration, dynamics, MIDI output,
//! and tune structure.
//!
//! All public types are re-exported at this module level for convenience.
//! Import with `use abc::types::*` or individual types like `use abc::types::Pitch`.
//!
//! The main `Pitch` type preserves musical spelling (C# vs Db). For MIDI-specific
//! pitch representation with microtonal support, see [`midi::MidiPitch`].

// Internal modules - types are re-exported below
mod articulation;
mod context;
mod document;
mod duration;
mod dynamics;
mod error;
mod event;
mod instrument;
mod key;
mod lyric;
mod meter;
pub mod midi;
mod pitch;
mod tune;

pub(crate) mod ast;

// Public exports
pub use articulation::Articulation;
pub use context::Context;
pub use document::{Document, DocumentMetadata};
pub use duration::Duration;
pub use dynamics::{DynamicDirection, Dynamics};
pub use error::{AbcError, Result};
pub use event::{
    AnnotationEvent, AnnotationPlacement, Chord, Event, GuitarChordEvent, MarkerEvent,
    MidiControlEvent, MidiPitchBendEvent, Note, Rest, TimedEvent,
};
pub use instrument::Instrument;
pub use key::{Clef, KeySignature, Mode};
pub use lyric::Syllable;
pub use meter::{Meter, MeterSymbol, TempoMarking};
pub use midi::{Channel, ControlChange, MidiNote, MidiPitch, PitchBendChange};
pub use pitch::{Accidental, Octave, Pitch, PitchClass};
pub use tune::{
    Measure, MeasureNumber, PartLabel, ReferenceNumber, Tune, TuneMetadata, Voice, VoiceId,
};
