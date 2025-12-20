# ABC Music Notation Format - Complete Feature Breakdown

## Overview

ABC notation is a text-based music notation system created by Chris Walshaw in
the 1990s. Originally designed for notating folk and traditional music of
Western European origin (particularly music that fits on a single staff), it has
evolved into a comprehensive system capable of representing complex classical
scores with multiple voices, staves, and advanced musical elements.

Current Standard: ABC 2.1 (released December 21, 2011)

Design Philosophy: ABC is designed to be readable by both humans and
computers, using only ASCII characters - letters, digits, and punctuation marks.
This makes it ideal for sharing music over the internet and in plain text files.

## File Structure

### Basic Organization

ABC files consist of:

- File identifier: Optional `%abc` or `%abc-2.1` at the beginning
- File header: Optional default settings for all tunes in the file
- Tune(s): One or more tunes, each with a header and body
- Separators: Empty lines separate tunes

### Tune Structure

Each tune contains:

1. Header: Information fields (metadata)
2. Body: Musical notation
3. Trailer: Optional additional fields like lyrics

## Information Fields

Information fields begin with a letter followed by a colon (e.g., `T:Title`).

### Required Fields

- `X:` - Reference number (marks the start of a tune, must be first)
- `T:` - Title (must follow X:)
- `K:` - Key signature (marks end of header, must be last in header)

### Standard Header Fields

Compositional Information:

- `C:` - Composer
- `O:` - Origin (geographical)
- `A:` - Area (geographical region)
- `Z:` - Transcription notes (who transcribed it)
- `H:` - History
- `B:` - Book source
- `S:` - Source
- `D:` - Discography (recording information)
- `F:` - File URL or path

Musical Parameters:

- `M:` - Meter/time signature (e.g., `M:4/4`, `M:6/8`, `M:C` for common time,
  `M:C|` for cut time)
- `L:` - Default unit note length (e.g., `L:1/8` for eighth notes)
- `Q:` - Tempo (e.g., `Q:1/4=120` means 120 quarter notes per minute)
- `R:` - Rhythm type (e.g., jig, reel, hornpipe, waltz)
- `P:` - Parts (e.g., `P:AABBCCAA` defines part ordering)

Additional Fields:

- `N:` - Notes (general annotations)
- `G:` - Group (classification)
- `W:` - Words/lyrics (end-of-tune lyrics)
- `w:` - Aligned lyrics (within tune body)

### Inline Fields

Fields can appear mid-tune in square brackets to change settings:

- `[M:3/4]` - Change meter
- `[K:D]` - Change key
- `[Q:1/4=100]` - Change tempo
- `[V:2]` - Switch to voice 2

## Note Notation

### Pitch Representation

Basic pitches:

- `C D E F G A B` - Lower octave (around middle C and below)
- `c d e f g a b` - Higher octave (above middle C)

Octave modifiers:

- `,` (comma) - Lowers pitch by one octave (`C,` is one octave below middle C)
- `'` (apostrophe) - Raises pitch by one octave (`c'` is one octave above middle
  c)
- Multiple modifiers possible: `C,,` (two octaves down), `c''` (two octaves up)

### Accidentals

Standard accidentals:

- `^` - Sharp (e.g., `^F` for F♯)
- `_` - Flat (e.g., `_B` for B♭)
- `=` - Natural (cancels key signature or previous accidental)

Double accidentals:

- `^^` - Double sharp
- `__` - Double flat

Scope: Accidentals apply only to the note they precede and last through the
measure (unless barlines are disabled).

### Note Duration

Duration modifiers:

- `A` - One unit note length (as specified by L: field)
- `A2` - Double length
- `A3` - Triple length
- `A/` or `A/2` - Half length
- `A/4` - Quarter length
- `A3/2` - Three halves length

Default behavior: Without an L: field, the default unit length is calculated
from the meter:

- Meters < 0.75: default to 1/16
- Meters ≥ 0.75: default to 1/8

### Rests and Silence

