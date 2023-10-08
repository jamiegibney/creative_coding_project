use nannou::prelude::*;
use Note::*;

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
    // 
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

    fn from_value(value: i32) -> Self {
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
            _ => panic!("unknown note value: {value}"),
        }
    }
}

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

pub const MAJOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 4, 5, 7, 9, 11];
pub const MINOR_SCALE_INTERVALS: [u32; 7] = [0, 2, 3, 5, 7, 8, 10];
pub const MAJOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 2, 4, 7, 9];
pub const MINOR_PENTATONIC_SCALE_INTERVALS: [u32; 5] = [0, 3, 5, 7, 10];
