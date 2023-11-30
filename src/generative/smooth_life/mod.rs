use crate::prelude::*;
use std::ops::{Add, Rem};

mod grid;
mod state;
use grid::Grid;
use state::SLState;

pub struct SmoothLife {
    state: SLState,
    grid: Grid,
    diff: Grid,
}

impl SmoothLife {
    pub fn new(size: usize) -> Self {
        Self {
            state: SLState::default(),
            grid: Grid::new_square(size),
            diff: Grid::new_square(size),
        }
    }

    pub fn update(&mut self) {
        self.compute();
        self.apply_diff();
    }

    // pub fn draw(&self, draw: &Draw) {
    //
    // }

    #[allow(clippy::many_single_char_names)]
    fn compute(&mut self) {
        let w = self.grid.width();
        let h = self.grid.height();
        let ri = self.state.radius_inner;
        let ra = self.state.radius_outer;

        // TODO refactor into cleaner rust
        for cx in 0..w {
            for cy in 0..h {
                let (mut m, mut m_norm, mut n, mut n_norm) =
                    (0.0, 0.0, 0.0, 0.0);
                let ra_1 = self.state.radius_outer - 1.0;
                let min = (-ra_1) as usize;
                let max = ra_1 as usize;

                for dx in min..=max {
                    for dy in min..=max {
                        let x = wrap(cx + dx, w);
                        let y = wrap(cy + dy, h);

                        let d = (dx * dx + dy * dy) as f64;

                        if d <= ri * ri {
                            m += self.grid[x][y];
                            m_norm += 1.0;
                        }
                        else if d <= ra * ra {
                            n += self.grid[x][y];
                            n_norm += 1.0;
                        }
                    }
                }

                n /= n_norm;
                m /= m_norm;

                let q = self.state.transition(n, m);
                self.diff[cx][cy] = 2.0f64.mul_add(q, -1.0);
            }
        }
    }

    fn apply_diff(&mut self) {
        let dt = self.state.dt;

        for (gr, df) in self.grid.iter_mut().zip(self.diff.iter()) {
            for (grid, &diff) in gr.iter_mut().zip(df.iter()) {
                *grid = dt.mul_add(diff, *grid).clamp(0.0, 1.0);
            }
        }
    }
}

fn wrap<T>(value: T, bound: T) -> T
where
    T: Add<Output = T> + Rem<Output = T> + Copy,
{
    (value % bound + bound) % bound
}
