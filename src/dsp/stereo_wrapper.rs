use super::Effect;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct StereoWrapper<E: Effect> {
    pair: (E, E),
}

impl<E: Effect + Clone> StereoWrapper<E> {
    pub fn from_single(effect: E) -> Self {
        Self { pair: (effect.clone(), effect) }
    }

    pub fn from_pair(effect_l: E, effect_r: E) -> Self {
        Self { pair: (effect_l, effect_r) }
    }
}

impl<E: Effect> Deref for StereoWrapper<E> {
    type Target = (E, E);

    fn deref(&self) -> &Self::Target {
        &self.pair
    }
}

impl<E: Effect> DerefMut for StereoWrapper<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pair
    }
}

impl<E: Effect + Clone> Effect for StereoWrapper<E> {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let out_l = self.pair.0.process_mono(in_l);
        let out_r = self.pair.1.process_mono(in_r);

        (out_l, out_r)
    }

    fn process_mono(&mut self, input: f64) -> f64 {
        unimplemented!(
            "StereoWrapper does not currently support processing to mono"
        )
    }

    fn get_sample_rate(&self) -> f64 {
        self.pair.0.get_sample_rate()
    }
}
