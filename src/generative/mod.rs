pub mod contours;
pub mod smooth_life;
pub mod voronoi;
pub mod vectors;

pub use contours::{Contours, ContoursGPU};
pub use smooth_life::{SmoothLife, SmoothLifeGPU};
pub use vectors::Vectors;

// Note: perlin noise is already supported in Nannou
