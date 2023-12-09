use super::*;

pub struct Tree {
    trunk: Option<Limb>,
    leaves: Vec<Leaf>,
    len: u32,
    trunk_complete: bool,
    current_trunk: Branch,
    matured: bool,
    total_branches: u32,
    total_limbs: u32,
    limb_thresh: u32,
    decay: bool,
    exists: bool,
}

impl Tree {
    pub fn new(
        len: u32,
        leaf_area: f64,
        num_leaves: usize,
        root_start: u32,
        root_dir: u32,
        limb_thresh: u32,
    ) -> Self {
        let root = Branch::new(root_start, root_dir, len, None, 0);

        Self {
            trunk: Some(Limb::new(Some(&root), 0, None, None)),
            leaves: vec![Leaf::new(leaf_area); num_leaves],
            len,
            trunk_complete: false,
            current_trunk: root,
            matured: true,
            total_branches: 0,
            total_limbs: 0,
            limb_thresh,
            decay: false,
            exists: false,
        }
    }

    pub fn grow_trunk(&mut self) {
        let new_branch = Branch::new(
            None, None, self.len, self.current_trunk, self.total_branches,
        );
        self.total_branches += 1;
        self.trunk.branches.push(new_branch);
        self.current_trunk = new_branch;
    }
}
