use super::*;
use super::audio::AudioModel;
use super::view::view;

/// The app's model, i.e. its general state.
pub struct Model {
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    _window: window::Id,
}

pub fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1400, 800)
        .key_pressed(key_press::key_pressed)
        .mouse_moved(mouse::mouse_moved)
        .view(view)
        .build()
        .unwrap();

    let audio_host = nannou_audio::Host::new();

    let mut audio_model = AudioModel::new();
    audio_model.initialise();

    let sample_rate = audio_model.sample_rate();

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::audio)
        .channels(2)
        .sample_rate(sample_rate as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model { audio_stream: stream, _window }
}
