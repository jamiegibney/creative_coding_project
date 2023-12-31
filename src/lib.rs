#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::return_self_not_must_use,
    unused
)]
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
// pub mod musical;

// GUI stuff
pub mod gui;

// Generative algorithms
pub mod generative;

pub mod fonts;

pub mod simd;
