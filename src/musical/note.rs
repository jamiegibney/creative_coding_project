use nannou::prelude::*;

// this is OK - there is no intention of changing the variants of this enum.
use Note::*;
use Octave::*;

pub fn midi_note_value_from(octave: Octave, note: Note) -> i32 {
    octave.starting_midi_note() + note.note_value()
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Octave {
    /// Octave covering C-1 - B-1 (MIDI note range 0 - 11)
    Cneg1,
    /// Octave covering C0 - B0 (MIDI note range 12 - 23)
    C0,
    /// Octave covering C1 - B1 (MIDI note range 24 - 35)
    C1,
    /// Octave covering C2 - B2 (MIDI note range 36 - 47)
    C2,
    /// Octave covering C3 - B3 (MIDI note range 48 - 59)
    #[default]
    C3,
    /// Octave covering C4 - B4 (MIDI note range 60 - 71)
    C4,
    /// Octave covering C5 - B5 (MIDI note range 72 - 83)
    C5,
    /// Octave covering C6 - B6 (MIDI note range 84 - 95)
    C6,
    /// Octave covering C7 - B7 (MIDI note range 96 - 107)
    C7,
    /// Octave covering C8 - B8 (MIDI note range 108 - 119)
    C8,
    /// Octave covering C9 - B9 (MIDI note range 120 - 131)
    C9,
}

impl Octave {
    /// Returns the value of the starting note of this octave.
    pub fn starting_midi_note(&self) -> i32 {
        match self {
            Cneg1 => 0,
            C0 => 12,
            C1 => 24,
            C2 => 36,
            C3 => 48,
            C4 => 60,
            C5 => 72,
            C6 => 84,
            C7 => 96,
            C8 => 108,
            C9 => 120,
        }
    }

    /// Returns the `Octave` which covers the provided MIDI note.o
    ///
    /// # Panics
    ///
    /// Panics if `note` is outside of the range `0` to `132`.
    pub fn from_note(note: i32) -> Self {
        match note {
            0..=11 => Cneg1,
            12..=23 => C0,
            24..=35 => C1,
            36..=47 => C2,
            48..=59 => C3,
            60..=71 => C4,
            72..=83 => C5,
            84..=95 => C6,
            96..=107 => C7,
            108..=119 => C8,
            120..=131 => C9,
            _ => panic!(
                "value provided ({note}) is outside of the acceptible range"
            ),
        }
    }

    /// Increases the octave by one. Does not exceed C9.
    pub fn increase(&mut self) {
        *self = match self {
            Cneg1 => C0,
            C0 => C1,
            C1 => C2,
            C2 => C3,
            C3 => C4,
            C4 => C5,
            C5 => C6,
            C6 => C7,
            C7 => C8,
            C8 => C9,
            C9 => C9,
        };
    }

    /// Increases the octave by `amount`. Does not exceed C9.
    pub fn increase_by(&mut self, amount: i32) {
        for _ in 0..amount {
            self.increase();
        }
    }

    /// Decreases the octave by one. Does not exceed C-1.
    pub fn decrease(&mut self) {
        *self = match self {
            Cneg1 => Cneg1,
            C0 => Cneg1,
            C1 => C0,
            C2 => C1,
            C3 => C2,
            C4 => C3,
            C5 => C4,
            C6 => C5,
            C7 => C6,
            C8 => C7,
            C9 => C8,
        };
    }

    /// Decreases the octave by `amount`. Does not exceed C-1.
    pub fn decrease_by(&mut self, amount: i32) {
        for _ in 0..amount {
            self.decrease();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Note {
    /// Returns the note with a given transposition.
    pub fn transpose(&self, semitones: i32) -> Self {
        let mut value = (self.note_value() + semitones) % 12;
        while value < 0 {
            value += 12;
        }

        Self::from_value(value)
    }

    /// Returns the key associated with a specific key on the keyboard.
    pub fn from_key(key: &Key) -> Option<Self> {
        match key {
            Key::A => Some(C),
            Key::W => Some(Cs),
            Key::S => Some(D),
            Key::E => Some(Ds),
            Key::D => Some(E),
            Key::F => Some(F),
            Key::T => Some(Fs),
            Key::G => Some(G),
            Key::Y => Some(Gs),
            Key::H => Some(A),
            Key::U => Some(As),
            Key::J => Some(B),
            Key::K => Some(C),
            Key::O => Some(Cs),
            Key::L => Some(D),
            Key::P => Some(Ds),
            _ => None,
        }
    }

    /// Returns the value of the note for any octave.
    ///
    /// `C` is represented as 0, and `B` as 11.
    pub fn note_value(&self) -> i32 {
        match self {
            C => 0,
            Cs => 1,
            D => 2,
            Ds => 3,
            E => 4,
            F => 5,
            Fs => 6,
            G => 7,
            Gs => 8,
            A => 9,
            As => 10,
            B => 11,
        }
    }

    /// Returns the note associated with the provided MIDI note value.
    ///
    /// # Panics
    ///
    /// Panics if `value` is out of the range `0` to `132`.
    pub fn from_value(value: i32) -> Self {
        assert!((0..=132).contains(&value));

        let value = value % 12;
        match value {
            0 => C,
            1 => Cs,
            2 => D,
            3 => Ds,
            4 => E,
            5 => F,
            6 => Fs,
            7 => G,
            8 => Gs,
            9 => A,
            10 => As,
            11 => B,
            // this will never happen but might as well panic!
            _ => panic!("unknown note value: {value}"),
        }
    }
}

// TODO is this needed?
pub const KEYBOARD_MUSICAL_NOTES: [Key; 16] = [
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::G,
    Key::H,
    Key::J,
    Key::K,
    Key::L,
    Key::W,
    Key::E,
    Key::T,
    Key::Y,
    Key::U,
    Key::O,
    Key::P,
];

/// The intervals of notes in a major scale for a single octave.
pub const MAJOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 4, 5, 7, 9, 11];
/// The intervals of notes in a minor scale for a single octave.
pub const MINOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 3, 5, 7, 8, 10];
/// The intervals of notes in a major pentatonic scale for a single octave.
pub const MAJOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 2, 4, 7, 9];
/// The intervals of notes in a minor pentatonic scale for a single octave.
pub const MINOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 3, 5, 7, 10];
