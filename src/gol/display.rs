use std::sync::Arc;

use crate::gol::grid::Grid;
use minifb::{Window, WindowOptions};

const COLOR_ALIVE: u32 = 0xFFFFFF; // White
const COLOR_DEAD: u32 = 0x000000; // Black
const SCALE: usize = 10; // Upscaling factor

// Display window for the Game of Life
pub struct Display<'a, const H: usize, const W: usize> {
    grid: Arc<&'a Grid<H, W>>,
    window: Window,
    delay: u64,
}

// Implement Display
impl<'a, const H: usize, const W: usize> Display<'a, H, W> {
    pub fn new(grid: Arc<&'a Grid<H, W>>, delay: u64) -> Self {
        let window = Window::new(
            "Conway's Game of Life",
            W * SCALE,
            H * SCALE,
            WindowOptions::default(),
        )
        .unwrap();

        Self {
            grid,
            window,
            delay,
        }
    }

    pub fn update(&mut self) {
        let mut buffer: Vec<u32> = vec![0; W * H];

        for y in 0..H {
            for x in 0..W {
                let color = {
                    let cell = self.grid.get(x as isize, y as isize);
                    if cell.alive() {
                        COLOR_ALIVE
                    } else {
                        COLOR_DEAD
                    }
                };
                buffer[y * W + x] = color;
            }
        }
        self.window.update_with_buffer(&buffer, W, H).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(self.delay as u64));
    }
}

#[cfg(test)]
mod tests {
    use crate::gol::*;

    use std::{borrow::BorrowMut, sync::Arc};

    pub const GLIDER_OFFSETS: [(isize, isize); 5] = [(2, 0), (2, 1), (2, 2), (1, 2), (0, 1)];

    #[test]
    fn test_glider_display() {
        const H: usize = 100;
        const W: usize = 100;
        const GENERATIONS: usize = 1000;

        let grid: Grid<H, W> = Grid::<H, W>::new();
        let grid = Arc::new(&grid);
        grid.spawn_shape((0, 0), &GLIDER_OFFSETS);

        let mut generator = Generator::<H, W>::new(Arc::clone(&grid));
        let mut display = Display::<H, W>::new(Arc::clone(&grid), 0);

        for _ in 0..GENERATIONS {
            generator.generate();
            display.update();
        }
    }

    #[test]
    fn test_random_display() {
        const H: usize = 100;
        const W: usize = 100;
        const GENERATIONS: usize = 1000;

        let grid: Grid<H, W> = Grid::<H, W>::new();
        let grid = Arc::new(&grid);

        randomize_grid(&grid);

        let generator = Generator::<H, W>::new(Arc::clone(&grid));
        let mut display = Display::<H, W>::new(Arc::clone(&grid), 0);

        for _ in 0..GENERATIONS {
            generator.generate();
            display.update();
        }
    }
}
