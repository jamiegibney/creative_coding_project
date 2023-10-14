use crate::prelude::*;
use std::collections::VecDeque as Deque;

/// An enum to represent individual note states and their data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteEvent {
    NoteOn {
        /// The MIDI note value of the note.
        note: u8,
        /// The sample offset from the start of the block to the start of the note.
        timing: u32,
        // TODO: needed?
        // id: Option<i32>,
    },
    NoteOff {
        /// The MIDI note value of the note.
        note: u8,
        /// The sample offset from the start of the block to the start of the note.
        timing: u32,
        // TODO: needed?
        // id: Option<i32>,
    },
}

impl NoteEvent {
    /// Returns the MIDI note value of the event.
    pub fn note_value(&self) -> u8 {
        match self {
            Self::NoteOn { note, .. } => *note,
            Self::NoteOff { note, .. } => *note,
        }
    }

    /// Returns the frequency value of the event.
    pub fn freq_value(&self) -> f64 {
        note_to_freq(self.note_value() as f64)
    }

    /// Returns the sample timing of the event.
    pub fn timing(&self) -> u32 {
        match self {
            Self::NoteOn { timing, .. } => *timing,
            Self::NoteOff { timing, .. } => *timing,
        }
    }
}

#[derive(Debug)]
pub struct NoteHandler {
    events: Deque<NoteEvent>,
}

impl NoteHandler {
    /// Returns a new, empty `NoteHandler`.
    pub fn new() -> Self {
        Self { events: Deque::new() }
    }

    /// Adds a note event to the internal queue.
    pub fn push_event(&mut self, event: NoteEvent) {
        self.events.push_back(event);
    }

    /// Obtains the next event in the internal queue, or returns `None` if 
    /// there are no events.
    pub fn next_event(&mut self) -> Option<NoteEvent> {
        self.events.pop_front()
    }
}
