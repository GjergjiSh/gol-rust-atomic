use std::sync::atomic::AtomicU8;
use std::alloc::Layout;

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

pub fn u8_vector_creation_method_one() {
    let _cells: Vec<u8> = vec![1; SIZE];
}

pub fn u8_vector_creation_method_two() {
    let mut cells: Vec<u8> = Vec::<u8>::with_capacity(SIZE);
    for _ in 0..SIZE {
        cells.push(1);
    }
}

pub fn make_atomic_box(size: usize) -> Box<[AtomicU8]> {
    let layout = Layout::array::<AtomicU8>(size).unwrap();
    let cells = unsafe { std::alloc::alloc(layout) as *mut AtomicU8 };

    unsafe {
        std::ptr::write_bytes(cells, 1, size);
    }

    let slice = unsafe { std::slice::from_raw_parts_mut(cells, size) };
    let cells = unsafe { Box::from_raw(slice as *mut [AtomicU8]) };

    cells
}

use std::sync::atomic::Ordering;

// This gets created with vec![AtomicWrapper::new(); SIZE] in 146.33 ps
struct AtomicWrapper(AtomicU8);

impl Clone for AtomicWrapper {
    fn clone(&self) -> Self {
        AtomicWrapper(AtomicU8::new(self.0.load(Ordering::Acquire)))
    }
}

impl AtomicWrapper {
    fn new() -> Self {
        AtomicWrapper(AtomicU8::new(0))
    }
}

pub fn atomic_u8_vector_creation_method_one() {
    vec![AtomicWrapper::new(); SIZE];
}

pub fn atomic_u8_vector_creation_method_two() {
    let mut cells: Vec<AtomicU8> = Vec::<AtomicU8>::with_capacity(SIZE);

    for _ in 0..SIZE {
        cells.push(AtomicU8::new(0));
    }
}