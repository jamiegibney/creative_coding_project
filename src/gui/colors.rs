use nannou::prelude::*;
use std::marker::PhantomData;

/// Big text labels.
pub const BIG_LABEL: Rgb =
    Rgb { red: 0.35, green: 0.35, blue: 0.35, standard: PhantomData };

/// Value text labels.
pub const LABEL: Rgb =
    Rgb { red: 0.5, green: 0.5, blue: 0.5, standard: PhantomData };

/// Non-selected text labels.
pub const NON_SELECTED: Rgb =
    Rgb { red: 0.7, green: 0.7, blue: 0.7, standard: PhantomData };

/// Non-selected background (main background color).
pub const BG_NON_SELECTED: Rgb =
    Rgb { red: 0.18, green: 0.18, blue: 0.18, standard: PhantomData };

/// Hovered-over text labels.
pub const HOVERED: Rgb =
    Rgb { red: 0.85, green: 0.85, blue: 0.85, standard: PhantomData };

/// Hovered-over background.
pub const BG_HOVERED: Rgb =
    Rgb { red: 0.35, green: 0.35, blue: 0.35, standard: PhantomData };

/// Selected text labels.
pub const SELECTED: Rgb =
    Rgb { red: 1.0, green: 1.0, blue: 1.0, standard: PhantomData };

/// Selected background.
pub const BG_SELECTED: Rgb =
    Rgb { red: 0.5, green: 0.5, blue: 0.5, standard: PhantomData };

/// Value text labels.
pub const VALUE: Rgb =
    Rgb { red: 1.0, green: 1.0, blue: 1.0, standard: PhantomData };
