/// Generic trait for audio effects.
pub trait Effect: dyn_clone::DynClone + Send {
    fn process(&mut self, sample: f64) -> f64;
}

dyn_clone::clone_trait_object!(Effect);

// TODO how can trait objects derive Debug?
