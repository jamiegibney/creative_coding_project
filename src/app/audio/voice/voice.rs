use nannou_audio::Buffer;
use std::sync::{mpsc, Arc, Mutex};

use super::note::NoteHandler;
use crate::dsp::synthesis::*;
use crate::dsp::*;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VoiceEvent {
    ReleaseAll,
    KillAll,
}

/// A struct to represent each individual voice.
#[derive(Clone, Debug)]
pub struct Voice {
    /// The voice's unique ID.
    pub id: u64,
    /// The MIDI note of the voice.
    pub note: u8,

    /// The voice's ADSR envelope.
    pub envelope: AdsrEnvelope,

    /// Whether or not the voice is currently releasing, which contains
    /// the number of samples left until the voice should be cleared.
    pub releasing: bool,

    /// The audio generator stored within the voice.
    pub generator: Generator,
    // this may cause issues with constructing new voices...
}

impl Voice {
    pub fn new(
        id: u64,
        note: u8,
        generator: Generator,
        envelope: Option<AdsrEnvelope>,
    ) -> Self {
        Self {
            id,
            note,
            envelope: envelope.unwrap_or_default(),
            releasing: false,
            generator,
        }
    }
}

/// A struct to handle all voices, i.e. the spawning and termination of voices.
#[derive(Debug)]
pub struct VoiceHandler {
    /// A reference to the note handler to obtain note events.
    pub note_handler_ref: Arc<Mutex<NoteHandler>>,
    /// The array of voices.
    pub voices: [Option<Voice>; NUM_VOICES as usize],
    voice_event_receiver: mpsc::Receiver<VoiceEvent>,
    /// Internal counter for assigning new IDs.
    id_counter: u64,
}

impl VoiceHandler {
    /// Builds a new `VoiceHandler` with a reference to the `NoteHandler`.
    ///
    /// The `NoteHandler` reference is used to obtain new note events
    /// automatically.
    pub fn build(
        note_handler_ref: Arc<Mutex<NoteHandler>>,
        voice_event_receiver: mpsc::Receiver<VoiceEvent>,
    ) -> Self {
        Self {
            note_handler_ref,
            voices: std::array::from_fn(|_| None),
            voice_event_receiver,
            id_counter: 0,
        }
    }

    pub fn process_block(
        &mut self,
        buffer: &mut Buffer<f64>,
        block_start: usize,
        block_end: usize,
        gain: [f64; MAX_BLOCK_SIZE],
    ) {
        let block_len = block_end - block_start;
        let mut voice_amp_envelope = [0.0; MAX_BLOCK_SIZE];

        // process any received voice events
        if let Ok(msg) = self.voice_event_receiver.try_recv() {
            match msg {
                VoiceEvent::ReleaseAll => {
                    self.start_release_for_active_voices();
                }
                VoiceEvent::KillAll => self.kill_active_voices(),
            }
        }

        for voice in self.voices.iter_mut().filter_map(|v| v.as_mut()) {
            voice
                .envelope
                .next_block(&mut voice_amp_envelope, block_len);

            for (value_idx, sample_idx) in (block_start..block_end).enumerate()
            {
                let amp = gain[value_idx] * voice_amp_envelope[value_idx];

                let (sample_l, sample_r) = voice.generator.process();

                // * 2 because the channels are interleaved
                buffer[sample_idx * 2] += sample_l * amp;
                buffer[sample_idx * 2 + 1] += sample_r * amp;
            }
        }
    }

    /// Starts a new voice.
    #[allow(clippy::missing_panics_doc)] // this function should not panic
    pub fn start_voice(
        &mut self,
        note: u8,
        envelope: Option<AdsrEnvelope>,
    ) -> &mut Voice {
        let sr = unsafe { SAMPLE_RATE };
        let mut new_voice = Voice {
            id: self.next_voice_id(),
            note,
            envelope: envelope.unwrap_or_default(),
            releasing: false,
            generator: Generator::Saw(Phasor::new(
                note_to_freq(note as f64),
                sr,
            )),
            // generator: Generator::Noise,
        };

        new_voice.envelope.set_trigger(true);

        // is there a free voice?
        if let Some(free_idx) =
            self.voices.iter().position(|voice| voice.is_none())
        {
            self.voices[free_idx] = Some(new_voice);
            return self.voices[free_idx].as_mut().unwrap();
        }

        // as we know voices are in use, we can use unwrap_unchecked()
        // to avoid some unnecessary checks.
        let oldest_voice = unsafe {
            self.voices
                .iter_mut()
                .min_by_key(|voice| voice.as_ref().unwrap_unchecked().id)
                .unwrap_unchecked()
        };

        *oldest_voice = Some(new_voice);
        return oldest_voice.as_mut().unwrap();
    }

    /// Starts a voice's release stage.
    pub fn start_release_for_voice(&mut self, voice_id: Option<u64>, note: u8) {
        for voice in &mut self.voices {
            match voice {
                Some(Voice {
                    id: candidate_id,
                    note: candidate_note,
                    releasing,
                    envelope,
                    ..
                }) if voice_id == Some(*candidate_id)
                    || note == *candidate_note =>
                {
                    *releasing = true;
                    envelope.set_trigger(false);
                }
                _ => (),
            }
        }
    }

    /// Starts the release stage for all active voices.
    pub fn start_release_for_active_voices(&mut self) {
        self.voices.iter_mut().for_each(|v| {
            if let Some(voice) = v {
                voice.releasing = true;
                voice.envelope.set_trigger(false);
            }
        });
    }

    /// Immediately terminates all active voices.
    pub fn kill_active_voices(&mut self) {
        self.voices.iter_mut().for_each(|v| {
            if v.is_some() {
                *v = None;
            }
        });
    }

    /// Terminates all voices which are releasing and which have an
    /// idle envelope.
    pub fn terminate_finished_voices(&mut self) {
        for voice in &mut self.voices {
            match voice {
                Some(v) if v.releasing && v.envelope.is_idle() => {
                    *voice = None;
                }
                _ => (),
            }
        }
    }

    /// Returns whether there is at least one voice active or not.
    pub fn is_voice_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_some())
    }

    fn next_voice_id(&mut self) -> u64 {
        self.id_counter = self.id_counter.wrapping_add(1);
        self.id_counter
    }
}
