use super::{audio::AudioModel, *};
use crate::dsp::filters::biquad::FilterType;
use crate::musical::Note;

/// Function for handling keypresses.
pub fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    if key == Key::Space {
        if model.audio_stream.is_playing() {
            model.audio_stream.pause().unwrap();
        }
        else {
            model.audio_stream.play().unwrap();
        }
    }

    let note = Note::from_key(&key);
    model.note = note;

    if let Some(note) = note {
        model.audio_senders.filter_freq = 
        model.audio_senders.envelope_trigger.send(true).unwrap();
    }
}

/// Function for handling key releases.
pub fn key_released(_app: &App, model: &mut Model, key: Key) {
    if model.note.is_none() {
        model.audio_senders.envelope_trigger.send(false).unwrap();
    }
}
