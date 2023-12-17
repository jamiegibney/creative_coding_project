use crate::dsp::BUTTERWORTH_Q;

use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub enum LowBandType {
    #[default]
    /// Low-cut (AKA high-pass) filter type.
    LowCut,
    /// Low-shelf filter type with fixed Butterworth Q value.
    LowShelf,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MidBandType {
    #[default]
    /// Peak filter type with fixed Butterworth Q value.
    Peak,
    /// Notch filter type.
    Notch,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum HighBandType {
    #[default]
    /// High-cut (AKA low-pass) filter type.
    HighCut,
    /// High-shelf filter type with fixed Butterworth Q value.
    HighShelf,
}

/// Parameters for a three-band EQ with restricted filter types.
#[derive(Clone, Copy, Debug)]
pub struct EQParams {
    /// The filter type of the low EQ band.
    pub low_band_type: LowBandType,
    /// The cutoff frequency of the low EQ band in hertz.
    pub low_band_cutoff: Smoother<f64>,
    /// The gain/Q value of the low EQ band.
    pub low_band_gain: Smoother<f64>,

    /// The filter type of the mid EQ band.
    pub mid_band_type: MidBandType,
    /// The cutoff frequency of the mid EQ band in hertz.
    pub mid_band_cutoff: Smoother<f64>,
    /// The gain value of the mid EQ band — only applicable to the peak filter.
    pub mid_band_gain: Smoother<f64>,
    /// The Q value of the mid EQ band.
    pub mid_band_q: Smoother<f64>,

    /// The filter type of the high EQ band.
    pub high_band_type: HighBandType,
    /// The cutoff frequency of the high EQ band in hertz.
    pub high_band_cutoff: Smoother<f64>,
    /// The gain/Q value of the high EQ band.
    pub high_band_gain: Smoother<f64>,
}

impl Default for EQParams {
    fn default() -> Self {
        Self {
            low_band_cutoff: smoother(150.0),
            low_band_gain: smoother(0.0),

            mid_band_cutoff: smoother(500.0),
            mid_band_gain: smoother(0.0),
            mid_band_q: smoother(BUTTERWORTH_Q),

            high_band_cutoff: smoother(1500.0),
            high_band_gain: smoother(0.0),

            low_band_type: LowBandType::default(),
            mid_band_type: MidBandType::default(),
            high_band_type: HighBandType::default(),
        }
    }
}