- `z` - Visible rest (can be modified with duration: `z2`, `z/2`)
- `x` - Invisible rest
- `Z` - Multi-measure rest (e.g., `Z4` = 4 measures of rest)
- `X` - Invisible multi-measure rest

## Rhythmic Features

### Broken Rhythm

Simulates dotted rhythm patterns without explicit dotting:

- `>` - Previous note dotted, next note halved (e.g., `A>B` = dotted eighth +
  sixteenth)
- `<` - Previous note halved, next note dotted (e.g., `A<B` = sixteenth + dotted
  eighth)
- `>>` - Previous note double-dotted, next note quartered
- `<<` - Previous note quartered, next note double-dotted

### Tuplets

Notation: `(p:q:r` where:

- `p` = number of notes in the tuplet
- `q` = number of notes they replace (optional)
- `r` = total duration they occupy (optional)

Examples:

- `(3abc` - Triplet (3 notes in the time of 2)
- `(2ab` - Duplet (2 notes in the time of 3)
- `(4abcd` - Quadruplet
- `(3:2:4 abc` - 3 notes in the time of 2, occupying 4 unit lengths
- Supports up to `(9`

### Beaming

Notes grouped without spaces are beamed together:

- `ABCD` - Four notes beamed together
- `AB CD` - Two pairs of beamed notes
- `A2BC` - A held note followed by two beamed notes

## Bar Lines and Repeats

### Bar Lines

- `|` - Standard bar line
- `||` - Double bar line (section ending)
- `|]` - Thin-thick double bar (final bar line)
- `[|` - Thick-thin double bar (start of section)

### Repeat Notation

- `|:` - Start repeat
- `:|` - End repeat
- `::` - Start and end repeat (double repeat)
- `|1` or `[1` - First ending
- `|2` or `[2` - Second ending
- Multiple endings: `[1,3`, `[2,4` (play on 1st and 3rd times, or 2nd and 4th
  times)

### Variant Endings

```abc
|: ABC |1 DEF :|2 GHI ||
```

First time through: ABC DEF, repeat to ABC Second time through: ABC GHI,
continue

## Chords and Harmony

### Simultaneous Notes (Chords)

Square brackets create chords (notes played simultaneously):

- `[CEG]` - C major chord
- `[^F_A]` - F♯ and A♭ together
- `[C2E2G2]` - Held chord (all double length)
- `[DD]` - Unison (same pitch notated twice)

### Guitar/Song Chords

Quoted text above staff indicates chord symbols:

- `"C"C2D2` - C chord symbol above melody
- `"Am7"A4` - A minor 7th chord
- `"G/B"B2` - G chord with B bass note

### Chord Definition (MIDI)

Define custom chord shapes for accompaniment:

```abc
%%MIDI chordname Cmaj7 0 4 7 11
```

## Multiple Voices and Polyphony

### Voice Declaration

The `V:` field defines separate voices/parts:

```abc
V:1 name="Soprano" clef=treble
V:2 name="Alto" clef=treble
V:3 name="Tenor" clef=bass octave=-1
V:4 name="Bass" clef=bass
```

Voice attributes:

- `name` - Voice name
- `clef` - Clef type
- `stems` - Stem direction (up/down/auto)
- `octave` - Transposition in octaves
- `transpose` - Transposition in semitones
- `instrument` - MIDI instrument

### Voice Switching

Switch between voices in the tune body:

```abc
[V:1] CDEF | GABc |
[V:2] C,D,E,F, | G,A,B,C |
```

### Voice Overlay

The `&` operator overlays voices within a single measure:

```abc
V:1
CDEF & G,B,DF |
```

This creates two parts in the same bar, sharing a staff.

## Lyrics

### Aligned Lyrics

The `w:` field aligns syllables with notes:

```abc
CDEF GABc |
w: Do Re Mi Fa Sol La Ti Do
```

Syllable alignment:

- Space - Next note
- `-` - Syllable continues (hyphenation)
- `_` - Hold syllable over multiple notes
- `*` - One note with no syllable
- `~` - appears as a space in the output
- `|` - Advances to next bar

