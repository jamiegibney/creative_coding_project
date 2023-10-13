//! Module for signal generation.
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct SawGenerator {
    //
}

#[derive(Debug, Clone, Copy)]
pub enum Generator {
    Saw(SawGenerator),
}

// pub trait Generator: dyn_clone::DynClone {}

// dyn_clone::clone_trait_object!(Generator);
