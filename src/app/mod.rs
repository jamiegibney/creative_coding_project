use crate::prelude::*;
use nannou::prelude::*;
use nannou::LoopMode::RefreshSync;
use nannou_audio;

pub mod audio;
mod key;
mod model;
mod mouse;
mod update;
mod view;

pub use model::{GenerativeAlgo, Model};
use update::update;

/// Runs the app via Nannou.
pub fn run_app() {
    nannou::app(model::Model::build)
        .loop_mode(RefreshSync)
        .update(update)
        .run();
}