Multiple verses:

```abc
CDEF |
w: First verse words here
w: Sec-ond verse words here
w: Third verse words here
```

### End-of-Tune Lyrics

The `W:` field adds lyrics after the tune:

```abc
W: First verse complete text
W: Can span multiple lines
W: Second verse text here
```

## Ornaments and Decorations

### Shorthand Decorations

Single-character shortcuts (placement varies by implementation):

- `~` - Roll/turn
- `.` - Staccato
- `H` - Fermata (hold)
- `L` - Accent/emphasis
- `M` - Lower mordent
- `P` - Upper mordent
- `T` - Trill
- `u` - Up-bow (strings)
- `v` - Down-bow (strings)

### Exclamation Mark Notation

The `!decoration!` syntax provides explicit decoration names:

Articulation:

- `!staccato!` - Staccato dot
- `!tenuto!` - Tenuto mark
- `!accent!` - Accent mark
- `!emphasis!` - Emphasis
- `!fermata!` - Fermata/hold
- `!shortphrase!` - Short phrase mark
- `!mediumphrase!` - Medium phrase mark
- `!longphrase!` - Long phrase mark

Dynamics:

- `!pppp!`, `!ppp!`, `!pp!`, `!p!` - Very very soft to soft
- `!mp!` - Mezzo-piano
- `!mf!` - Mezzo-forte
- `!f!`, `!ff!`, `!fff!`, `!ffff!` - Loud to very very loud
- `!sfz!` - Sforzando (sudden accent)
- `!crescendo(!` and `!crescendo)!` - Crescendo start/end
- `!diminuendo(!` and `!diminuendo)!` - Diminuendo start/end

Ornaments:

- `!trill!` - Trill
- `!trill(!` and `!trill)!` - Trill with wavy line start/end
- `!lowermordent!` - Lower mordent
- `!uppermordent!` - Upper mordent
- `!mordent!` - Mordent
- `!pralltriller!` - Pralltriller
- `!roll!` - Roll
- `!turn!` - Turn
- `!turnx!` - Inverted turn
- `!invertedturn!` - Inverted turn
- `!arpeggio!` - Arpeggio
- `!slide!` - Slide

Bowing (strings):

- `!upbow!` - Up-bow
- `!downbow!` - Down-bow

Repeats and navigation:

- `!D.C.!` - Da Capo
- `!D.S.!` - Dal Segno
- `!fine!` - Fine
- `!segno!` - Segno symbol
- `!coda!` - Coda symbol

Pedal (keyboard):

- `!ped!` - Pedal down
- `!ped-up!` - Pedal up

### Plus Sign Alternative

The `+decoration+` syntax is an alternative to `!decoration!`:

- `+trill+` instead of `!trill!`

Control which to use with:

```abc
I:decoration +
```

or

```abc
I:decoration !
```

### Grace Notes

Grace notes appear in curly braces `{}`:

- `{A}C` - Single grace note before C
- `{AB}C` - Multiple grace notes
- `{/A}C` - Acciaccatura (slashed grace note)

Grace note timing:

```abc
%%MIDI grace a/b
```

Grace notes occupy fraction a/b of the following note's duration.

## Ties, Slurs, and Phrasing

### Ties

Hyphen `-` after a note ties it to the next note of the same pitch:

```abc
C2-C2
```

The two C notes are joined, sounding as one C4.

### Slurs

Parentheses `()` create slurs across different pitches:

```abc
(CDEF)
```

All four notes are slurred together.

Dotted slurs:

```abc
.(CDEF)
```

Multiple slurs:

```abc
(C(D)E)
```

Creates nested slurs.

### Phrase Marks

Longer phrase markings can be indicated with decoration pairs:

- `!longphrase(!` ... `!longphrase)!`

## Key Signatures and Modes

### Key Signature Syntax

Format: `K:[tonic][#/b][mode]`

Major keys:

