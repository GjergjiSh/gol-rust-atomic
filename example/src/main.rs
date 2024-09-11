use std::{
    sync::{Barrier, Condvar, Mutex},
    thread,
    time::Duration,
};

use gol::*;

const H: usize = 100;
const W: usize = 100;
const GENERATIONS: usize = 1000;
const DISPLAY: bool = true;
const DISPLAY_DELAY: u64 = 0;
const BENCHMARKS: usize = 10;

// Multi threaded
pub fn multi_threaded(thread_count: usize) -> (Duration, Duration, f32) {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let grid = Arc::new(grid);

    randomize_grid(&grid);

    let generator = AtomicGenerator::<H, W>::new(Arc::clone(&grid));
    let generator = Arc::new(generator);

    let mut display = None;

    if DISPLAY {
        display = Some(AtomicDisplay::<H, W>::new(grid, DISPLAY_DELAY));
    }

    // NOTE: +1 for the main thread
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let signal = Arc::new((Mutex::new(false), Condvar::new()));

    let mut handles = Vec::new();

    let rows_per_thread = H / thread_count;
    let cols_per_thread = W / thread_count;

    for i in 0..thread_count {
        let generator = Arc::clone(&generator);
        let barrier = Arc::clone(&barrier);
        let signal = Arc::clone(&signal);

        let start_row = i * rows_per_thread;
        let end_row = if i == thread_count - 1 {
            H
        } else {
            (i + 1) * rows_per_thread
        };

        let start_col = i * cols_per_thread;
        let end_col = if i == thread_count - 1 {
            W
        } else {
            (i + 1) * cols_per_thread
        };

        handles.push(thread::spawn(move || {
            // Wait until the state is cached
            let (lock, cvar) = &*signal;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
            drop(ready);

            // After the state is cached we are free to generate
            generator.update_grid_range((start_row, start_col), (end_row, end_col));

            barrier.wait();
        }));
    }

    let start = std::time::Instant::now();
    match display {
        Some(ref mut display) => {
            for _ in 0..GENERATIONS {
                unsafe {
                    generator.u_update_cache();
                }
                let (lock, cvar) = &*signal;
                let mut ready = lock.lock().unwrap();
                *ready = true;
                cvar.notify_all();
                barrier.wait();
                display.update();
            }
        }
        None => for _ in 0..GENERATIONS {},
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

    for thread in handles {
        thread.join().unwrap();
    }

    (elapsed, elapsed_per_generation, kb_per_second)
}

// Single threaded
pub fn single_threaded() -> (Duration, Duration, f32) {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let grid = Arc::new(grid);

    randomize_grid(&grid);

    let generator = AtomicGenerator::<H, W>::new(Arc::clone(&grid));
    let mut display = None;

    if DISPLAY {
        display = Some(AtomicDisplay::<H, W>::new(Arc::clone(&grid), DISPLAY_DELAY));
    }

    let start = std::time::Instant::now();
    match display {
        Some(ref mut display) => {
            for _ in 0..GENERATIONS {
                unsafe {
                    generator.u_generate();
                }
                display.update();
            }
        }
        None => {
            for _ in 0..GENERATIONS {
                unsafe {
                    generator.u_generate();
                }
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
        // let (elapsed, elapsed_per_generation, kb_per_second) = single_threaded();
        let (elapsed, elapsed_per_generation, kb_per_second) = multi_threaded(4);
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
