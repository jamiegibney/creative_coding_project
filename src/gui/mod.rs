pub mod spectrum;

// mod gradient;
mod rdp;
pub mod mesh;

use crate::prelude::*;
use earcutr::earcut;
// use egui::{
//     lerp, pos2, vec2, Color32, Mesh, Pos2, Rgba, Sense, Shape, Ui, Vec2,;
// };
use rdp::decimate_points;
