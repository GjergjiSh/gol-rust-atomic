use crate::{
    cell::AtomicCell,
    common::{Cell, Generator},
    grid::AtomicGrid,
};

use std::sync::Arc;

pub struct SingleThreadedGenerator<'a, const H: usize, const W: usize> {
    grid: Arc<&'a AtomicGrid<H, W>>,
    cache: AtomicGrid<H, W>,
}

impl<'a, const H: usize, const W: usize> SingleThreadedGenerator<'a, H, W> {
    pub fn new(grid: Arc<&'a AtomicGrid<H, W>>) -> Self {
        Self {
            grid: grid,
            cache: AtomicGrid::new(),
        }
    }

    fn generate(&self) {
        unsafe {
            self.cache.unsafe_copy_from(&self.grid);
        }

        for x in 0..H {
            for y in 0..W {
                let x = x as isize;
                let y = y as isize;

                let cell = self.cache.get(x, y);

                if cell.fetch() == 0b0000_0000 {
                    continue;
                }

                let neighbor_count = cell.neighbors();

                if cell.alive() {
                    if neighbor_count < 2 || neighbor_count > 3 {
                        self.grid.kill(x, y);
                    }
                } else {
                    if neighbor_count == 3 {
                        self.grid.spawn(x, y);
                    }
                }
            }
        }
    }

    fn grid(&self) -> &AtomicGrid<H, W> {
        &self.grid
    }
}

impl<'a, const H: usize, const W: usize> Generator<H, W> for SingleThreadedGenerator<'a, H, W> {
    fn generate(&self) {
        self.generate();
    }

    fn grid(&self) -> &AtomicGrid<H, W> {
        self.grid()
    }
}