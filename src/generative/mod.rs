pub mod contours;
pub mod l_system;
pub mod smooth_life;
pub mod voronoi;
// pub mod space_colonization;

pub use contours::{Contours, ContoursGPU};
pub use smooth_life::{SmoothLife, SmoothLifeGPU};

// Note: perlin noise is already supported in Nannou
