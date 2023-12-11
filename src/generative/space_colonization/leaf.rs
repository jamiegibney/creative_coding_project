use super::*;

pub struct Leaf {
    pos: DVec2,
    reached: bool,
}

impl Leaf {
    pub fn new(area: DVec3) -> Self {
        let mut s = Self { pos: dvec2(area.x, area.y), reached: false };

        let start_offset = DVec2::new(
            scale(random_f64(), -1.0, 1.0),
            scale(random_f64(), -1.0, 1.0),
        ) * area.z;

        s.pos += start_offset;
        s.pos.clamp(dvec2(-1.0, -1.0), dvec2(1.0, 1.0));

        s
    }

    pub fn show(&self) {
        todo!()
    }
}
