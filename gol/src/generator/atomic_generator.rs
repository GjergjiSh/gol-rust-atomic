use crate::grid::caching::{CachingStrategy, UnsafeCachingStrategy};
use crate::grid::AtomicGrid;

use std::sync::Arc;

use super::{SafeGenerator, UnsafeGenerator};

// Uses the AtomicGrid to generate the next generation
pub struct AtomicGenerator<'a, const H: usize, const W: usize> {
    grid: Arc<&'a AtomicGrid<H, W>>,
    cache: AtomicGrid<H, W>,
}

// Implement AtomicGenerator
impl<'a, const H: usize, const W: usize> AtomicGenerator<'a, H, W> {
    pub fn new(grid: Arc<&'a AtomicGrid<H, W>>) -> Self {
        Self {
            grid: grid,
            cache: AtomicGrid::new(),
        }
    }

    // pub fn from(grid: &AtomicGrid<H, W>) -> Self {
    //     Self {
    //         grid: Arc::new(grid),
    //         cache: AtomicGrid::new(),
    //     }
    // }

    #[inline]
    pub fn grid(&self) -> &AtomicGrid<H, W> {
        &self.grid
    }

    #[inline]
    pub fn cache(&self) -> &AtomicGrid<H, W> {
        &self.cache
    }

    #[inline]
    fn _update_grid(&self) {
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

    #[inline]
    fn _update_cache(&mut self) {
        self.cache.copy_from(&self.grid);
    }

    #[inline]
    unsafe fn _unsafe_update_cache(&self) {
        self.cache.unsafe_copy_from(&self.grid);
    }
}

// Implement CachingStrategy for AtomicGenerator
impl<'a, const H: usize, const W: usize> CachingStrategy<H, W> for AtomicGenerator<'a, H, W> {
    #[inline]
    fn update_cache(&mut self) {
        self._update_cache();
    }
}

// Implement UnsafeCachingStrategy for AtomicGenerator
impl<'a, const H: usize, const W: usize> UnsafeCachingStrategy<H, W> for AtomicGenerator<'a, H, W> {
    #[inline]
    unsafe fn u_update_cache(&self) {
        self._unsafe_update_cache();
    }
}

// Implement Safe Generation for AtomicGenerator
impl<'a, const H: usize, const W: usize> SafeGenerator<H, W> for AtomicGenerator<'a, H, W> {
    #[inline]
    fn generate(&mut self) {
        self.update_cache();
        self._update_grid();
    }
}

// Implement Unsafe Generation for AtomicGenerator
impl<'a, const H: usize, const W: usize> UnsafeGenerator<H, W> for AtomicGenerator<'a, H, W> {
    #[inline]
    unsafe fn u_generate(&self) {
        self._unsafe_update_cache();
        self._update_grid();
    }
}
