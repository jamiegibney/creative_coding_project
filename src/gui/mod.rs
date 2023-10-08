pub mod spectrum;

mod gradient;
mod rdp;

use crate::prelude::*;
use earcutr::earcut;
use nannou_egui;
use nannou_egui::egui as egui;
use egui::{
    lerp, pos2, vec2, Color32, Mesh, Pos2, Rgba, Sense, Shape, Ui, Vec2,
};
use rdp::decimate_points;
