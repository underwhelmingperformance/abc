//! MIDI-specific types.
//!
//! This module provides types for MIDI output generation: channel numbers,
//! program (instrument) numbers, and velocity values.
//!
//! These types are intentionally isolated in this module to keep MIDI concepts
//! separate from the abstract musical representation used elsewhere in the library.

/// A MIDI channel number (1-16).
///
/// MIDI supports 16 channels, numbered 1 through 16. Channel 10 is
/// conventionally reserved for percussion in General MIDI.
///
/// # Examples
///
/// ```
/// # use abc::types::Channel;
/// let channel = Channel::new(1).unwrap();
/// let percussion = Channel::new(10).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Channel(u8);

impl Channel {
    /// Creates a new MIDI channel, returning `None` if the value is not in
    /// the range 1-16.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Channel;
    /// assert!(Channel::new(1).is_some());
    /// assert!(Channel::new(16).is_some());
    /// assert!(Channel::new(0).is_none());
    /// assert!(Channel::new(17).is_none());
    /// ```
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value >= 1 && value <= 16 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the channel number (1-16).
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns the zero-based channel index (0-15) for use with MIDI libraries.
    ///
    /// Most MIDI APIs use zero-based indexing internally, whilst ABC notation
    /// uses one-based numbering.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Channel;
    /// let channel = Channel::new(1).unwrap();
    /// assert_eq!(channel.zero_based(), 0);
    /// ```
    #[must_use]
    pub const fn zero_based(self) -> u8 {
        self.0 - 1
    }
}

/// A MIDI note number (0-127).
///
/// MIDI note numbers represent pitches, where middle C (C4) is 60.
/// Each increment represents one semitone.
///
/// This type is used for MIDI output generation. For abstract pitch
/// representation that preserves musical spelling, use
/// [`Pitch`](super::Pitch) instead.
///
/// # Examples
///
/// ```
/// # use abc::types::midi::MidiNote;
/// let middle_c = MidiNote::new(60).unwrap();
/// assert_eq!(middle_c.get(), 60);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MidiNote(u8);

impl MidiNote {
    /// Middle C (C4).
    pub const MIDDLE_C: Self = Self(60);

    /// The minimum valid MIDI note.
    pub const MIN: Self = Self(0);

    /// The maximum valid MIDI note.
    pub const MAX: Self = Self(127);

    /// Creates a new MIDI note, returning `None` if the value exceeds 127.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::MidiNote;
    /// assert!(MidiNote::new(60).is_some());
    /// assert!(MidiNote::new(128).is_none());
    /// ```
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 127 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the MIDI note number (0-127).
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Transposes this note by the given number of semitones.
    ///
    /// Returns `None` if the result would be outside the valid range.
    #[must_use]
    pub const fn transpose(self, semitones: i8) -> Option<Self> {
        let result = self.0 as i16 + semitones as i16;
        if result >= 0 && result <= 127 {
            Some(Self(result as u8))
        } else {
            None
        }
    }
}

/// A MIDI velocity value (0-127).
///
/// Velocity represents how hard a note is struck, affecting its volume and
/// timbre. A velocity of 0 is typically interpreted as a note-off event.
///
/// This type is used internally for MIDI output generation. For abstract
/// dynamics in the musical representation, use [`Dynamics`](super::Dynamics)
/// instead.
///
/// # Examples
///
/// ```
/// # use abc::types::midi::Velocity;
/// let soft = Velocity::new(40).unwrap();
/// let loud = Velocity::new(100).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Velocity(u8);

impl Velocity {
    /// Creates a new velocity, returning `None` if the value exceeds 127.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::Velocity;
    /// assert!(Velocity::new(64).is_some());
    /// assert!(Velocity::new(128).is_none());
    /// ```
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 127 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the velocity value (0-127).
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// A musical pitch represented as a MIDI note number with optional microtonal offset.
///
/// Middle C (C4) is represented as 60. The valid range for the base MIDI note
/// is 0-127 as per the MIDI specification. The microtonal offset represents
/// quarter-tone adjustments, where:
/// - 0 = no offset (standard MIDI pitch)
/// - +1 = quarter-tone sharp
/// - +2 = semitone sharp
/// - +3 = three-quarter-tone sharp
/// - -1 = quarter-tone flat
/// - -2 = semitone flat
/// - -3 = three-quarter-tone flat
///
/// # Examples
///
/// ```
/// # use abc::types::midi::MidiPitch;
/// let middle_c = MidiPitch::new_semitone(60).unwrap();
/// let c_sharp = MidiPitch::new_semitone(61).unwrap();
/// let c_quarter_sharp = MidiPitch::new_microtonal(60, 1).unwrap();
/// assert!(c_sharp > middle_c);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MidiPitch {
    /// Base MIDI note number (0-127)
    midi_note: u8,
    /// Quarter-tone offset (-3 to +3)
    microtonal_offset: i8,
}

impl MidiPitch {
    /// The MIDI note number for middle C (C4).
    pub const MIDDLE_C: Self = Self {
        midi_note: 60,
        microtonal_offset: 0,
    };

    /// The minimum valid MIDI note number.
    pub const MIN: Self = Self {
        midi_note: 0,
        microtonal_offset: 0,
    };

    /// The maximum valid MIDI note number.
    pub const MAX: Self = Self {
        midi_note: 127,
        microtonal_offset: 0,
    };

    /// Creates a new pitch with semitone precision (standard MIDI), returning `None` if the value exceeds 127.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::MidiPitch;
    /// assert!(MidiPitch::new_semitone(60).is_some());
    /// assert!(MidiPitch::new_semitone(128).is_none());
    /// ```
    #[must_use]
    pub const fn new_semitone(midi_note: u8) -> Option<Self> {
        if midi_note <= 127 {
            Some(Self {
                midi_note,
                microtonal_offset: 0,
            })
        } else {
            None
        }
    }

