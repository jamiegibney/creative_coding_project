pub mod spectrum;
pub use spectrum::*;

// mod gradient;
mod rdp;
pub mod mesh;

use crate::prelude::*;
use earcutr::earcut;
use rdp::decimate_points;
