use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;

// Configuration
const H: usize = 100;
const W: usize = 100;
const SINGLE_THREADED_GENERATIONS: usize = 1000;

use gol::*;

pub fn bench_atomic_grid_copy() {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let other: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    for cell in grid.iter() {
        cell.store(0b0001_0001);
    }

    grid.copy_from(&other);
}

pub fn bench_atomic_grid_unsafe_copy() {
    let grid: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    let other: AtomicGrid<H, W> = AtomicGrid::<H, W>::new();
    for cell in grid.iter() {
        cell.store(0b0001_0001);
    }
    unsafe { grid.unsafe_copy_from(&other) };
}

pub fn bench_atomic_generation() {
    let grid = AtomicGrid::<H, W>::new();
    let generator = AtomicGenerator::<H, W>::new(Arc::new(&grid));
    for cell in grid.iter() {
        cell.store(0b0001_0001);
    }
    unsafe { generator.u_generate() };
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
    // c.bench_function("bench_atomic_grid_copy", |b| {
    //     b.iter(|| bench_atomic_grid_copy())
    // });
    // c.bench_function("bench_atomic_grid_unsafe_copy", |b| {
    //     b.iter(|| bench_atomic_grid_unsafe_copy())
    // });
    c.bench_function("bench_atomic_generation", |b| {
        b.iter(|| bench_atomic_generation())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