- `K:C` - C major (no sharps/flats)
- `K:G` - G major (1 sharp)
- `K:D` - D major (2 sharps)
- `K:F` - F major (1 flat)
- `K:Bb` - B♭ major (2 flats)

Minor keys:

- `K:Am` - A minor (natural minor)
- `K:Em` - E minor
- `K:Dm` - D minor

### Modes

Supported modes (replace "major" default):

- `K:G major` - Ionian (major) - default
- `K:D dorian` - Dorian mode
- `K:E phrygian` - Phrygian mode
- `K:F lydian` - Lydian mode
- `K:G mixolydian` - Mixolydian mode
- `K:A minor` or `K:Am` - Aeolian (natural minor)
- `K:B locrian` - Locrian mode

Mode abbreviations:

```abc
K:Dmin, K:Dmix, K:Ddor, K:Dphr, K:Dlyd, K:Dloc
```

### Special Keys

- `K:HP` - Highland bagpipe (F♯, C♯, G♮)
- `K:none` - No key signature

### Explicit Accidentals

Override key signature with explicit accidentals:

```abc
K:D =c
```

D major, but show natural on C as advisory.

### Key Change

Change key mid-tune:

```abc
[K:G]
```

## Clefs and Transposition

### Clef Selection

Clefs specified in K: or V: fields:

- `clef=treble` - Treble/G clef (default)
- `clef=bass` - Bass/F clef
- `clef=alto` - Alto/C clef (middle line)
- `clef=tenor` - Tenor clef
- `clef=perc` - Percussion (no pitch)

Octave transposition:

- `clef=treble-8` - Treble clef, sounds one octave lower
- `clef=treble+8` - Treble clef, sounds one octave higher
- `clef=bass+8` - Bass clef, sounds one octave higher

Line placement:

```abc
clef=treble1  % G on bottom line
clef=treble2  % G on second line
```

### Transposition

Semitone transposition:

```abc
K:C transpose=-2
```

Transposes down 2 semitones (to B♭).

Octave transposition:

```abc
K:C octave=-1
```

Transposes down one octave.

Middle pitch reference:

```abc
K:C middle=d
```

Defines middle line pitch as D.

### Staff Lines

Custom staff line count:

```abc
K:C stafflines=4
```

## MIDI Features

ABC notation includes extensive MIDI directives for playback control. All MIDI
directives use `%%MIDI` prefix or equivalent `I:MIDI` inline syntax.

### Channel and Program Selection

Channel selection:

```abc
%%MIDI channel n
```

Selects MIDI channel (1-16) for melody.

Program/instrument selection:

```abc
%%MIDI program [c] n
```

- `n` - Instrument number (0-127, General MIDI)
- `c` - Optional channel number (1-16)

Examples:

```abc
%%MIDI program 73        % Flute on current channel
%%MIDI program 1 0       % Acoustic Grand Piano on channel 1
%%MIDI program 10 127    % Gunshot on channel 10 (drums)
```

### Dynamics and Velocity

Beat strength:

```abc
%%MIDI beat a b c n
```

- `a` - Velocity for first note in bar (0-128)
- `b` - Velocity for strong beats
- `c` - Velocity for weak beats
- `n` - Beat pattern indicator

Beat string:

```abc
%%MIDI beatstring fmfp
```

- `f` - Forte/strong
- `m` - Medium
- `p` - Piano/soft

Example: `fmfp` for 4/4 with strong first beat, medium second, strong third,
soft fourth.

### MIDI Transposition

Absolute transpose:

```abc
%%MIDI transpose n
```

Transposes output by `n` semitones (positive or negative).

Relative transpose:

```abc
%%MIDI rtranspose n
```

Adds `n` semitones to current transposition.

Pitch mapping:

```abc
%%MIDI c n
```

Sets MIDI pitch for note C to value `n` (typically multiples of 12).

### Guitar Chord Accompaniment

Chord pattern:

```abc
%%MIDI gchord string
```

- `z` - Rest
- `c` - Chord notes
- `f` - Fundamental/bass note
- `b` - Both bass and chord

