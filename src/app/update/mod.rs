use super::*;
use std::sync::{Arc, Mutex, RwLock};

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.update_input_data(app, update);
    model.update_vectors(app);
    model.increment_mask_scan_line();

    let input_data = &model.input_data;
    let mouse_pos = input_data.mouse_pos;

    // ui components
    model.ui_components.update(app, input_data);

    // spectral mask
    if model.mask_rect.contains(mouse_pos) && model.input_data.is_left_clicked {
        let x_pos = mouse_pos.x as f64;
        let l = model.mask_rect.left() as f64;
        let r = model.mask_rect.right() as f64;

        model.mask_scan_line_pos = normalise(x_pos, l, r);
    }

    let pos = model.mask_scan_line_pos;
    let mask_len = model.ui_params.mask_resolution.lr().value();

    match model.ui_params.mask_algorithm.lr() {
        GenerativeAlgo::Contours => {
            let ctr = Arc::clone(model.contours.as_mut().unwrap());

            let mut ctr = ctr.write().unwrap();

            ctr.update(app, input_data);
            ctr.column_to_mask(
                model.spectral_mask.input_buffer(),
                mask_len,
                pos,
            );

            drop(ctr);
            model.spectral_mask.publish();
        }
        GenerativeAlgo::SmoothLife => {
            let sml = Arc::clone(&model.smooth_life.as_mut().unwrap());

            let mut sml = sml.write().unwrap();

            sml.update(app, input_data);
            sml.column_to_mask(
                model.spectral_mask.input_buffer(),
                mask_len,
                pos,
            );

            drop(sml);
            model.spectral_mask.publish();
        }
    }
}
