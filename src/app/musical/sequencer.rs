use super::*;
use crate::prelude::*;
use crossbeam_channel::Sender;

use super::sequence::*;

/// For simplicity, the `Sequencer` assumes a 4/4 time signature.
pub struct Sequencer {
    /// The sample rate, held for easier timing calculations.
    sample_rate: f64,

    /// An internal scratch buffer for holding note events.
    note_events: Vec<Option<NoteEvent>>,

    /// The sequencer's BPM (beats per minute).
    bpm: f64,

    /// The sequencer's internal sample clock.
    clock: u32,

    /// The memoized samples per bar, used to wrap the clock.
    samples_per_bar: u32,

    /// The note event channel sender.
    ///
    /// Note: this is bounded to [`MAX_NOTE_EVENTS_PER_BUFFER`] elements at a time.
    note_event_channel: Sender<NoteEvent>,
}

// - tracks time on a per-sample basis - so needs to know the sample rate to keep track of time.
// - is called each audio callback to generate events.
// - has full control over the notation and timing of note events
//     (and thus can generate notes in any feasible way)
// -

impl Sequencer {
    pub fn new(sample_rate: f64, note_event_channel: Sender<NoteEvent>) -> Self {
        let mut s = Self {
            sample_rate,
            note_events: vec![None; MAX_NOTE_EVENTS_PER_BUFFER],
            bpm: 0.0,
            clock: 0,
            samples_per_bar: 0,
            note_event_channel,
        };

        s.set_bpm(DEFAULT_BPM);
        s
    }

    /// This method will clamp `bpm` between `30.0` and `240.0`.
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm.clamp(30.0, 240.0);

        let secs_per_bar = 60.0 / self.bpm * 4.0;
        self.samples_per_bar = (self.sample_rate * secs_per_bar) as u32;
    }

    /// Generates note events and sends them to the audio thread via the `note_event`
    /// MPMC channel.
    ///
    /// # Panics
    ///
    /// Panics if the `note_event` channel is disconnected.
    pub fn process(&mut self, buffer_size: usize) {
        unimplemented!();

        self.note_events.iter().filter_map(|&e| e).for_each(|e| {
            self.note_event_channel
                .send(e)
                .expect("expected the note event channel to exist!");
        });
    }

    fn increment_clock_by(&mut self, inc: u32) {
        self.clock += inc;

        if self.clock >= self.samples_per_bar {
            self.clock -= self.samples_per_bar;
        }
    }

    fn increment_clock(&mut self) {
        self.clock += 1;

        if self.clock >= self.samples_per_bar {
            self.clock -= self.samples_per_bar;
        }
    }
}
