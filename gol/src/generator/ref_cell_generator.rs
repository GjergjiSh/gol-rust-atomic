use std::{cell::UnsafeCell, sync::Arc};

use crate::grid::SimpleGrid;

pub struct SharedState<const H: usize, const W: usize>(UnsafeCell<SimpleGrid<H, W>>);

impl<const H: usize, const W: usize> SharedState<H, W> {
    pub fn new() -> Self {
        Self(UnsafeCell::new(SimpleGrid::new()))
    }

    pub fn state(&self) -> &SimpleGrid<H, W> {
        unsafe { &*self.0.get() }
    }

    pub fn mut_state(&self) -> &mut SimpleGrid<H, W> {
        unsafe { &mut *self.0.get() }
    }

    pub fn cache(&self) -> &SimpleGrid<H, W> {
        unsafe { &*self.0.get() }
    }

    pub fn mut_cache(&self) -> &mut SimpleGrid<H, W> {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl<const H: usize, const W: usize> Sync for SharedState<H, W> {}

pub struct Engine<const H: usize, const W: usize> {
    state: Arc<SharedState<H, W>>,
    cache: Arc<SharedState<H, W>>,
}

impl<const H: usize, const W: usize> Engine<H, W> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(SharedState::new()),
            cache: Arc::new(SharedState::new()),
        }
    }

    pub fn randomize(&mut self) {
        for x in 0..H {
            for y in 0..W {
                if rand::random() {
                    self.state.mut_state().spawn(x as isize, y as isize);
                }
            }
        }
    }

    pub fn generate(&self) {
        let state = &mut *self.state.mut_state();
        let cache = &mut *self.cache.mut_cache();

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
        let state = &mut *self.state.mut_state();
        let cache = &mut *self.cache.mut_state();

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
        let state = &mut *self.state.mut_state();
        let cache = &mut *self.cache.mut_cache();
        (state, cache)
    }

    pub fn state_cache_pair(&self) -> (&SimpleGrid<H, W>, &SimpleGrid<H, W>) {
        let state = &*self.state.state();
        let cache = &*self.cache.cache();
        (state, cache)
    }

    pub fn mut_cells(&mut self) -> &mut SimpleGrid<H, W> {
        &mut *self.state.mut_state()
    }

    pub fn cells(&self) -> &SimpleGrid<H, W> {
        &*self.state.state()
    }

    pub fn mut_cells_ref(&self) -> Arc<SharedState<H, W>> {
        Arc::clone(&self.state)
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
        let engine = Engine::<H, W>::new();
        let (average_time, total_time) = measure_execution_time(
            || engine.cache.state().clone_from(&engine.state.state()),
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

        let engine = Engine::<H, W>::new();

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

        let mut engine = Engine::<H, W>::new();

        engine.randomize();
        let state = engine.cells().clone();

        for _ in 0..10 {
            engine.generate();
        }

        let state2 = engine.cells().clone();
        assert_ne!(state, state2);
    }
}
