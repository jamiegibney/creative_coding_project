//! Second-order biquad filter form supporting various filter types.

#![allow(unused, clippy::must_use_candidate)]
mod filter;
use super::{Filter, FilterType};
use crate::util;

pub use filter::*;

#[cfg(test)]
mod tests {
    use super::{filter::*, *};

    // filter bounds tests

    #[test]
    #[should_panic]
    fn bad_freq_argument_1() {
        let sample_rate = 44100.0;
        let mut filter = BiquadFilter::new(sample_rate);

        filter.set_freq(sample_rate);
    }

    #[test]
    #[should_panic]
    fn bad_freq_argument_2() {
        let sample_rate = 44100.0;
        let mut filter = BiquadFilter::new(sample_rate);

        filter.set_params(&BiquadParams {
            freq: sample_rate / 2.0 + 0.00001,
            ..Default::default()
        });
    }

    #[test]
    #[should_panic]
    fn bad_q_argument() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_q(-1.32);
    }

    // filter bandpass/notch tests
    #[test]
    #[should_panic]
    fn incompatible_half_power_types() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_type(FilterType::Peak);
        let _ = filter.bp_notch_half_power_points();
    }

    #[test]
    fn test_half_power_points() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_params(&BiquadParams {
            freq: 1000.0,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Bandpass,
        });

        let (f_min, f_max) = filter.bp_notch_half_power_points();

        let f_min_correct = util::within_tolerance(f_min, 618.0, 2.0);
        let f_max_correct = util::within_tolerance(f_max, 1618.0, 2.0);

        assert!(f_min_correct && f_max_correct);

        filter.set_q(0.25);

        let (f_min, f_max) = filter.bp_notch_half_power_points();

        let f_min_correct = util::within_tolerance(f_min, 236.0, 2.0);
        let f_max_correct = util::within_tolerance(f_max, 4236.0, 2.0);

        assert!(f_min_correct && f_max_correct);
    }

    #[test]
    fn test_bandwidth() {
        let mut filter = BiquadFilter::new(44100.0);
        filter.set_params(&BiquadParams {
            freq: 1000.0,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Bandpass,
        });

        assert!(util::within_tolerance(
            filter.bp_notch_bandwidth(),
            1000.0,
            0.1
        ));

        filter.set_q(0.25);

        assert!(util::within_tolerance(
            filter.bp_notch_bandwidth(),
            4000.0,
            0.1
        ));
    }
}
