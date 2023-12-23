use crate::app::*;
use std::fmt::{Debug, Display};

/// A trait for preparing an enum for use with the [`Menu`] UI component.
///
/// The enum must also implement:
/// - [`Display`], so its name can be printed in the menu
/// - [`Clone`] and [`Copy`], so the value can be returned from the menu
/// - [`Default`], so the menu knows which variant to use when created
/// - [`Debug`], so the whole menu can be debugged
///
/// Please see the documentation of all the required methods for more information.
pub trait MenuEnum: Display + Clone + Copy + Default + Debug {
    /// The number of variants in the enum, used to layout the menu.
    ///
    /// ***Note***: you can technically set this to less than the actual number of
    /// variants to truncate the menu. Setting a value greater than the number of
    /// variants, however, will cause the menu to panic when it is created.
    fn num_variants() -> usize;
    /// The index for a given variant. Used to avoid having to iterate through all variants
    /// each frame. You should ensure that this matches the layout of [`from_idx()`](MenuEnum::from_idx).
    fn idx(&self) -> usize;
    /// The variant from a given index. This method should be used to describe the
    /// order of elements in the menu.
    fn from_idx(idx: usize) -> Option<Self>;
}

impl MenuEnum for GenerativeAlgo {
    fn num_variants() -> usize {
        2
    }

    fn idx(&self) -> usize {
        match self {
            Self::Contours => 0,
            Self::SmoothLife => 1,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Contours),
            1 => Some(Self::SmoothLife),
            _ => None,
        }
    }
}

impl MenuEnum for SmoothLifePreset {
    fn num_variants() -> usize {
        2
    }

    fn idx(&self) -> usize {
        match self {
            Self::Fluid => 0,
            Self::Swirl => 1,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Fluid),
            1 => Some(Self::Swirl),
            _ => None,
        }
    }
}

impl MenuEnum for SpectrogramView {
    fn num_variants() -> usize {
        3
    }

    fn idx(&self) -> usize {
        match self {
            Self::PrePost => 0,
            Self::PreOnly => 1,
            Self::PostOnly => 2,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::PrePost),
            1 => Some(Self::PreOnly),
            2 => Some(Self::PostOnly),
            _ => None,
        }
    }
}

impl MenuEnum for DistortionType {
    fn num_variants() -> usize {
        5
    }

    fn idx(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Soft => 1,
            Self::Hard => 2,
            Self::Wrap => 3,
            Self::Downsample => 4,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::None),
            1 => Some(Self::Soft),
            2 => Some(Self::Hard),
            3 => Some(Self::Wrap),
            4 => Some(Self::Downsample),
            _ => None,
        }
    }
}

impl MenuEnum for Scale {
    fn num_variants() -> usize {
        5
    }

    fn idx(&self) -> usize {
        match self {
            Self::Major => 0,
            Self::Minor => 1,
            Self::MajPentatonic => 2,
            Self::MinPentatonic => 3,
            Self::Chromatic => 4,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Major),
            1 => Some(Self::Minor),
            2 => Some(Self::MajPentatonic),
            3 => Some(Self::MinPentatonic),
            4 => Some(Self::Chromatic),
            _ => None,
        }
    }
}

impl MenuEnum for SmoothLifeSize {
    fn num_variants() -> usize {
        6
    }

    fn idx(&self) -> usize {
        match self {
            Self::S16 => 0,
            Self::S32 => 1,
            Self::S64 => 2,
            Self::S128 => 3,
            Self::S256 => 4,
            Self::S512 => 5,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::S16),
            1 => Some(Self::S32),
            2 => Some(Self::S64),
            3 => Some(Self::S128),
            4 => Some(Self::S256),
            5 => Some(Self::S512),
            _ => None,
        }
    }
}

impl MenuEnum for SpectrogramSize {
    fn num_variants() -> usize {
        4
    }

    fn idx(&self) -> usize {
        match self {
            Self::S1024 => 0,
            Self::S2048 => 1,
            Self::S4096 => 2,
            Self::S8192 => 3,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::S1024),
            1 => Some(Self::S2048),
            2 => Some(Self::S4096),
            3 => Some(Self::S8192),
            _ => None,
        }
    }
}

impl MenuEnum for SpectralFilterSize {
    fn num_variants() -> usize {
        9
    }

    fn idx(&self) -> usize {
        match self {
            Self::S64 => 0,
            Self::S128 => 1,
            Self::S256 => 2,
            Self::S512 => 3,
            Self::S1024 => 4,
            Self::S2048 => 5,
            Self::S4096 => 6,
            Self::S8192 => 7,
            Self::S16384 => 8,
        }
    }

    fn from_idx(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::S64),
            1 => Some(Self::S128),
            2 => Some(Self::S256),
            3 => Some(Self::S512),
            4 => Some(Self::S1024),
            5 => Some(Self::S2048),
            6 => Some(Self::S4096),
            7 => Some(Self::S8192),
            8 => Some(Self::S16384),
            _ => None,
        }
    }
}
