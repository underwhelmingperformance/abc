//! Tune transformation from AST to public types.
//!
//! This module implements the `TryFrom` conversion from internal AST types
//! to the public `Tune`, `Voice`, and `Measure` types. The transformation
//! resolves all context-dependent values and validates the tune structure.

use std::{collections::HashMap, convert::TryFrom, rc::Rc};

use super::{
    body::BodyTransformer,
    context::{TransformContext, infer_unit_length},
};
use crate::types::{
    AbcError, Duration, KeySignature, Meter, MeterSymbol, ReferenceNumber, TempoMarking, Tune,
    TuneMetadata, VoiceId,
    ast::{self, InformationField, MacroDefinition},
};

/// Extracted metadata from a tune header.
struct HeaderMetadata<'input> {
    reference_number: ReferenceNumber,
    title: &'input str,
    key: KeySignature,
    meter: Meter,
    tempo: Option<TempoMarking>,
    unit_length: Duration,
    composer: Option<&'input str>,
    origin: Option<&'input str>,
    rhythm: Option<&'input str>,
    metadata: TuneMetadata<'input>,
    macros: HashMap<char, Rc<MacroDefinition<'input>>>,
}

impl<'input> TryFrom<ast::Tune<'input>> for Tune<'input> {
    type Error = AbcError<'input>;

    fn try_from(ast_tune: ast::Tune<'input>) -> Result<Self, Self::Error> {
        // Extract required metadata from header
        let HeaderMetadata {
            reference_number,
            title,
            key,
            meter,
            tempo,
            unit_length,
            composer,
            origin,
            rhythm,
            metadata,
            macros,
        } = extract_metadata(ast_tune.header)?;

        // Transform body - clone key and tempo to wrap in Rc for context sharing
        let context_builder = TransformContext::new(
            Rc::new(key.clone()),
            meter,
            tempo.as_ref().map(|t| Rc::new(t.clone())),
            unit_length,
        );
        let mut transformer = BodyTransformer::new(context_builder, macros);

        for element in &ast_tune.body.elements {
            transformer.process_element(element);
        }

        let voices = transformer.into_voices();
        let default_voice = VoiceId::new("1");

        Ok(Tune {
            reference_number,
            title,
            composer,
            origin,
            rhythm,
            metadata,
            key,
            meter,
            tempo,
            unit_length,
            voices,
            default_voice,
        })
    }
}

/// Extract metadata from a tune header.
fn extract_metadata<'input>(
    header: ast::TuneHeader<'input>,
) -> Result<HeaderMetadata<'input>, AbcError<'input>> {
    let mut reference_number = None;
    let mut title = None;
    let mut key = None;
    let mut meter = Meter {
        numerator: 4,
        denominator: 4,
    };
    let mut tempo = None;
    let mut unit_length = None;
    let mut composer = None;
    let mut origin = None;
    let mut rhythm = None;

    // Optional metadata fields
    let mut area = None;
    let mut book = None;
    let mut discography = None;
    let mut file_url = None;
    let mut group = None;
    let mut history = None;
    let mut notes = None;
    let mut source = None;
    let mut transcription = None;

    // Macro definitions
    let mut macros = HashMap::new();

    // Extract fields
    for field in header.fields {
        match field {
            InformationField::ReferenceNumber(r) => reference_number = Some(r),
            InformationField::Title(t) => title = Some(t),
            InformationField::Key(k) => key = Some(k),
            InformationField::Meter(m) => {
                meter = match m {
                    MeterSymbol::CommonTime => Meter {
                        numerator: 4,
                        denominator: 4,
                    },
                    MeterSymbol::CutTime => Meter {
                        numerator: 2,
                        denominator: 2,
                    },
                    MeterSymbol::Explicit(m) => m,
                };
            }
            InformationField::Tempo(t) => tempo = Some(t),
            InformationField::UnitNoteLength(l) => unit_length = Some(l),
            InformationField::Composer(c) => composer = Some(c),
            InformationField::Origin(o) => origin = Some(o),
            InformationField::Rhythm(r) => rhythm = Some(r),
            InformationField::Area(a) => area = Some(a),
            InformationField::Book(b) => book = Some(b),
            InformationField::Discography(d) => discography = Some(d),
            InformationField::FileUrl(f) => file_url = Some(f),
            InformationField::Group(g) => group = Some(g),
            InformationField::History(h) => history = Some(h),
            InformationField::Notes(n) => notes = Some(n),
            InformationField::Source(s) => source = Some(s),
            InformationField::Transcription(z) => transcription = Some(z),
            InformationField::Macro(macro_def) => {
                macros.insert(macro_def.symbol, macro_def);
            }
            // Voice, Parts, UserDefined, etc. are handled elsewhere or not yet implemented
            _ => {}
        }
    }

    // Validate required fields
    // Check reference number first so we can use it in other error messages
    let reference_number = reference_number.ok_or(AbcError::MissingRequiredField {
        field: "X (reference number)".to_string(),
        reference: ReferenceNumber(0),
    })?;

    let title = title.ok_or(AbcError::MissingRequiredField {
        field: "T (title)".to_string(),
        reference: reference_number,
    })?;
    let key = key.ok_or(AbcError::MissingRequiredField {
        field: "K (key)".to_string(),
        reference: reference_number,
    })?;

    // Infer unit length if not specified
    let unit_length = unit_length.unwrap_or_else(|| infer_unit_length(&meter));

    let metadata = TuneMetadata {
        area,
        book,
        discography,
        file_url,
        group,
        history,
        notes,
        source,
        transcription,
    };

    Ok(HeaderMetadata {
        reference_number,
        title,
        key,
        meter,
        tempo,
        unit_length,
        composer,
        origin,
        rhythm,
        metadata,
        macros,
    })
}
