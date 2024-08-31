use crate::{cell::AtomicCell, grid::AtomicGrid};

use rand::random;

pub fn randomize_grid<const H: usize, const W: usize>(grid: &AtomicGrid<H, W>) {
    for x in 0..H {
        for y in 0..W {
            if random() {
                let x = x as isize;
                let y = y as isize;
                grid.spawn(x, y);
            }
        }
    }
}