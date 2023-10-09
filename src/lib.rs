#![allow(clippy::module_name_repetitions, clippy::wildcard_imports)]
// GUI and program related
pub mod app;

// Signal processing
pub mod dsp;

// General utilities
pub mod util;

// Some widely-used re-exports
pub mod prelude;

// Program-wide settings
pub mod settings;

// Musical systems and structures
pub mod musical;

// GUI stuff
pub mod gui;

// L-system related
pub mod l_system;

// RNG utility functions
pub use nannou::rand;
