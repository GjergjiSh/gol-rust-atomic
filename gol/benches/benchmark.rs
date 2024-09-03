use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

// Configuration
const H: usize = 100;
const W: usize = 100;
const SINGLE_THREADED_GENERATIONS: usize = 1000;

use gol::*;

/* Creating Benchmarks */

pub fn create_atomic_generator() {
    AtomicGrid::<H, W>::new();
}

pub fn create_ref_cell_generator() {
    UnsafeCellGenerator::<H, W>::new();
}

/* Caching Benchmarks */
/*
atomic_generator_safe_caching time:   [72.981 µs 73.065 µs 73.161 µs]
atomic_generator_unsafe_caching time:   [20.142 µs 20.217 µs 20.304 µs]
ref_cell_generator_caching time:   [571.54 ns 574.05 ns 576.89 ns]
*/

pub fn atomic_generator_safe_caching() {
    let grid = AtomicGrid::<H, W>::new();
    let mut atomic_generator = AtomicGenerator::<H, W>::new(Arc::new(&grid));
    atomic_generator.update_cache();
}

pub fn atomic_generator_unsafe_caching() {
    let grid = AtomicGrid::<H, W>::new();
    let atomic_generator = AtomicGenerator::<H, W>::new(Arc::new(&grid));
    unsafe { atomic_generator.u_update_cache() };
}

pub fn ref_cell_generator_caching() {
    let generator = UnsafeCellGenerator::<H, W>::new();
    let state = &mut *generator.grid().get_mut();
    let cache = &mut *generator.cache().get_mut();
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

/* Simple Copy Benchmarks */
const SIZE: usize = 1024 * 100000;

pub fn simple_copy_method_one() {
    let cells: Vec<u8> = vec![1; SIZE];
    let _cache: Vec<u8> = cells.clone();
}

pub fn simple_copy_method_two() {
    let cells: Vec<u8> = vec![1; SIZE];
    let mut cache: Vec<u8> = Vec::<u8>::with_capacity(SIZE);

    unsafe {
        // Perform the unsafe memory copy
        std::ptr::copy_nonoverlapping(cells.as_ptr(), cache.as_mut_ptr(), cells.len());
    }
}

pub fn simple_copy_method_three() {
    let cells: Vec<u8> = vec![1; SIZE];
    let mut cache: Vec<u8> = Vec::<u8>::with_capacity(SIZE);

    for (cell, cache_cell) in cells.iter().zip(cache.iter_mut()) {
        *cache_cell = *cell;
    }
}

pub fn simple_copy_method_four() {
    let cells: Vec<u8> = vec![1; SIZE];
    let mut cache: Vec<u8> = Vec::<u8>::with_capacity(SIZE);

    for cell in cells.iter() {
        let _ = cache.push(*cell);
    }
}

/* Atomic Copy Benchmarks */

pub fn atomic_copy_method_one() {
    let cells: Vec<AtomicU8> = (0..SIZE).map(|_| AtomicU8::new(1)).collect();
    let mut cache: Vec<AtomicU8> = Vec::<AtomicU8>::with_capacity(SIZE);

    for (cell, cache_cell) in cells.iter().zip(cache.iter_mut()) {
        let _ = cache_cell.compare_exchange(
            cache_cell.load(Ordering::Relaxed),
            cell.load(Ordering::Relaxed),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }
}

pub fn atomic_method_two() {
    let cells: Vec<AtomicU8> = (0..SIZE).map(|_| AtomicU8::new(1)).collect();
    let mut cache: Vec<AtomicU8> = Vec::<AtomicU8>::with_capacity(SIZE);

    unsafe {
        // Perform the unsafe memory copy
        std::ptr::copy_nonoverlapping(cells.as_ptr(), cache.as_mut_ptr(), cells.len());
    }
}

pub fn atomic_method_three() {
    let cells: Vec<AtomicU8> = (0..SIZE).map(|_| AtomicU8::new(1)).collect();
    let mut cache: Vec<AtomicU8> = Vec::<AtomicU8>::with_capacity(SIZE);

    for cell in cells.iter() {
        let _ = cache.push(cell.load(Ordering::Relaxed).into());
    }
}

pub fn atomic_method_four() {
    let cells: Vec<AtomicU8> = (0..SIZE).map(|_| AtomicU8::new(1)).collect();
    let mut cache: Vec<AtomicU8> = Vec::<AtomicU8>::with_capacity(SIZE);

    for cell in cells.iter() {
        cache.push(cell.load(Ordering::SeqCst).into());
    }
}

/* Atomic Single Threaded Generation */

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

/* Main */

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple_copy_method_one", |b| {
        b.iter(|| simple_copy_method_one())
    });
    c.bench_function("simple_copy_method_two", |b| {
        b.iter(|| simple_copy_method_two())
    });
    c.bench_function("simple_copy_method_three", |b| {
        b.iter(|| simple_copy_method_three())
    });
    c.bench_function("simple_copy_method_four", |b| {
        b.iter(|| simple_copy_method_four())
    });
    c.bench_function("atomic_copy_method_one", |b| {
        b.iter(|| atomic_copy_method_one())
    });
    c.bench_function("atomic_method_two", |b| b.iter(|| atomic_method_two()));
    c.bench_function("atomic_method_three", |b| b.iter(|| atomic_method_three()));
    c.bench_function("atomic_method_four", |b| b.iter(|| atomic_method_four()));
    // c.bench_function("single_threaded", |b| b.iter(|| single_threaded()));
    // c.bench_function("create_atomic_generator", |b| {
    //     b.iter(|| create_atomic_generator())
    // });
    // c.bench_function("create_ref_cell_generator", |b| {
    //     b.iter(|| create_ref_cell_generator())
    // });
    // c.bench_function("atomic_generator_safe_caching", |b| {
    //     b.iter(|| atomic_generator_safe_caching())
    // });
    // c.bench_function("atomic_generator_unsafe_caching", |b| {
    //     b.iter(|| atomic_generator_unsafe_caching())
    // });
    // c.bench_function("ref_cell_generator_caching", |b| {
    //     b.iter(|| ref_cell_generator_caching())
    // });
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
