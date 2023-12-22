use crate::prelude::*;
use nannou::prelude::*;
use nannou::LoopMode::RefreshSync;
use nannou_audio;

pub mod audio;
pub mod event;
mod key;
mod model;
mod mouse;
pub mod musical;
pub mod params;
pub mod update;
pub mod view;

use event::event;
pub use model::Model;
pub use musical::*;
pub use params::*;
use update::update;

/// Runs the app via Nannou.
pub fn run_app() {
    nannou::app(model::Model::build)
        .loop_mode(RefreshSync)
        .update(update)
        .run();
}
