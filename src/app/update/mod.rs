use super::*;

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.contours.update();

    // TODO logarithmic frequency scaling for the mask
    // TODO only one column is read for the spectral mask at a time... so why
    // not copy the values from the noise so that any size can be represented,
    // rather than copying the pixel data? that way there's no pixel conversion,
    // and any FFT block size can be supported.
    // if let Some(col) = model.contours.column(128) {
    //     let mut mask = model.spectral_mask.lock().unwrap();
    //
    //     // mask.iter_mut()
    //     //     .zip(col.iter().rev())
    //     //     .for_each(|(msk, &col)| {
    //     //         let val = col[0] as f64 / u8::MAX as f64;
    //     //         *msk = 1.0 - val;
    //     //     });
    // }

    if let Ok(mut mask) = model.spectral_mask.lock() {
        model.contours.column_direct(&mut mask, 128);
    }
}
