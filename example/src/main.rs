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
const BENCHMARKS: usize = 1;
const MULTI_THREADED: bool = true;
const THREAD_COUNT: usize = 4;

// Multi threaded
pub fn multi_threaded() -> (Duration, Duration, f32) {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let grid = Arc::new(grid);

    randomize_grid(&grid);

    let generator = AtomicGenerator::<H, W>::new(Arc::clone(&grid));
    let generator = Arc::new(generator);

    let mut display = None;

    if DISPLAY {
        display = Some(AtomicDisplay::<H, W>::new(Arc::clone(&grid), DISPLAY_DELAY));
    }

    let barrier = Arc::new(Barrier::new(THREAD_COUNT + 1)); // +1 for the main thread
    let cache_updated = Arc::new((Mutex::new(false), Condvar::new()));
    let threads_done = Arc::new((Mutex::new(0), Condvar::new()));
    let stop_signal = Arc::new(Mutex::new(false));

    let mut handles = Vec::new();

    for i in 0..THREAD_COUNT {
        let generator = Arc::clone(&generator);
        let barrier = Arc::clone(&barrier);
        let cache_updated = Arc::clone(&cache_updated);
        let threads_done = Arc::clone(&threads_done);
        let stop_signal = Arc::clone(&stop_signal);

        let rows_per_thread = H / THREAD_COUNT;
        let cols_per_thread = W / THREAD_COUNT;

        let start_row = (i / THREAD_COUNT) * rows_per_thread;
        let end_row = start_row + rows_per_thread;

        let start_col = (i % THREAD_COUNT) * cols_per_thread;
        let end_col = start_col + cols_per_thread;

        handles.push(thread::spawn(move || {
            loop {
                // Wait for cache to be updated
                let (cache_lock, cache_cvar) = &*cache_updated;
                let mut cache_ready = cache_lock.lock().unwrap();
                while !*cache_ready {
                    cache_ready = cache_cvar.wait(cache_ready).unwrap();
                }
                drop(cache_ready);

                // Check if we should stop
                if *stop_signal.lock().unwrap() {
                    println!("Thread {} stopping", i);
                    break;
                }

                println!(
                    "Thread {} processing rows {}-{} cols {}-{}",
                    i, start_row, end_row, start_col, end_col
                );
                generator.update_grid_range((start_row, start_col), (end_row, end_col));

                // Signal that this thread is done
                let (done_lock, done_cvar) = &*threads_done;
                let mut done_count = done_lock.lock().unwrap();
                *done_count += 1;
                if *done_count == THREAD_COUNT {
                    done_cvar.notify_all();
                }
                drop(done_count);

                // Wait for all threads to finish processing
                barrier.wait();
            }
        }));
    }

    let start = std::time::Instant::now();
    for _ in 0..GENERATIONS {
        // Update the cache for the next generation
        unsafe {
            generator.u_update_cache();
        }

        // Reset the threads_done counter
        let (done_lock, _) = &*threads_done;
        let mut done_count = done_lock.lock().unwrap();
        *done_count = 0;
        drop(done_count);

        // Signal that cache is updated
        let (cache_lock, cache_cvar) = &*cache_updated;
        {
            let mut cache_ready = cache_lock.lock().unwrap();
            *cache_ready = true;
            cache_cvar.notify_all();
        }

        // Wait for all threads to finish processing
        let (done_lock, done_cvar) = &*threads_done;
        let mut done_count = done_lock.lock().unwrap();
        while *done_count < THREAD_COUNT {
            done_count = done_cvar.wait(done_count).unwrap();
        }
        drop(done_count);

        // Update display if necessary
        if let Some(ref mut display) = display {
            display.update();
        }

        // Reset the cache_updated flag for the next generation
        let mut cache_ready = cache_updated.0.lock().unwrap();
        *cache_ready = false;

        // Wait for all threads to reach the barrier
        barrier.wait();
    }
    let end = std::time::Instant::now();

    // Signal threads to stop
    {
        let mut stop = stop_signal.lock().unwrap();
        *stop = true;
    }

    // Wake up threads one last time so they can see the stop signal
    let (cache_lock, cache_cvar) = &*cache_updated;
    {
        let mut cache_ready = cache_lock.lock().unwrap();
        *cache_ready = true;
        cache_cvar.notify_all();
    }

    for thread in handles {
        thread.join().unwrap();
    }

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
        if MULTI_THREADED {
            println!("Running multi-threaded benchmark");
            let (elapsed, elapsed_per_generation, kb_per_second) = multi_threaded();
            total_elapsed += elapsed.as_secs_f64();
            total_elapsed_per_generation += elapsed_per_generation.as_secs_f64();
            total_kb_per_second += kb_per_second;
        } else {
            println!("Running single-threaded benchmark");
            let (elapsed, elapsed_per_generation, kb_per_second) = single_threaded();
            total_elapsed += elapsed.as_secs_f64();
            total_elapsed_per_generation += elapsed_per_generation.as_secs_f64();
            total_kb_per_second += kb_per_second;
        }
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
