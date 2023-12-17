use super::*;

fn egui(val: &mut STR) {
    let mut s = egui::Slider::new(&mut val.value, 0.0..=1.0)
        .suffix(" %")
        .fixed_decimals(1);
}

struct STR {
    value: f64,
}

impl STR {
    pub fn new() -> Self {
        Self { value: 0.34 }
    }
}
