//! Document structure types.
//!
//! This module defines the `Document` type, which represents a complete ABC
//! document containing one or more tunes with optional document-level metadata.

use std::convert::TryFrom;

use super::{ast, error::AbcError, tune::Tune};

/// A complete ABC document.
///
/// A document contains one or more tunes, optionally preceded by document-level
/// metadata. This is what users receive from `abc::parse()`.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use abc::{parse, types::*};
///
/// let input = "\
/// X:1
/// T:First Tune
/// C:Trad.
/// K:G
///
/// X:2
/// T:Second Tune
/// R:Reel
/// K:D
/// ";
/// let doc = parse(input).unwrap();
///
/// let mut voices1 = HashMap::new();
/// voices1.insert(
///     VoiceId::new("1"),
///     Voice {
///         id: VoiceId::new("1"),
///         name: None,
///         measures: Vec::new(),
///     },
/// );
///
/// let mut voices2 = HashMap::new();
/// voices2.insert(
///     VoiceId::new("1"),
///     Voice {
///         id: VoiceId::new("1"),
///         name: None,
///         measures: Vec::new(),
///     },
/// );
///
/// let expected = Document {
///     tunes: vec![
///         Tune {
///             reference_number: ReferenceNumber(1),
///             title: "First Tune",
///             composer: Some("Trad."),
///             origin: None,
///             rhythm: None,
///             metadata: TuneMetadata::default(),
///             key: KeySignature {
///                 tonic: PitchClass::G,
///                 accidental: None,
///                 mode: Mode::Major,
///                 explicit_accidentals: Vec::new(),
///                 clef: None,
///                 transpose: None,
///                 octave_shift: None,
///                 middle: None,
///                 stafflines: None,
///             },
///             meter: Meter { numerator: 4, denominator: 4 },
///             tempo: None,
///             unit_length: Duration::new(1, 8),
///             voices: voices1,
///             default_voice: VoiceId::new("1"),
///         },
///         Tune {
///             reference_number: ReferenceNumber(2),
///             title: "Second Tune",
///             composer: None,
///             origin: None,
///             rhythm: Some("Reel"),
///             metadata: TuneMetadata::default(),
///             key: KeySignature {
///                 tonic: PitchClass::D,
///                 accidental: None,
///                 mode: Mode::Major,
///                 explicit_accidentals: Vec::new(),
///                 clef: None,
///                 transpose: None,
///                 octave_shift: None,
///                 middle: None,
///                 stafflines: None,
///             },
///             meter: Meter { numerator: 4, denominator: 4 },
///             tempo: None,
///             unit_length: Duration::new(1, 8),
///             voices: voices2,
///             default_voice: VoiceId::new("1"),
///         },
///     ],
///     metadata: DocumentMetadata::default(),
/// };
///
/// assert_eq!(&doc, &expected);
/// ```
#[derive(Debug, PartialEq, Default, bon::Builder)]
pub struct Document<'input> {
    /// The tunes in this document.
    #[builder(default)]
    pub tunes: Vec<Tune<'input>>,

    /// Document-level metadata (version, creator, etc.).
    #[builder(default)]
    pub metadata: DocumentMetadata<'input>,
}

impl<'input> Document<'input> {
    /// Get the tunes in this document.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:Jig\nR:Jig\nK:D\n\nX:2\nT:Reel\nR:Reel\nK:G\n";
    /// let doc = parse(input).unwrap();
    ///
    /// let mut voices1 = HashMap::new();
    /// voices1.insert(VoiceId::new("1"), Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: Vec::new(),
    /// });
    ///
    /// let mut voices2 = HashMap::new();
    /// voices2.insert(VoiceId::new("1"), Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: Vec::new(),
    /// });
    ///
    /// let expected = vec![
    ///     Tune {
    ///         reference_number: ReferenceNumber(1),
    ///         title: "Jig",
    ///         composer: None,
    ///         origin: None,
    ///         rhythm: Some("Jig"),
    ///         metadata: TuneMetadata::default(),
    ///         key: KeySignature {
    ///             tonic: PitchClass::D,
    ///             accidental: None,
    ///             mode: Mode::Major,
    ///             explicit_accidentals: Vec::new(),
    ///             clef: None,
    ///             transpose: None,
    ///             octave_shift: None,
///                 middle: None,
///                 stafflines: None,
    ///         },
    ///         meter: Meter { numerator: 4, denominator: 4 },
    ///         tempo: None,
    ///         unit_length: Duration::new(1, 8),
    ///         voices: voices1,
    ///         default_voice: VoiceId::new("1"),
    ///     },
    ///     Tune {
    ///         reference_number: ReferenceNumber(2),
    ///         title: "Reel",
    ///         composer: None,
    ///         origin: None,
    ///         rhythm: Some("Reel"),
    ///         metadata: TuneMetadata::default(),
    ///         key: KeySignature {
    ///             tonic: PitchClass::G,
    ///             accidental: None,
    ///             mode: Mode::Major,
    ///             explicit_accidentals: Vec::new(),
    ///             clef: None,
    ///             transpose: None,
    ///             octave_shift: None,
///                 middle: None,
///                 stafflines: None,
    ///         },
    ///         meter: Meter { numerator: 4, denominator: 4 },
    ///         tempo: None,
    ///         unit_length: Duration::new(1, 8),
    ///         voices: voices2,
    ///         default_voice: VoiceId::new("1"),
    ///     },
    /// ];
    ///
    /// assert_eq!(doc.tunes(), &expected[..]);
    /// ```
    pub fn tunes(&self) -> &[Tune<'input>] {
        &self.tunes
    }