    /// Creates a new pitch with microtonal offset (quarter-tone precision).
    ///
    /// The microtonal offset must be between -3 and +3 (inclusive), representing
    /// quarter-tone adjustments. Returns `None` if the MIDI note exceeds 127 or
    /// if the microtonal offset is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::MidiPitch;
    /// assert!(MidiPitch::new_microtonal(60, 0).is_some());  // Middle C
    /// assert!(MidiPitch::new_microtonal(60, 1).is_some());  // C quarter-sharp
    /// assert!(MidiPitch::new_microtonal(60, -2).is_some()); // C flat (via microtonal)
    /// assert!(MidiPitch::new_microtonal(60, 4).is_none());  // Invalid offset
    /// ```
    #[must_use]
    pub const fn new_microtonal(midi_note: u8, microtonal_offset: i8) -> Option<Self> {
        if midi_note <= 127 && microtonal_offset >= -3 && microtonal_offset <= 3 {
            Some(Self {
                midi_note,
                microtonal_offset,
            })
        } else {
            None
        }
    }

    /// Returns the base MIDI note number (0-127).
    #[must_use]
    pub const fn midi_note(self) -> u8 {
        self.midi_note
    }

    /// Returns the microtonal offset in quarter-tones (-3 to +3).
    #[must_use]
    pub const fn microtonal_offset(self) -> i8 {
        self.microtonal_offset
    }

    /// Returns the total pitch in quarter-tones above MIDI note 0.
    #[must_use]
    pub const fn as_quartertones(self) -> i16 {
        (self.midi_note as i16) * 4 + (self.microtonal_offset as i16)
    }

    /// Transposes this pitch by a given number of semitones.
    ///
    /// Returns `None` if the transposition would result in a pitch outside
    /// the valid MIDI range (0-127).
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::MidiPitch;
    /// let c = MidiPitch::new_semitone(60).unwrap();
    /// assert_eq!(c.transpose(2), MidiPitch::new_semitone(62)); // D
    /// assert_eq!(c.transpose(-1), MidiPitch::new_semitone(59)); // B
    /// ```
    #[must_use]
    pub const fn transpose(self, semitones: i8) -> Option<Self> {
        let result = self.midi_note as i16 + semitones as i16;
        if result >= 0 && result <= 127 {
            Some(Self {
                midi_note: result as u8,
                microtonal_offset: self.microtonal_offset,
            })
        } else {
            None
        }
    }
}

impl PartialOrd for MidiPitch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MidiPitch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_quartertones().cmp(&other.as_quartertones())
    }
}

/// A MIDI control change message.
///
/// Control changes modify various parameters of the MIDI synthesizer, such as
/// volume (CC 7), pan (CC 10), expression (CC 11), and many others.
///
/// # Examples
///
/// ```
/// # use abc::types::midi::{ControlChange, Channel};
/// // Set volume (controller 7) to 100 on channel 1
/// let volume_change = ControlChange {
///     channel: Channel::new(1).unwrap(),
///     controller: 7,
///     value: 100,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlChange {
    /// The MIDI channel for this control change.
    pub channel: Channel,
    /// The controller number (0-127).
    ///
    /// Common controllers:
    /// - 7: Channel Volume
    /// - 10: Pan
    /// - 11: Expression
    /// - 64: Sustain Pedal
    pub controller: u8,
    /// The controller value (0-127).
    pub value: u8,
}

impl ControlChange {
    /// Creates a new control change message, returning `None` if controller or value exceeds 127.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::{ControlChange, Channel};
    /// let cc = ControlChange::new(Channel::new(1).unwrap(), 7, 100);
    /// assert!(cc.is_some());
    /// assert!(ControlChange::new(Channel::new(1).unwrap(), 128, 100).is_none());
    /// ```
    #[must_use]
    pub const fn new(channel: Channel, controller: u8, value: u8) -> Option<Self> {
        if controller <= 127 && value <= 127 {
            Some(Self {
                channel,
                controller,
                value,
            })
        } else {
            None
        }
    }
}

/// A MIDI pitch bend message.
///
/// Pitch bend modifies the pitch of all notes on a channel. The value ranges
/// from -8192 to +8191, where 0 represents no pitch change. The actual pitch
/// shift in semitones depends on the pitch bend sensitivity setting (typically ±2 semitones).
///
/// # Examples
///
/// ```
/// # use abc::types::midi::{PitchBendChange, Channel};
/// // Bend pitch up slightly on channel 1
/// let pitch_bend = PitchBendChange {
///     channel: Channel::new(1).unwrap(),
///     value: 2000,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PitchBendChange {
    /// The MIDI channel for this pitch bend.
    pub channel: Channel,
    /// The pitch bend value (-8192 to +8191).
    ///
    /// - -8192: Maximum pitch down
    /// - 0: No pitch change
    /// - +8191: Maximum pitch up
    pub value: i16,
}

impl PitchBendChange {
    /// Creates a new pitch bend message, returning `None` if value is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::midi::{PitchBendChange, Channel};
    /// let pb = PitchBendChange::new(Channel::new(1).unwrap(), 0);
    /// assert!(pb.is_some());
    /// assert!(PitchBendChange::new(Channel::new(1).unwrap(), -8192).is_some());
    /// assert!(PitchBendChange::new(Channel::new(1).unwrap(), 8191).is_some());
    /// assert!(PitchBendChange::new(Channel::new(1).unwrap(), 8192).is_none());
    /// ```
    #[must_use]
    pub const fn new(channel: Channel, value: i16) -> Option<Self> {
        if value >= -8192 && value <= 8191 {
            Some(Self { channel, value })
        } else {
            None
        }
    }
}
