use crate::app::audio::NoteHandler;
use std::sync::{Arc, Mutex};

pub use crate::app::audio::NoteEvent;
pub use crate::settings::*;
pub use crate::util::*;
pub use nannou::rand::{random_f64, random_range};
pub use std::f64::consts::{PI, TAU};
// pub use crate::util::{
//     db_to_level, freq_to_note, level_to_db, map, normalise, note_to_freq,
//     scale, within_tolerance,
// };

pub const MINUS_INFINITY_DB: f64 = -100.0;
pub const MINUS_INFINITY_GAIN: f64 = 1e-5;

pub type NoteHandlerRef = Arc<Mutex<NoteHandler>>;