    /// Iterator over the tunes in this document.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use abc::{parse, types::*};
    ///
    /// let input = "X:1\nT:First\nK:C\n\nX:2\nT:Second\nK:D\n";
    /// let doc = parse(input).unwrap();
    ///
    /// let tunes: Vec<&Tune> = doc.iter().collect();
    ///
    /// let mut voices1 = HashMap::new();
    /// voices1.insert(VoiceId::new("1"), Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: Vec::new(),
    /// });
    ///
    /// let tune1 = Tune {
    ///     reference_number: ReferenceNumber(1),
    ///     title: "First",
    ///     composer: None,
    ///     origin: None,
    ///     rhythm: None,
    ///     metadata: TuneMetadata::default(),
    ///     key: KeySignature {
    ///         tonic: PitchClass::C,
    ///         accidental: None,
    ///         mode: Mode::Major,
    ///         explicit_accidentals: Vec::new(),
    ///         clef: None,
    ///         transpose: None,
    ///         octave_shift: None,
///                 middle: None,
///                 stafflines: None,
    ///     },
    ///     meter: Meter { numerator: 4, denominator: 4 },
    ///     tempo: None,
    ///     unit_length: Duration::new(1, 8),
    ///     voices: voices1,
    ///     default_voice: VoiceId::new("1"),
    /// };
    ///
    /// let mut voices2 = HashMap::new();
    /// voices2.insert(VoiceId::new("1"), Voice {
    ///     id: VoiceId::new("1"),
    ///     name: None,
    ///     measures: Vec::new(),
    /// });
    ///
    /// let tune2 = Tune {
    ///     reference_number: ReferenceNumber(2),
    ///     title: "Second",
    ///     composer: None,
    ///     origin: None,
    ///     rhythm: None,
    ///     metadata: TuneMetadata::default(),
    ///     key: KeySignature {
    ///         tonic: PitchClass::D,
    ///         accidental: None,
    ///         mode: Mode::Major,
    ///         explicit_accidentals: Vec::new(),
    ///         clef: None,
    ///         transpose: None,
    ///         octave_shift: None,
///                 middle: None,
///                 stafflines: None,
    ///     },
    ///     meter: Meter { numerator: 4, denominator: 4 },
    ///     tempo: None,
    ///     unit_length: Duration::new(1, 8),
    ///     voices: voices2,
    ///     default_voice: VoiceId::new("1"),
    /// };
    ///
    /// let expected = vec![&tune1, &tune2];
    ///
    /// assert_eq!(tunes, expected);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Tune<'input>> {
        self.tunes.iter()
    }
}

/// Document-level metadata.
///
/// Contains information that applies to the entire document rather than
/// individual tunes. Common examples include ABC version, character encoding,
/// and creator/software identification.
///
/// # Examples
///
/// ```
/// use abc::types::DocumentMetadata;
///
/// let metadata = DocumentMetadata {
///     version: Some("2.1"),
///     charset: Some("utf-8"),
///     creator: Some("Example Software v1.0"),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentMetadata<'input> {
    /// ABC version (I:abc-version).
    pub version: Option<&'input str>,

    /// Character set encoding (I:abc-charset).
    pub charset: Option<&'input str>,

    /// Creator/software identification (I:abc-creator).
    pub creator: Option<&'input str>,

    /// Other instruction directives.
    pub instructions: Vec<&'input str>,
}

impl<'input> TryFrom<ast::AbcDocument<'input>> for Document<'input> {
    type Error = AbcError<'input>;

    fn try_from(ast_doc: ast::AbcDocument<'input>) -> Result<Self, Self::Error> {
        // Transform all tunes
        let tunes = ast_doc
            .tunes
            .into_iter()
            .map(Tune::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        // Extract document-level metadata
        let metadata = if let Some(header) = ast_doc.header {
            extract_document_metadata(header)
        } else {
            DocumentMetadata::default()
        };

        Ok(Document { tunes, metadata })
    }
}

/// Extract document-level metadata from the document header.
fn extract_document_metadata(header: ast::DocumentHeader<'_>) -> DocumentMetadata<'_> {
    let mut metadata = DocumentMetadata::default();

    for field in header.fields {
        if let ast::InformationField::Instruction(inst) = field {
            // Parse specific instruction types
            if let Some(version) = inst.strip_prefix("abc-version:") {
                metadata.version = Some(version.trim());
            } else if let Some(charset) = inst.strip_prefix("abc-charset:") {
                metadata.charset = Some(charset.trim());
            } else if let Some(creator) = inst.strip_prefix("abc-creator:") {
                metadata.creator = Some(creator.trim());
            } else {
                metadata.instructions.push(inst);
            }
        }
    }

    metadata
}
