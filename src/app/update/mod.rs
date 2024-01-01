use super::*;
use std::sync::{Arc, Mutex, RwLock};

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.update_input_data(app, update);

    if !model.input_data.is_win_focussed {
        return;
    }

    model.update_vectors(app);
    model.increment_mask_scan_line();
    model.update_filters();

    // update the mask scan line based on mouse events
    if model.ui_components.mask_resolution.is_open()
        && model.input_data.is_left_clicked
        && model
            .ui_components
            .mask_resolution
            .rect()
            .contains(model.input_data.mouse_pos)
    {
        model.mouse_clicked_outside_of_mask = true;
    }
    model.update_mask_scan_line_from_mouse();

    // let input_data = &model.input_data;

    // ui components
    model.ui_components.update(app, &model.input_data);

    let pos = model.mask_scan_line_pos;
    let mask_len = model.ui_params.mask_resolution.lr().value();

    match model.ui_params.mask_algorithm.lr() {
        GenerativeAlgo::Contours => {
            let mut ctr = model.contours.write().unwrap();

            ctr.update(app, &model.input_data);
            ctr.column_to_mask(
                model.spectral_mask.input_buffer(),
                mask_len,
                pos,
            );
        }
        GenerativeAlgo::SmoothLife => {
            let mut sml = model.smooth_life.write().unwrap();

            sml.update(app, &model.input_data);
            sml.column_to_mask(
                model.spectral_mask.input_buffer(),
                mask_len,
                pos,
            );
        }
        GenerativeAlgo::Voronoi => {
            model.update_voronoi_vectors(app);

            let mut vrn = model.voronoi_mask.write().unwrap();
            let vv = model.voronoi_vectors.read().unwrap();

            vrn.copy_from_vectors(&vv);
            vrn.set_weight(model.ui_params.voronoi_border_weight.lr());

            vrn.update(app, &model.input_data);
            vrn.column_to_mask(
                model.spectral_mask.input_buffer(),
                mask_len,
                pos,
            );
        }
    }

    model.spectral_mask.publish();

    model.voronoi_reso_bank.update(app, &model.input_data);

    model.update_filter_line();
    model.update_filter_nodes(app);
}
