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