Example: `%%MIDI gchord fzcz` plays bass on beats 1 and 3, chord on beats 2
and 4.

Chord instrument:

```abc
%%MIDI chordprog n
```

Sets MIDI instrument for chord notes (0-127).

Chord volume:

```abc
%%MIDI chordvol n
```

Sets velocity for chord notes (0-127).

Enable/disable chords:

```abc
%%MIDI gchordon    % Enable
%%MIDI gchordoff   % Disable
```

Custom chord definitions:

```abc
%%MIDI chordname name n1 n2 n3 n4 n5 n6
```

Defines chord with root and up to 5 additional notes (in semitones).

Example:

```abc
%%MIDI chordname Cmaj7 0 4 7 11
```

### Bass Notes

Bass instrument:

```abc
%%MIDI bassprog n
```

Sets instrument for bass notes (0-127).

Bass volume:

```abc
%%MIDI bassvol n
```

Sets velocity for bass notes (0-127).

### Drum Patterns

```abc
%%MIDI drum string [drum programs] [drum velocities]
```

- `d` - Drum strike
- `z` - Rest

Drums use channel 10 (General MIDI percussion).

Enable/disable:

- `!drum!` - Start drum pattern
- `!nodrum!` - Stop drum pattern

Example:

```abc
%%MIDI drum d2zd 36 38
```

Two bass drums, rest, one snare.

### Grace Note Timing

```abc
%%MIDI grace a/b
```

Grace notes occupy fraction `a/b` of the next note's duration.

Example: `%%MIDI grace 1/8` makes grace notes take 1/8 of the following note.

### Broken Rhythm Ratio

```abc
%%MIDI ratio n m
```

Sets ratio for broken rhythm notation (default 2:1 for hornpipes, 3:1 for
standard).

### Control Changes

```abc
%%MIDI control [bass/chord] n1 n2
```

Generates MIDI controller event:

- `n1` - Controller number (e.g., 7 for volume, 10 for pan)
- `n2` - Controller value (0-127)

Examples:

```abc
%%MIDI control 7 100      % Set volume to 100
%%MIDI control 10 64      % Center pan
%%MIDI control bass 7 80  % Set bass volume to 80
```

### Pitch Bend

```abc
%%MIDI pitchbend [bass/chord] <high byte> <low byte>
```

Creates pitch bend effect (experimental).

### Barline Behavior

```abc
%%MIDI nobarlines   % Accidentals persist across bars
%%MIDI barlines     % Normal barline behavior (default)
```

Used for early music notation without regular barlines.

## Tablature Support

ABC notation can be extended to support tablature for stringed instruments,
though this is implementation-dependent.

### Software Support

Several tools extend ABC for tablature:

abctab2ps: Extended abc2ps to support guitar and lute tablature notation.

FolkTab: Online editor supporting tablature for:

- Guitar (standard EADGBE tuning)
- DADGAD guitar
- Mandolin
- Violin
- 5-string banjo
- Ukulele

Michael Eskin's ABC Tools: Supports standard notation alongside tablature
for various tunings.

### Tablature Notation (Tool-Dependent)

Generally uses additional fields and syntax extensions:

```abc
%%tablature guitar
```

Specific implementation varies by software package.

## Macros and User-Defined Symbols

### Macros

The `m:` field defines reusable note sequences:

```abc
m:~1 = {gf}
```

Usage in tune:

```abc
~1C
```

Expands to `{gf}C`.

Common use: Defining standard ornaments for consistent playback.

### Redefinable Symbols

The `U:` field redefines symbol meanings:

```abc
U:T = !trill!
```

Now `T` in the tune body applies a trill.

Purpose: Customize shorthand symbols or create new ones.

### Symbol Scope

Macros and redefined symbols can be:

- File-level: Defined in file header (apply to all tunes)
- Tune-level: Defined in tune header (apply to one tune)

## Stylesheet Directives and Formatting

Stylesheet directives control page layout and typesetting. Use `%%` prefix or
`I:` field.

### Interchange Syntax

```abc
%%directive value
```

