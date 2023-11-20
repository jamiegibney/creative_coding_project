use super::*;

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.contours.update();

    if let Ok(mut mask) = model.spectral_mask.lock() {
        model.contours.column_to_mask(&mut mask, 0.5);
    }
}
