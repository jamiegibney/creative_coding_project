//! State used by the SmoothLife algorithms.

use atomic::Atomic;

use super::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SmoothLifeState {
    pub ri: f32,
    pub ra: f32,

    pub alpha_n: f32,
    pub alpha_m: f32,

    pub b1: f32,
    pub b2: f32,

    pub d1: f32,
    pub d2: f32,

    pub dt: f32,
    pub delta_time: f32,

    pub width: u32,
    pub height: u32,
    pub should_randomize: u32,
}

impl SmoothLifeState {
    pub fn flow(w: u32, h: u32) -> Self {
        Self {
            ri: 10.0 / 3.0,
            ra: 10.0,

            alpha_n: 0.033,
            alpha_m: -7.947,

            b1: -0.138,
            b2: 0.265,

            d1: 0.167,
            d2: 0.445,

            dt: 2.0,
            delta_time: 0.0,

            width: w,
            height: h,
            should_randomize: 1,
        }
    }

    pub fn slime(w: u32, h: u32) -> Self {
        Self {
            ri: 3.667,
            ra: 11.0,

            alpha_n: 0.198,
            alpha_m: 0.347,

            b1: 0.078,
            b2: 0.265,
            d2: 0.845,
            d1: 0.167,

            dt: 2.0,
            delta_time: 0.0,

            width: w,
            height: h,
            should_randomize: 1,
        }
    }

    pub fn corrupt(w: u32, h: u32) -> Self {
        Self {
            ri: 3.667,
            ra: 11.0,

            alpha_n: 0.028,
            alpha_m: 0.147,

            b1: 0.278,
            b2: 0.365,
            d1: 0.267,
            d2: 0.445,

            dt: 2.0,
            delta_time: 0.0,

            width: w,
            height: h,
            should_randomize: 1,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}

pub struct SmoothLifeStateAtomic {
    pub speed: Arc<AtomicF32>,
    pub ra: Arc<AtomicF32>,
    pub preset: Arc<Atomic<SmoothLifePreset>>,
    pub should_reset: Arc<AtomicBool>,
    pub should_update_preset: AtomicBool,
}

impl Default for SmoothLifeStateAtomic {
    fn default() -> Self {
        Self {
            speed: Arc::new(AtomicF32::new(8.0)),
            ra: Arc::new(AtomicF32::new(32.0)),
            preset: Arc::new(Atomic::new(SmoothLifePreset::default())),
            should_reset: Arc::new(AtomicBool::new(true)),
            should_update_preset: AtomicBool::new(true),
        }
    }
}
