pub mod contours;
pub mod smooth_life;
pub mod voronoi;

pub use contours::{Contours, ContoursGPU};
pub use smooth_life::{SmoothLife, SmoothLifeGPU};

// Note: perlin noise is already supported in Nannou
