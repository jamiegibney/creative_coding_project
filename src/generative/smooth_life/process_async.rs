use super::*;
use std::{
    io::Write,
    sync::{Arc, Mutex, RwLock},
};

pub struct SmoothLifeGeneratorAsync {
    /// The internal state of the simulation.
    pub state: Arc<SLState>,
    /// The current grid.
    pub grid: Arc<RwLock<Grid>>,
    /// Thread scratch buffers.
    thread_buffers: Vec<Arc<Mutex<Vec<Vec<f64>>>>>,
    /// The internal thread pool.
    pool: ThreadPool,
}

impl SmoothLifeGeneratorAsync {
    pub fn new(size: usize) -> Self {
        let grid = Arc::new(RwLock::new(Grid::new_square(size).with_random()));

        let thread_buffers = 8;
        let thread_buf_height = size / thread_buffers;

        Self {
            state: Arc::new(SLState::default()),
            thread_buffers: {
                let mut v = Vec::with_capacity(thread_buffers);
                (0..thread_buffers).for_each(|_| {
                    v.push(Arc::new(Mutex::new(vec![
                        vec![
                            0.0;
                            thread_buf_height
                        ];
                        size
                    ])));
                });
                v
            },
            grid,

            pool: ThreadPool::build(4)
                .expect("failed to build thread pool for SmoothLifeGenerator"),
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.compute_diff();
        self.merge_thread_buffers(delta_time);
    }

    pub fn set_state(&mut self, state: SLState) {
        self.state = Arc::new(state);
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.state = Arc::new(SLState { dt: speed, ..*self.state });
    }

    pub fn set_outer_radius(&mut self, ra: f64) {
        self.state = Arc::new(SLState {
            radius_outer: ra,
            radius_inner: ra / 3.0,
            ..*self.state
        });
    }

    pub fn reset(&mut self) {
        self.grid.write().unwrap().randomize();

        for buf in self.thread_buffers.iter_mut() {
            let mut guard = buf.lock().unwrap();
            guard.iter_mut().for_each(|v| v.fill(0.0));
        }
    }

    /// This method expects `x` an `y` to be in the range `0.0` to `1.0`.
    ///
    /// This interpolates with [bilinear interpolation](https://en.wikipedia.org/wiki/Bilinear_interpolation).
    pub fn get_value(&self, mut x: f64, mut y: f64) -> f64 {
        let grid = self.grid.read().unwrap();
        // map to range
        x = x.clamp(0.0, 1.0) * (grid.width() - 1) as f64;
        y = y.clamp(0.0, 1.0) * (grid.height() - 1) as f64;

        // interpolation values
        let xt = x - x.floor();
        let yt = y - y.floor();

        // indices
        let x1 = x.floor() as usize;
        let x2 = x1 + 1;
        let y1 = y.floor() as usize;
        let y2 = y1 + 1;

        // x-axis lerp
        let top = lerp(grid[x1][y1], grid[x2][y1], xt);
        let bottom = lerp(grid[x1][y2], grid[x2][y2], xt);

        drop(grid);

        // y-axis lerp
        lerp(top, bottom, yt)
    }

    pub fn get_value_nn(&self, mut x: f64, mut y: f64) -> f64 {
        let grid = self.grid.read().unwrap();
        x = x.clamp(0.0, 1.0) * (grid.width() - 1) as f64;
        y = y.clamp(0.0, 1.0) * (grid.height() - 1) as f64;

        let x = x.round() as usize;
        let y = y.round() as usize;

        grid[x][y]
    }

    #[allow(clippy::many_single_char_names)]
    // pub fn draw_size(&self, w: usize, h: usize) {
    //     let grid = self.grid.read().unwrap();
    //     let mut s = String::with_capacity(w * 2 * h + h + 1);
    //
    //     let len = GRAD.len();
    //
    //     clearscreen::clear().expect("failed to clear screen");
    //
    //     // loop
    //     for y in 0..h {
    //         for x in 0..w {
    //             let x = x as f64 / w as f64;
    //             let y = y as f64 / h as f64;
    //
    //             let idx = self.get_value(x, y).mul_add(len as f64, -1.1).floor()
    //                 as usize;
    //
    //             s.push_str(&format!("{} ", GRAD[idx.clamp(0, len - 1)]));
    //         }
    //
    //         s.push('\n');
    //     }
    //
    //     drop(grid);
    //
    //     print!("{s}");
    //
    //     let _ = std::io::stdout().flush();
    // }
    #[allow(clippy::many_single_char_names)]
    fn compute_diff(&mut self) {
        let grid = self.grid.read().unwrap();
        let w = grid.width();
        let h = grid.height();
        drop(grid);

        let ri = self.state.radius_inner;
        let ra = self.state.radius_outer;

        let buffers = self.thread_buffers.len();
        let rows_per_buf = h / buffers;

        for i in 0..buffers {
            let grid = Arc::clone(&self.grid);
            let buf = Arc::clone(&self.thread_buffers[i]);
            let state = Arc::clone(&self.state);
            let start_row = i * rows_per_buf;

            self.pool.execute(move || {
                let ri = state.radius_inner;
                let ra = state.radius_outer;
                let grid = grid.read().unwrap();

                for cx in 0..w {
                    for cy in start_row..(start_row + rows_per_buf) {
                        let mut buf = buf.lock().unwrap();
                        let buf_y = cy - start_row;

                        let (mut m, mut m_norm, mut n, mut n_norm) =
                            (0.0, 0.0, 0.0, 0.0);
                        let ra_1 = ra - 1.0;
                        let min = (-ra_1) as usize;
                        let max = ra_1 as usize;

                        for dx in min..=max {
                            for dy in min..=max {
                                let x = emod(cx + dx, w);
                                let y = emod(cy + dy, h);

                                // this dx + dx is not part of the original algorithm,
                                // but creates really cool stretchy patterns. happy
                                // accidents!
                                let d = (dx + dx + dy * dy) as f64;

                                if d <= ri * ri {
                                    m += grid[x][y];
                                    m_norm += 1.0;
                                }
                                else if d <= ra * ra {
                                    n += grid[x][y];
                                    n_norm += 1.0;
                                }
                            }
                        }

                        n /= n_norm;
                        m /= m_norm;

                        let q = state.transition(n, m);
                        buf[cx][buf_y] = 2.0f64.mul_add(q, -1.0);
                    }
                }

                drop(grid);
            });
        }

        // this is extremely important - without it, there will be a race condition
        // across threads on locking the diff buffers!
        self.pool.wait_until_done();
    }

    fn merge_thread_buffers(&mut self, delta_time: f64) {
        let dt = self.state.dt * delta_time;
        let mut grid = self.grid.write().unwrap();
        let w = grid.width();
        let h = grid.height();

        let buffers = self.thread_buffers.len();
        let rows_per_buf = h / buffers;

        for i in 0..buffers {
            let start_row = i * rows_per_buf;
            let diff_buf = self.thread_buffers[i].lock().unwrap();

            for x in 0..w {
                for y in start_row..(start_row + rows_per_buf) {
                    let diff_y = y - start_row;
                    grid[x][y] = dt
                        .mul_add(diff_buf[x][diff_y], grid[x][y] + 0.002)
                        .clamp(0.0, 1.0);
                }
            }
        }
    }
}

// TODO does this actually work?
fn wrap<T>(value: T, bound: T) -> T
where
    T: Add<Output = T> + Rem<Output = T> + Copy,
{
    (value % bound + bound) % bound
}

fn emod(value: usize, bound: usize) -> usize {
    (value % bound + bound) % bound
}

pub fn lerp(a: f64, b: f64, mut t: f64) -> f64 {
    t = t.clamp(0.0, 1.0);
    if t <= f64::EPSILON {
        return a;
    }
    else if t >= 1.0 - f64::EPSILON {
        return b;
    }

    t.mul_add(b - a, a)
}
