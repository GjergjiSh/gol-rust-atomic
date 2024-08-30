mod gol;

use gol::*;

const H: usize = 100;
const W: usize = 100;
const GENERATIONS: usize = 1000;
const DISPLAY: bool = false;
const DISPLAY_DELAY: u64 = 0;
const BENCHMARKS: usize = 10;

// Single threaded
pub fn single_threaded() {
    let grid: Grid<H, W> = Grid::<H, W>::new();
    let grid = Arc::new(&grid);

    randomize_grid(&grid);

    let generator = Generator::<H, W>::new(Arc::clone(&grid));
    let mut display = None;

    if DISPLAY {
        let grid_ref = Arc::new(generator.grid());
        display = Some(Display::<H, W>::new(grid_ref, DISPLAY_DELAY));
    }

    let start = std::time::Instant::now();
    match display {
        Some(ref mut display) => {
            for _ in 0..GENERATIONS {
                generator.generate();
                display.update();
            }
        }
        None => {
            for _ in 0..GENERATIONS {
                generator.generate();
            }
        }
    }
    let end = std::time::Instant::now();
    println!(
        "Time taken to generate {} generations of size {} {}: {:?}",
        GENERATIONS,
        H,
        W,
        end - start
    );
    println!(
        "Average time taken to generate a generation: {:?}",
        (end - start) / GENERATIONS as u32
    );

    let kb_processed = H * W * GENERATIONS / 1024;
    let kb_per_second = kb_processed as f32 / (end - start).as_secs_f32();
    println!("Processed {} KB at {:.2} KB/s", kb_processed, kb_per_second);
}

fn main() {
    for _ in 0..BENCHMARKS {
        single_threaded();
    }
}