or equivalently:

```abc
I:directive value
```

### Common Directives

Page layout:

- `%%pagewidth 21cm` - Set page width
- `%%pageheight 29.7cm` - Set page height
- `%%topmargin 1cm` - Top margin
- `%%botmargin 1cm` - Bottom margin
- `%%leftmargin 1.5cm` - Left margin
- `%%rightmargin 1.5cm` - Right margin
- `%%landscape` - Landscape orientation

Staff formatting:

- `%%staffwidth 18cm` - Width of staff
- `%%scale 0.75` - Scale factor for entire output
- `%%staffsep 40pt` - Space between staves
- `%%systemsep 50pt` - Space between systems
- `%%maxshrink 0.65` - Maximum compression for line fitting
- `%%stretchlast` - Stretch last line to full width

Text formatting:

- `%%titlefont Times-Bold 20` - Title font and size
- `%%subtitlefont Times 16` - Subtitle font
- `%%composerfont Times-Italic 14` - Composer font
- `%%gchordfont Helvetica 12` - Guitar chord font
- `%%vocalfont Times-Bold 13` - Lyrics font
- `%%textfont Times 12` - General text font

Staff elements:

- `%%stafflines 5` - Number of staff lines
- `%%measurenb 1` - Show measure numbers (start at 1)
- `%%measurefirst 1` - First measure to number
- `%%barsperstaff 4` - Force 4 bars per staff line

Line breaking:

- `%%linebreak <none>` - No automatic line breaks
- `%%linebreak $` - Break at `$` symbols in tune
- `%%continueall` - Show continuation across line breaks
- `%%breaklimit 0.7` - Line fullness before breaking (0-1)

Miscellaneous:

- `%%shownotes` - Show note names on noteheads (for teaching)
- `%%writefields MTCOKL` - Select which fields to display
- `%%titlecaps` - Force title to capitals
- `%%infoline` - Show tune info in one line

### Character Set

```abc
I:abc-charset utf-8
```

Specifies character encoding for the file.

### ABC Version

```abc
I:abc-version 2.1
```

Declares ABC standard version used.

## Comments and Remarks

### Full-Line Comments

`%` at start of line (or after whitespace) begins a comment:

```abc
% This is a comment
X:1
% Another comment
T:Sample Tune
```

Comments are ignored by parsers and not printed.

### Inline Remarks

Square bracket remarks appear in the score:

```abc
[r:legato]
```

These are typeset as annotations.

### Text Annotations

Quoted text without chords can appear above/below staff:

```abc
"^Above staff text"C4
"_Below staff text"C4
"<To left"C4
">To right"C4
"@Center above"C4
```

## Parts and Structure

### Part Definition

The `P:` field defines part ordering:

```abc
P:AABBCCDD
```

### Part Markers

Mark sections in the tune body:

```abc
P:A
[musical notation for part A]
P:B
[musical notation for part B]
```

When playing, follows order in header P: field.

### Nested Repeats

Part notation can indicate complex repeat structures:

```abc
P:(AB)3C2
```

Play A then B three times, then C twice.

## Advanced Features

### Line Continuation

Backslash `\` continues a line:

```abc
CDEF GABB \
cBAG FEDC |
```

Both lines treated as continuous music.

### Typeset Text

Text directives for printing:

```abc
%%text This text appears in the printed score
%%center This is centered
%%vskip 1cm
```

### Embedded ABC

ABC can be embedded in HTML/XML with type indicators:

```html
<div class="abc">X:1 T:Embedded Tune K:D DEFG|</div>
```

### Microtones

Some implementations support microtonal notation:

```abc
^3/4F  % Three-quarter sharp
_1/4B  % Quarter flat
```

Support varies by software.

### Spacer Rests

Invisible rest (takes space but doesn't show):

```abc
x4
```

Useful for aligning voices.

### Chord Syntax Extensions

Modern implementations may support:

```abc
[1C] [2D] [3E] [4F]
```

Creates four separate chords with voice numbers.

### Alignment

Align notes vertically in multi-voice music:

```abc
V:1
C & E |
V:2
D & F |
```

The `&` ensures vertical alignment.

## File Organization and Includes

### Multiple Tunes

A single ABC file can contain many tunes:

```abc
X:1
T:First Tune
K:G
...

