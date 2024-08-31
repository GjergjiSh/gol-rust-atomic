use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;

// Configuration
const H: usize = 100;
const W: usize = 100;
const SINGLE_THREADED_GENERATIONS: usize = 1000;

use gol::*;

/* Caching Benchmarks */

/*  atomic_grid_safe_copy   time:   [75.358 µs 75.830 µs 76.349 µs]
    atomic_grid_unsafe_copy time:   [20.371 µs 20.445 µs 20.525 µs]
    unsafe_cell_copy        time:   [375.47 ns 377.40 ns 379.91 ns]
*/

pub fn atomic_grid_safe_copy() {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let other: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    grid.copy_from(&other);
}

pub fn atomic_grid_unsafe_copy() {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let other: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    unsafe { grid.unsafe_copy_from(&other) };
}

pub fn unsafe_cell_copy() {
    let state = Arc::new(SharedState::<H, W>::new());
    let mut cache = Arc::new(SharedState::<H, W>::new());
    cache.clone_from(&state);
}

/* Generation Benchmarks */

pub fn unsafe_atomic_generation() {
    let grid = AtomicGrid::<H, W>::new();
    let generator = AtomicGenerator::<H, W>::new(Arc::new(&grid));

    for cell in grid.iter() {
        cell.store(0b0001_0001);
    }

    //TODO: Unsafe
    unsafe { generator.u_generate() };
}

pub fn safe_atomic_generation() {
    let grid = AtomicGrid::<H, W>::new();
    let mut generator = AtomicGenerator::<H, W>::new(Arc::new(&grid));

    for cell in grid.iter() {
        cell.store(0b0001_0001);
    }

    generator.generate();
}

pub fn unsafe_simple_cell_generation() {
    let generator = UnsafeCellGenerator::<H, W>::new();
    generator.generate();
}

// Single threaded
pub fn single_threaded() {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let grid = Arc::new(&grid);
    randomize_grid(&grid);

    let generator = AtomicGenerator::<H, W>::new(Arc::clone(&grid));

    for _ in 0..SINGLE_THREADED_GENERATIONS {
        unsafe {
            generator.u_generate();
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("single_threaded", |b| b.iter(|| single_threaded()));
    c.bench_function("atomic_grid_safe_copy", |b| {
        b.iter(|| atomic_grid_safe_copy())
    });
    c.bench_function("atomic_grid_unsafe_copy", |b| {
        b.iter(|| atomic_grid_unsafe_copy())
    });
    c.bench_function("unsafe_cell_copy", |b| {
        b.iter(|| unsafe_cell_copy())
    });
    // c.bench_function("unsafe_atomic_generation", |b| {
    //     b.iter(|| unsafe_atomic_generation())
    // });
    // c.bench_function("safe_atomic_generation", |b| {
    //     b.iter(|| safe_atomic_generation())
    // });
    // c.bench_function("unsafe_simple_cell_generation", |b| {
    //     b.iter(|| unsafe_simple_cell_generation())
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
