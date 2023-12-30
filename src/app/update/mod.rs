use super::*;
use std::sync::{Arc, Mutex, RwLock};

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.update_input_data(app, update);
    model.update_vectors(app);
    model.increment_mask_scan_line();
    model.update_filters();

    // update the mask scan line based on mouse events
    if model.ui_components.mask_resolution.is_open()
        && model.input_data.is_left_clicked
    {
        model.mouse_clicked_outside_of_mask = true;
    }
    model.update_mask_scan_line_from_mouse();

    let input_data = &model.input_data;

    // ui components
    model.ui_components.update(app, input_data);

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

    model.update_filter_line();
}
