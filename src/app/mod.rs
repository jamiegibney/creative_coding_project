use crate::prelude::*;
use nannou::prelude::*;
use nannou::LoopMode::RefreshSync;
use nannou_audio;
use nannou_audio::Buffer;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod model;
mod audio;
mod key_press;
mod mouse;
mod view;

pub use model::Model;

/// Runs the app via Nannou.
pub fn run_app() {
    nannou::app(model::model)
        .loop_mode(RefreshSync)
        .update(update)
        .run();
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}
