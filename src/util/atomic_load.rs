use atomic_float::AtomicF64;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering::Relaxed};

/// Trait for shorthand implementation of Relaxed atomic load and store operations.
pub trait AtomicLoad: Default {
    type NonAtomic: Default;

    /// Shorthand method for `self.load(Relaxed)`.
    fn lr(&self) -> Self::NonAtomic;
    /// Shorthand method for `self.store(value, Relaxed)`.
    fn sr(&self, value: Self::NonAtomic);
}

impl AtomicLoad for AtomicI32 {
    type NonAtomic = i32;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicLoad for AtomicUsize {
    type NonAtomic = usize;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicLoad for AtomicBool {
    type NonAtomic = bool;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}

impl AtomicLoad for AtomicF64 {
    type NonAtomic = f64;

    fn lr(&self) -> Self::NonAtomic {
        self.load(Relaxed)
    }

    fn sr(&self, value: Self::NonAtomic) {
        self.store(value, Relaxed);
    }
}
