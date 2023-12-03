use super::*;
use std::sync::{Arc, Mutex, RwLock};

pub fn update(app: &App, model: &mut Model, update: Update) {
    let dt = model.get_delta_time();
    model.increment_mask_scan_line();

    // if let Ok(mut mask) = model.spectral_mask.lock() {
    let pos = model.mask_scan_line_pos;

    let mask = Arc::clone(&model.spectral_mask);

    match model.current_gen_algo {
        GenerativeAlgo::Contours => {
            let ctr = Arc::clone(model.contours.as_mut().unwrap());

            model.mask_thread_pool.execute(move || {
                let mut mask = mask.lock().unwrap();
                let mut ctr = ctr.write().unwrap();

                ctr.update(dt);
                ctr.column_to_mask(mask.input_buffer(), pos);
                drop(ctr);
                mask.publish();
            });
        }
        GenerativeAlgo::SmoothLife => {
            let sml = Arc::clone(model.smooth_life.as_mut().unwrap());

            model.mask_thread_pool.execute(move || {
                let mut mask = mask.lock().unwrap();
                let mut sml = sml.write().unwrap();

                sml.update(dt);
                sml.column_to_mask(mask.input_buffer(), pos);
                drop(sml);
                mask.publish();
            });
        }
    }
    // }
}
