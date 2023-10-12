pub mod adsr;
pub mod delay;
pub mod distortion;
pub mod dynamics;
pub mod filters;
pub mod ramp;
pub mod ring_buffer;
pub mod spectral;
pub mod synthesis;
pub mod width;

pub mod effect_trait;

use crate::prelude::*;
pub use effect_trait::Effect;
pub use adsr::AdsrEnvelope;
pub use distortion::Waveshaper;
pub use filters::{
    allpass::Allpass,
    biquad::{BiquadFilter, BiquadParams},
    comb::{FirCombFilter, IirCombFilter},
    first_order::FirstOrderFilter,
    Filter, FilterType, BUTTERWORTH_Q,
};
pub use ramp::Ramp;
pub use ring_buffer::RingBuffer;
pub use synthesis::{
    Generator,
};
