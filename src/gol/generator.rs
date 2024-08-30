use crate::gol::{cell::Cell, grid::Grid};

pub struct Generator<const H: usize, const W: usize> {
    grid: Grid<H, W>,
    cache: Grid<H, W>,
}

impl<const H: usize, const W: usize> Generator<H, W> {
    pub fn new(grid: Grid<H,W>) -> Self {
        Self {
            grid: grid,
            cache: Grid::new(),
        }
    }

    pub fn generate(&self) {
        self.cache.copy_from(&self.grid);

        for x in 0..H {
            for y in 0..W {
                let x = x as isize;
                let y = y as isize;

                let cell = self.cache.get(x, y);

                if *cell == 0b00000000 {
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

    pub fn grid(&self) -> &Grid<H, W> {
        &self.grid
    }
}
