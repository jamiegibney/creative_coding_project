use super::model::model;
use super::{audio::AudioModel, *};
use crate::dsp::filters::FilterType;
use crate::musical::*;

/// Function for handling keypresses.
pub fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.audio_stream.is_playing() {
                model.audio_stream.pause().unwrap();
            }
            else {
                model.audio_stream.play().unwrap();
            }
        }
        Key::Z => {
            model.octave.decrease();
        }
        Key::X => {
            model.octave.increase();
        }
        _ => (),
    };

    let note = Note::from_key(&key);
    model.note = note;

    if let Some(note) = note {
        model
            .audio_senders
            .filter_freq
            .send(note_to_freq(midi_note_value_from(model.octave, note) as f64))
            .unwrap();
        model.audio_senders.envelope_trigger.send(true).unwrap();
    }
}

/// Function for handling key releases.
pub fn key_released(_app: &App, model: &mut Model, key: Key) {
    if KEYBOARD_MUSICAL_NOTES.contains(&key) {
        model.audio_senders.envelope_trigger.send(false).unwrap();
    }
}