X:2
T:Second Tune
K:D
...
```

### File Header

Settings before first `X:` apply to all tunes:

```abc
%%scale 0.8
I:abc-charset utf-8

X:1
T:First Tune
...
```

### Include Files

Some implementations support:

```abc
%%abc-include filename.abc
```

Includes content from another file.

## Software Compatibility Notes

### Standard vs. Extensions

- Core ABC 2.1: Widely supported across all software
- MIDI directives: Supported by abc2midi and related tools
- Tablature: Implementation-specific extensions
- Advanced typography: Varies by rendering engine

### Common Software

Converters:

- abc2midi - ABC to MIDI conversion
- abc2ps/abcm2ps - ABC to PostScript/PDF
- midi2abc - MIDI to ABC conversion

Web-based:

- abcjs - JavaScript ABC renderer
- EasyABC - ABC editor and converter
- FolkTab - Online tablature editor

Desktop:

- EasyABC - Cross-platform editor
- ABCexplorer - Mac OS X editor
- TuxGuitar - Includes ABC support

## Best Practices

1. Always include required fields: X:, T:, and K:
2. Set L: explicitly for clarity, don't rely on defaults
3. Use meaningful reference numbers in X: for large collections
4. Comment complex sections with % comments
5. Specify ABC version with `I:abc-version 2.1` for compatibility
6. Test with multiple renderers as implementations vary
7. Keep lines under 80 characters for readability
8. Use consistent spacing around bar lines
9. Document custom macros and symbols
10. Validate files with ABC syntax checkers before sharing

## Example: Complete ABC Tune

```abc
%abc-2.1
X:1
T:Example Waltz
T:A Complete Demonstration
C:Anonymous
M:3/4
L:1/8
Q:1/4=120
R:Waltz
K:G
P:AABB
% Part A
P:A
|:"G"B2 d2 g2|"D7"f4 a2|"G"b2 g2 d2|"C"e6|
"G"B2 d2 g2|"D7"f2 e2 d2|"G"B4 G2|1"D7"A6:|2"G"G6||
% Part B
P:B
|:"C"e2 g2 c'2|"G"b4 d'2|"Am"c'2 a2 e2|"D7"f6|
"G"g2 b2 d'2|"Em"e2 g2 b2|"D7"a4 f2|1"G"g6:|2"G"g4||
```

## Resources and Further Reading

- [ABC Notation Standard 2.1][abc-standard] - Official specification
- [ABC Music Notation][abc-mit] - Comprehensive tutorial
- [Steve Mansfield's ABC Tutorial][abc-lesession] - Practical guide
- [ABC Notation - Wikipedia][abc-wiki] - Overview and history
- [abcjs Documentation][abcjs] - Modern web implementation
- [abc2midi Documentation][abc2midi] - MIDI directive reference
- [The ABC Music Project][abc-sourceforge] - Tools and resources
- [Making Music with ABC 2][abc-guide-pdf] - Practical guide (PDF)

[abc-standard]: https://abcnotation.com/wiki/abc:standard:v2.1
[abc-mit]: https://trillian.mit.edu/~jc/music/abc/doc/ABC.html
[abc-lesession]: http://www.lesession.co.uk/abc/abc_notation.htm
[abc-wiki]: https://en.wikipedia.org/wiki/ABC_notation
[abcjs]: https://paulrosen.github.io/abcjs/overview/abc-notation.html
[abc2midi]: https://abc.sourceforge.net/standard/abc2midi.txt
[abc-sourceforge]: https://abc.sourceforge.net/
[abc-guide-pdf]: https://abcplus.sourceforge.net/abcplus_en.pdf

---

Document Version: 1.0 Based on: ABC Standard 2.1 (December 2011) Last
Updated: 2025-12-19
