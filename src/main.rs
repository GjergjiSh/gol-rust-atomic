mod gol;

use std::time::Duration;

use gol::*;

const H: usize = 100;
const W: usize = 100;
const GENERATIONS: usize = 1000;
const DISPLAY: bool = false;
const DISPLAY_DELAY: u64 = 0;
const BENCHMARKS: usize = 10;

// Single threaded
pub fn single_threaded() -> (Duration, Duration, f32) {
    let grid: Grid<H, W> = Grid::<H, W>::new();
    let grid = Arc::new(&grid);

    randomize_grid(&grid);

    let generator = SingleThreadedGenerator::<H, W>::new(Arc::clone(&grid));
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
    let elapsed = end - start;
    let elapsed_per_generation = elapsed / GENERATIONS as u32;
    println!(
        "Time taken to generate {} generations of size {} {}: {:?}",
        GENERATIONS, H, W, elapsed
    );
    println!(
        "Average time taken to generate a generation: {:?}",
        elapsed_per_generation
    );

    let kb_processed = H * W * GENERATIONS / 1024;
    let kb_per_second = kb_processed as f32 / (end - start).as_secs_f32();
    println!("Processed {} KB at {:.2} KB/s", kb_processed, kb_per_second);

    (elapsed, elapsed_per_generation, kb_per_second)
}

fn main() {
    let mut total_elapsed = 0.0;
    let mut total_elapsed_per_generation = 0.0;
    let mut total_kb_per_second = 0.0;

    for _ in 0..BENCHMARKS {
        let (elapsed, elapsed_per_generation, kb_per_second) = single_threaded();
        total_elapsed += elapsed.as_secs_f64();
        total_elapsed_per_generation += elapsed_per_generation.as_secs_f64();
        total_kb_per_second += kb_per_second;
    }

    let avg_elapsed = total_elapsed / BENCHMARKS as f64;
    let avg_elapsed_per_generation = total_elapsed_per_generation / BENCHMARKS as f64;
    let avg_kb_per_second = total_kb_per_second / BENCHMARKS as f32;

    println!("Finished {} BENCHMARKS", BENCHMARKS);
    println!("Average elapsed time: {:.9} seconds", avg_elapsed);
    println!(
        "Average elapsed time per generation: {:.9} seconds",
        avg_elapsed_per_generation
    );
    println!("Average KB per second: {:.3} KB/s", avg_kb_per_second);
}
