use crate::dsp::filters::biquad::FilterType;

use super::{audio::AudioModel, *};

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
    if key == Key::J && model.audio_stream.is_playing() {
        model
            .audio_stream
            .send(|model: &mut AudioModel| {
                model.filter.suspend();
                model.filter_2.suspend();
            })
            .unwrap();
    }
    if key == Key::K && model.audio_stream.is_playing() {
        model
            .audio_stream
            .send(|model: &mut AudioModel| {
                model.filter.force_recompute();
                model.filter_2.force_recompute();
            })
            .unwrap();
    }
    let filter_type = match key {
        Key::A => Some(FilterType::Allpass),
        Key::L => Some(FilterType::Lowpass),
        Key::H => Some(FilterType::Highpass),
        Key::B => Some(FilterType::Bandpass),
        Key::P => Some(FilterType::Peak),
        Key::N => Some(FilterType::Notch),
        _ => None,
    };

    if key == Key::G {
        model.envelope_sender.send(true).unwrap();
    }

    model
        .audio_stream
        .send(move |model: &mut AudioModel| {
            if let Some(filter_type) = filter_type {
                model.filter.set_type(filter_type);
                model.filter_2.set_type(filter_type);
            }
        })
        .unwrap();
}

/// Function for handling key releases.
pub fn key_released(_app: &App, model: &mut Model, key: Key) {
    if key == Key::G {
        model.envelope_sender.send(false).unwrap();
    }
}
