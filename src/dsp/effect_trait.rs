/// Generic trait for audio processing effects.
pub trait Effect: dyn_clone::DynClone + Send {
    /// Method to process two stereo samples of audio.
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        (in_l, in_r)
    }

    /// Method to process a single sample of audio.
    fn process_mono(&mut self, input: f64) -> f64 {
        input
    }
}

dyn_clone::clone_trait_object!(Effect);

// TODO how can trait objects derive Debug?
