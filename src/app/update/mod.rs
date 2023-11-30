use super::*;

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.increment_mask_scan_line();
    model.contours.update();

    if let Ok(mut mask) = model.spectral_mask.lock() {
        model
            .contours
            .column_to_mask(&mut mask, model.mask_scan_line_pos);
    }

}
