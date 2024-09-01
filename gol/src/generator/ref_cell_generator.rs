use std::{cell::UnsafeCell, sync::Arc};

use crate::grid::SimpleGrid;

pub struct SharedState<const H: usize, const W: usize>(UnsafeCell<SimpleGrid<H, W>>);

impl<const H: usize, const W: usize> SharedState<H, W> {
    pub fn new() -> Self {
        Self(UnsafeCell::new(SimpleGrid::new()))
    }

    pub fn get(&self) -> &SimpleGrid<H, W> {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut SimpleGrid<H, W> {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl<const H: usize, const W: usize> Sync for SharedState<H, W> {}

pub struct UnsafeCellGenerator<const H: usize, const W: usize> {
    grid: Arc<SharedState<H, W>>,
    cache: Arc<SharedState<H, W>>,
}

impl<const H: usize, const W: usize> UnsafeCellGenerator<H, W> {
    pub fn new() -> Self {
        Self {
            grid: Arc::new(SharedState::new()),
            cache: Arc::new(SharedState::new()),
        }
    }

    pub fn randomize(&mut self) {
        for x in 0..H {
            for y in 0..W {
                if rand::random() {
                    self.grid.get_mut().spawn(x as isize, y as isize);
                }
            }
        }
    }

    pub fn generate(&self) {
        let state = &mut *self.grid.get_mut();
        let cache = &mut *self.cache.get_mut();

        cache.clone_from(&state);

        for x in 0..H {
            for y in 0..W {
                let x = x as isize;
                let y = y as isize;

                let cell = cache.get(x, y);

                if *cell == 0b00000000 {
                    continue;
                }

                let neighbour_count = cell.neighbors();

                if cell.alive() {
                    if neighbour_count < 2 || neighbour_count > 3 {
                        state.kill(x, y);
                    }
                } else {
                    if neighbour_count == 3 {
                        state.spawn(x, y);
                    }
                }
            }
        }
    }

    pub fn generate_slice(&self, x0: usize, xn: usize, y0: usize, yn: usize) {
        let state = &mut *self.grid.get_mut();
        let cache = &mut *self.cache.get_mut();

        for x in x0..xn {
            for y in y0..yn {
                let x = x as isize;
                let y = y as isize;

                let cell = cache.get(x, y);

                if *cell == 0b00000000 {
                    continue;
                }

                let neighbor_count = cell.neighbors();

                if cell.alive() {
                    if neighbor_count < 2 || neighbor_count > 3 {
                        state.kill(x, y);
                    }
                } else {
                    if neighbor_count == 3 {
                        state.spawn(x, y);
                    }
                }
            }
        }
    }

    pub fn mut_state_cache_pair(&self) -> (&mut SimpleGrid<H, W>, &mut SimpleGrid<H, W>) {
        let state = &mut *self.grid.get_mut();
        let cache = &mut *self.cache.get_mut();
        (state, cache)
    }

    pub fn state_cache_pair(&self) -> (&SimpleGrid<H, W>, &SimpleGrid<H, W>) {
        let state = &*self.grid.get();
        let cache = &*self.cache.get();
        (state, cache)
    }

    pub fn mut_cells(&mut self) -> &mut SimpleGrid<H, W> {
        &mut *self.grid.get_mut()
    }

    pub fn grid(&self) -> &SharedState<H, W> {
        &self.grid
    }

    pub fn cache(&self) -> &SharedState<H, W> {
        &self.cache
    }

    pub fn mut_cells_ref(&self) -> Arc<SharedState<H, W>> {
        Arc::clone(&self.grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn measure_execution_time<F>(mut f: F, count: usize) -> (Duration, Duration)
    where
        F: FnMut(),
    {
        let mut total_time = Duration::new(0, 0);

        for _ in 0..count {
            let start = Instant::now();
            f();
            let duration = start.elapsed();
            total_time += duration;
        }

        let average_time = total_time / count as u32;
        (average_time, total_time)
    }

    #[test]
    fn test_clone_time() {
        const H: usize = 100;
        const W: usize = 100;
        const COUNT: usize = 1000;
        let size = H * W;
        let engine = UnsafeCellGenerator::<H, W>::new();
        let (average_time, total_time) = measure_execution_time(
            || engine.cache.get().clone_from(&engine.grid.get()),
            COUNT,
        );

        println!(
            "Average time taken to clone {} bytes once: {:?}",
            size, average_time
        );
        println!(
            "Total time taken to clone {} bytes {} times: {:?}",
            size, COUNT, total_time
        );
    }

    #[test]
    fn test_generate_time() {
        const H: usize = 1000;
        const W: usize = 1000;
        const COUNT: usize = 100;

        let engine = UnsafeCellGenerator::<H, W>::new();

        let start = std::time::Instant::now();
        for _ in 0..COUNT {
            engine.generate();
        }
        let end = std::time::Instant::now();

        println!(
            "Time taken to generate {} generations: {:?}",
            COUNT,
            end - start
        );
        println!(
            "Average time taken to generate a generation: {:?}",
            (end - start) / COUNT as u32
        );
    }

    #[test]
    pub fn test_state_change() {
        //TODO: Implement
        const H: usize = 5;
        const W: usize = 5;

        let mut engine = UnsafeCellGenerator::<H, W>::new();

        engine.randomize();
        let state = engine.grid().get().clone();

        for _ in 0..10 {
            engine.generate();
        }

        let state2 = engine.grid().get().clone();
        assert_ne!(state, state2);
    }

    #[test]
    pub fn test_cache_update() {
        const H: usize = 5;
        const W: usize = 5;

        let generator = UnsafeCellGenerator::<H, W>::new();
        let state = &mut *generator.grid().get_mut();
        let cache = &mut *generator.cache().get_mut();

        for x in 0..H {
            for y in 0..W {
                state.spawn(x as isize, y as isize);
            }
        }

        assert_ne!(generator.grid.get(), generator.cache.get());

        cache.clone_from(&state);

        assert_eq!(generator.grid.get(), generator.cache.get());
    }

}
