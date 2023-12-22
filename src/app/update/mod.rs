use super::*;
use std::sync::{Arc, Mutex, RwLock};

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.update_input_data(app, update);
    model.increment_mask_scan_line();

    let egui_ctx = model.egui.begin_frame();
    let pos = model.mask_scan_line_pos;
    let mask = Arc::clone(&model.spectral_mask);

    let input_data = model.input_data;

    if let Ok(algo) = model.ui_params.mask_algorithm.read() {
        match *algo {
            GenerativeAlgo::Contours => {
                let ctr = Arc::clone(model.contours.as_mut().unwrap());

                model.mask_thread_pool.execute(move || {
                    let mut mask = mask.lock().unwrap();
                    let mut ctr = ctr.write().unwrap();

                    ctr.update(&input_data);
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

                    sml.update(&input_data);
                    sml.column_to_mask(mask.input_buffer(), pos);
                    drop(sml);
                    mask.publish();
                });
            }
        }
    }

    model.ui_components.update(&input_data);
}
