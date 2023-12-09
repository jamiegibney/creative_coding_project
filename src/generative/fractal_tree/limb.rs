use super::Branch;

pub struct Limb {
    param_index: usize,
    limbs: Vec<Limb>,
    branches: Vec<Branch>,
    len: f64,
    w: f64,
    x: f64,
    y: f64,
    freq: f64,
    decay: f64,
    amp: f64,
    detune: f64,
    sway: f64,
    is_growing: bool,
    parent: Option<Box<Limb>>,
    base_branch_index: usize,
}

impl Limb {
    pub fn new(
        base_branch: Option<&Branch>,
        index: usize,
        parent: Option<usize>,
        base_branch_index: usize,
    ) -> Self {
        let mut limbs = Vec::new();
        let mut branches = Vec::new();
        if let Some(&branch) = base_branch {
            branches.push(branch.clone());
        }

        Self {
            param_index: index,
            limbs,
            branches,
            len: 0.0,
            w: 0.0,
            x: 0.0,
            y: 0.0,
            freq: 0.0,
            decay: 0.0,
            amp: 0.0,
            detune: 0.0,
            sway: 0.0,
            is_growing: true,
            parent,
            base_branch_index,
        }
    }
}
