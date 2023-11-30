use super::*;

pub struct SmoothLifeGenerator {
    /// The internal state of the simulation.
    state: SLState,
    /// The current state of the grid.
    pub grid: Grid,
    /// Scratch buffer for computing the next grid.
    diff: Grid,
    /// The internal thread pool.
    // TODO async processing
    pool: ThreadPool,
}

impl SmoothLifeGenerator {
    pub fn new(size: usize) -> Self {
        let grid = Grid::new_square(size).with_random();
        Self {
            state: SLState::default(),
            diff: grid.clone(),
            grid,

            pool: ThreadPool::build(4)
                .expect("failed to build thread pool for SmoothLifeGenerator"),
        }
    }

    pub fn update(&mut self) {
        self.compute();
        self.apply_diff();
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    #[allow(clippy::many_single_char_names)]
    fn compute(&mut self) {
        let w = self.grid.width();
        let h = self.grid.height();
        let ri = self.state.radius_inner;
        let ra = self.state.radius_outer;

        // TODO refactor into more idiomatic rust
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
