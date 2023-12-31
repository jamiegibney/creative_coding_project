pub mod contours;
pub mod smooth_life;
pub mod vectors;
pub mod voronoi;

pub use contours::{Contours, ContoursGPU};
pub use smooth_life::{SmoothLife, SmoothLifeGPU};
pub use vectors::Vectors;
pub use voronoi::VoronoiGPU;

// Note: perlin noise is already supported in Nannou
