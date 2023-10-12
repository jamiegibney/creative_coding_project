// use crate::dsp::Generator;

// TODO this may represent an audio generator, but the Generator trait as
// above is a better option if multiple structs should be a type of generator.
struct Generator;

pub struct Voice {
    /// The voice's unique ID.
    id: i32,
    /// The MIDI note of the voice.
    note: u8,

    /// The current phase of the voice.
    phase: f64,
    /// The phase increment to control the frequency of the voice. Derived
    /// from the note value, this may be altered to change the voice's pitch.
    // TODO should this be smoothed?
    phase_increment: f64,

    /// Whether or not the voice is currently releasing, which contains
    /// the number of samples left until the voice should be cleared.
    releasing: Option<u32>,

    /// The audio generator stored within the voice.
    generator: Generator,
    // generator: Box<dyn Generator>,
}


