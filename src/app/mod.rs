use crate::prelude::*;
use nannou::prelude::*;
use nannou::LoopMode::RefreshSync;
use nannou_audio;
use nannou_audio::Buffer;
use nannou::rand::rngs::SmallRng;
use nannou::rand::{Rng, SeedableRng};

pub mod audio;
mod key;
mod model;
mod mouse;
mod view;
mod update;

pub use model::Model;
use update::update;

/// Runs the app via Nannou.
pub fn run_app() {
    nannou::app(model::Model::build)
        .loop_mode(RefreshSync)
        .update(update)
        .run();
}
