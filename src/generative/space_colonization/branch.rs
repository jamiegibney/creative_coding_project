use super::*;

use nannou::noise::Worley;

#[derive(Clone)]
pub struct Branch {
    parent: Option<Box<Branch>>,
    pos: DVec2,
    dir: DVec2,
    count: usize,
    save_dir: DVec2,
    len: f64,
    branched: bool,
    gen: usize,
    move_pos: DVec2,
}

impl Branch {
    pub fn new(
        pos: Option<DVec2>,
        dir: Option<DVec2>,
        len: f64,
        parent: Option<Box<Branch>>,
        gen: usize,
    ) -> Self {
        let mut new_pos;
        let mut new_dir;
        let mut new_parent;

        if let Some(parent) = parent {
            new_parent = Some(parent);
            new_pos = pos.unwrap_or(parent.next());
            new_dir = dir.unwrap_or(parent.next());
        }
        else {
            new_parent = None;
            new_pos = pos.unwrap_or(DVec2::ZERO);
            new_dir = dir.unwrap_or(DVec2::ZERO);
        }

        Self {
            parent,
            pos: new_pos,
            dir: new_dir,
            count: 0,
            save_dir: new_dir,
            len,
            branched: false,
            gen,
            move_pos: new_pos,
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.dir = self.save_dir;
    }

    pub fn next(&self) -> DVec2 {
        DVec2::new(
            self.dir.x * self.len + self.pos.x,
            self.dir.y * self.len + self.pos.y,
        )
    }
}
