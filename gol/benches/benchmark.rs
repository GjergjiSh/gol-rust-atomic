use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;
use std::sync::Arc;

const H: usize = 100;
const W: usize = 100;
const GENERATIONS: usize = 1000;
const DISPLAY: bool = false;
const DISPLAY_DELAY: u64 = 0;

use gol::*;

// Single threaded
pub fn single_threaded() -> (Duration, Duration, f32) {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("single_threaded", |b| b.iter(|| single_threaded()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);