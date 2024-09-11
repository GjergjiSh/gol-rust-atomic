use crate::cell::AtomicCell;

// 2D interface to a vector of cells
// Changes to the contained cells are atomic and a mutable reference
// to the grid is not required to change its state
pub struct AtomicGrid<const H: usize, const W: usize> {
    cells: Vec<AtomicCell>,
}

// Implement Grid
impl<const H: usize, const W: usize> AtomicGrid<H, W> {
    // Create a new grid with dead cells and 0 neighbors
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(H * W);

        for _ in 0..(H * W) {
            cells.push(AtomicCell::default());
        }

        Self { cells }

        // This is 26 to 40% slower than the previous
        // let cells = vec![AtomicCell::new(); H * W];
        // Self { cells }
    }

    #[inline]
    pub fn clone(&self) -> Vec<u8> {
        self.cells.iter().map(|cell| cell.fetch()).collect()
    }

    #[inline]
    // Index the grid with 2D coordinates
    pub fn get(&self, x: isize, y: isize) -> &AtomicCell {
        let w = W as isize;
        let h = H as isize;

        let wrapped_x = ((x % w + w) % w) as usize;
        let wrapped_y = ((y % h + h) % h) as usize;

        &self.cells[wrapped_y * W + wrapped_x]
    }

    #[inline]
    // Spawn a cell at the given 2D coordinates
    // and increment the neighbors of its 8 surrounding cells
    pub fn spawn(&self, x: isize, y: isize) {
        let cell = self.get(x, y);
        let neighbors = self.neighbor_coordinates(x, y);
        cell.spawn();

        for (x, y) in neighbors.iter() {
            let neighbor = self.get(*x, *y);
            neighbor.add_neighbor();
        }
    }

    #[inline]
    // Kill a cell at the given 2D coordinates
    // and decrement the neighbors of its 8 surrounding cells
    pub fn kill(&self, x: isize, y: isize) {
        let cell = self.get(x, y);
        let neighbors = self.neighbor_coordinates(x, y);
        cell.kill();

        for (x, y) in neighbors.iter() {
            let neighbor = self.get(*x, *y);
            neighbor.remove_neighbor();
        }
    }

    #[inline]
    // Spawn a shape at the given 2D coordinates
    // the offsets are relative to the start coordinates
    pub fn spawn_shape(&self, start: (isize, isize), offsets: &[(isize, isize)]) {
        for (dx, dy) in offsets {
            let (x, y) = (start.0 + dx, start.1 + dy);
            self.spawn(x, y)
        }
    }

    #[inline]
    //TODO: Explore optimizations for this
    // Copy the state of the grid to another grid
    // TODO: Check for differing dimensions that add up the the same size
    pub fn copy_from(&self, other: &Self) {
        for i in 0..self.cells.len() {
            let cell = &self.cells[i];
            let other_cell = &other.cells[i];

            cell.compare_and_swap(other_cell);
        }
    }

    #[inline]
    // Utility function to get the wrapped 2D coordinates
    pub fn neighbor_coordinates(&self, x: isize, y: isize) -> [(isize, isize); 8] {
        [
            (x.wrapping_sub(1), y.wrapping_sub(1)), // top_left
            (x, y.wrapping_sub(1)),                 // top
            (x.wrapping_add(1), y.wrapping_sub(1)), // top_right
            (x.wrapping_sub(1), y),                 // left
            (x.wrapping_add(1), y),                 // right
            (x.wrapping_sub(1), y.wrapping_add(1)), // bottom_left
            (x, y.wrapping_add(1)),                 // bottom
            (x.wrapping_add(1), y.wrapping_add(1)), // bottom_right
        ]
    }

    #[inline]
    // Copy the state of the other grid to the grid
    pub unsafe fn unsafe_copy_from(&self, other: &Self) {
        // Check if the grids have the same size
        assert_eq!(
            self.cells.len(),
            other.cells.len(),
            "Grids must have the same size"
        );

        // Perform the unsafe memory copy
        std::ptr::copy_nonoverlapping(
            other.cells.as_ptr(),
            self.cells.as_ptr() as *mut AtomicCell,
            self.cells.len(),
        );
    }

    // #[inline]
    // fn clone_from(&mut self, source: &Self) {
    //     *self = source.clone()
    // }

    #[inline]
    // Return the size of the grid
    pub fn size(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    // Iterate over the cells of the grid
    pub fn iter(&self) -> std::slice::Iter<AtomicCell> {
        self.cells.iter()
    }
}

// impl<const H: usize, const W: usize> ::core::clone::Clone for AtomicGrid<H, W> {
//     #[inline]
//     fn clone(&self) -> Self {
//         Self { cells: ::core::clone::Clone::clone(&self.cells) }
//     }
// }

// Implement Display for Grid
impl<const H: usize, const W: usize> std::fmt::Display for AtomicGrid<H, W> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print the top border of the grid with column numbers
        print!("  +");
        for x in 0..W {
            print!("--{}-+", x); // Col index
        }
        println!();

        // Print the field with side borders and row indices
        for y in 0..H {
            print!("{:2}|", y); // Row index
            for x in 0..W {
                let index = y * W + x;
                let cell = &self.cells[index];
                let neighbors = cell.neighbors();
                let symbol = if cell.alive() { '*' } else { ' ' };
                let symbol = format!("{}{}", symbol, neighbors);
                print!(" {} |", symbol);
            }
            println!(); // End of the row with a side border

            // Print the horizontal border between rows without column numbers
            print!("  +");
            for _ in 0..H {
                print!("----+");
            }
            println!();
        }

        println!();
        Result::Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use utils::*;

    use std::{sync::Arc, thread};

    mod utils {
        use super::*;

        pub const BLOCK_SHAPE_OFFSETS: [(isize, isize); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];

        // Set the cell at the given index to dead and 0 neighbors
        pub fn set_0b0000_0000<const H: usize, const W: usize>(
            grid: &mut AtomicGrid<H, W>,
            idx: usize,
        ) {
            let cell = &mut grid.cells[idx];

            while cell.neighbors() > 0 {
                cell.remove_neighbor();
            }

            cell.kill();
        }

        // Set the cell at the given index to alive and 8 neighbors
        pub fn set_0b0001_0001<const H: usize, const W: usize>(
            grid: &mut AtomicGrid<H, W>,
            idx: usize,
        ) {
            let cell = &mut grid.cells[idx];

            while cell.neighbors() < 8 {
                cell.add_neighbor();
            }

            cell.spawn();
        }

        // Check if the 2d index is correctly translated to a 1d index
        pub fn test_2d_index_translation<const H: usize, const W: usize>(
            idx: usize,
            x: isize,
            y: isize,
        ) {
            let mut grid = AtomicGrid::<H, W>::new();
            set_0b0001_0001(&mut grid, idx);

            let actual = grid.get(x, y);
            assert!(actual.alive());
            assert!(actual.neighbors() == 8);

            let expected = &grid.cells[idx];
            assert_eq!(actual.fetch(), expected.fetch());
        }
    }

    #[test]
    fn test_create_grid() {
        const H: usize = 1000;
        const W: usize = 1000;
        let grid = AtomicGrid::<H, W>::new();
        assert_eq!(grid.cells.len(), H * W);
    }

    #[test]
    fn test_state_manipulation() {
        let mut grid = AtomicGrid::<3, 3>::new();

        // Initial state of all cells: Dead and 0 neighbors (0b0000_0000)
        for cell in grid.cells.iter() {
            assert!(!cell.alive());
            assert!(cell.neighbors() == 0);
            assert!(cell.fetch() == 0b0000_0000);
        }

        // Spawn everything. Each cell is alive and has 8 neighbors (0b0001_0001)
        for i in 0..grid.cells.len() {
            set_0b0001_0001(&mut grid, i);
            let cell = &grid.cells[i];
            assert!(cell.alive());
            assert!(cell.neighbors() == 8);
            assert!(cell.fetch() == 0b0001_0001);
        }

        // KIll everything. Each cell is dead and has 0 neighbors (0b0000_0000)
        for i in 0..grid.cells.len() {
            set_0b0000_0000(&mut grid, i);
            let cell = &grid.cells[i];
            assert!(!cell.alive());
            assert!(cell.neighbors() == 0);
            assert!(cell.fetch() == 0b0000_0000);
        }
    }

    #[test]
    fn test_get_cell() {
        let mut grid = AtomicGrid::<1, 1>::new();

        // Change the cell at index 0 to alive and 8 neighbors
        // to differentiate it from the other cells
        set_0b0001_0001(&mut grid, 0);

        // Simple case: Get the cell at (0, 0)
        let cell = grid.get(0, 0);
        let expected = &grid.cells[0];

        assert!(cell.alive());
        assert!(cell.neighbors() == 8);
        assert_eq!(cell.fetch(), expected.fetch());
        assert_eq!(cell.fetch(), 0b0001_0001);
    }

    #[test]
    fn test_get_cell_w_wrapping() {
        const H: usize = 4;
        const W: usize = 4;

        /* Wrapping on the top left corner
            3  1
            2 [0][0][0][2]
              [0][0][0][0]
              [0][0][0][0]
              [1][0][0][3]
        */

        test_2d_index_translation::<H, W>(12, 0, -1); /* 1 */
        test_2d_index_translation::<H, W>(3, -1, 0); /* 2 */
        test_2d_index_translation::<H, W>(15, -1, -1); /* 3 */

        /* Wrapping on the top right corner
                        1  3
              [2][0][0][0] 2
              [0][0][0][0]
              [0][0][0][0]
              [3][0][0][1]
        */

        test_2d_index_translation::<H, W>(15, 3, -1); /* 1 */
        test_2d_index_translation::<H, W>(0, 4, 0); /* 2 */
        test_2d_index_translation::<H, W>(12, 4, -1); /* 3 */

        /* Wrapping on the bottom left corner
        //       [1][0][0][3]
        //       [0][0][0][0]
        //       [0][0][0][0]
        //     2 [0][0][0][2]
        //     3  1
        // */

        test_2d_index_translation::<H, W>(0, 0, 4); /* 1 */
        test_2d_index_translation::<H, W>(15, -1, 3); /* 2 */
        test_2d_index_translation::<H, W>(3, -1, 4); /* 3 */

        /* Wrapping on the bottom right corner
              [1][0][0][3]
              [0][0][0][0]
              [0][0][0][0]
              [2][0][0][0] 2
                        3  1
        */

        test_2d_index_translation::<H, W>(0, 4, 4); /* 1 */
        test_2d_index_translation::<H, W>(12, 4, 3); /* 2 */
        test_2d_index_translation::<H, W>(3, 3, 4); /* 3 */

        // Wrapping in the middle of the grid is implicitly tested
        // by the other tests
    }

    #[test]
    fn test_spawn_block_shape() {
        let grid = AtomicGrid::<4, 4>::new();

        /* Spawn a block shape at the top left corner
           [1][1][0][0]
           [1][1][0][0]
           [0][0][0][0]
           [0][0][0][0]
        */
        grid.spawn_shape((0, 0), &BLOCK_SHAPE_OFFSETS);

        for coordinate in &BLOCK_SHAPE_OFFSETS {
            let cell = grid.get(coordinate.0, coordinate.1);
            assert!(cell.alive());
            assert!(cell.neighbors() == 3);
        }

        /* Spawn a block shape at the top right corner
           [1][1][1][1]
           [1][1][1][1]
           [0][0][0][0]
           [0][0][0][0]
        */
        grid.spawn_shape((2, 0), &BLOCK_SHAPE_OFFSETS);

        for coordinate in &BLOCK_SHAPE_OFFSETS {
            let cell = grid.get(coordinate.0 + 2, coordinate.1);
            assert!(cell.alive());
            assert_eq!(cell.neighbors(), 5);
        }

        /* Spawn a block shape at the bottom right corner
           [1][1][1][1]
           [1][1][1][1]
           [1][1][0][0]
           [1][1][0][0]
        */
        grid.spawn_shape((2, 2), &BLOCK_SHAPE_OFFSETS);

        //TODO: Test the neighbors
        for coordinate in &BLOCK_SHAPE_OFFSETS {
            let cell = grid.get(coordinate.0 + 2, coordinate.1 + 2);
            assert!(cell.alive());
            // assert_eq!(cell.neighbors(), 5);
        }

        /* Spawn a block shape at the bottom left corner
           [1][1][1][1]
           [1][1][1][1]
           [1][1][1][1]
           [1][1][1][1]
        */
        grid.spawn_shape((0, 2), &BLOCK_SHAPE_OFFSETS);

        //TODO: Test the neighbors
        for coordinate in &BLOCK_SHAPE_OFFSETS {
            let cell = grid.get(coordinate.0, coordinate.1 + 2);
            assert!(cell.alive());
            // assert_eq!(cell.neighbors(), 5);
        }

        for cell in grid.cells.iter() {
            assert!(cell.alive());
            assert!(cell.neighbors() == 8);
        }

        println!("{}", grid);
    }

    #[test]
    fn test_copy_from() {
        let grid = AtomicGrid::<4, 4>::new();
        let mut other = AtomicGrid::<4, 4>::new();

        // Set the state of the other grid to alive and 8 neighbors
        for i in 0..other.cells.len() {
            set_0b0001_0001(&mut other, i);
        }

        let start = std::time::Instant::now();
        // Copy the state of the other grid to the grid
        grid.copy_from(&other);
        let end = std::time::Instant::now();
        println!(
            "Safe: Time taken to copy the state of the other grid to the grid: {:?}",
            end - start
        );

        // Check if the state of the grid is the same as the other grid
        for i in 0..grid.cells.len() {
            let cell = &grid.cells[i];
            assert!(cell.alive());
            assert_eq!(cell.neighbors(), 8);
            assert_eq!(cell.fetch(), 0b0001_0001);
        }

        // Check if the state of the other grid is the same as the grid
        for i in 0..other.cells.len() {
            let cell = &other.cells[i];
            assert!(cell.alive());
            assert_eq!(cell.neighbors(), 8);
            assert_eq!(cell.fetch(), 0b0001_0001);
        }
    }

    #[test]
    fn test_raw_unsafe_copy() {
        use std::cell::UnsafeCell;

        let grid = AtomicGrid::<4, 4>::new();
        let other = AtomicGrid::<4, 4>::new();

        let grid = UnsafeCell::new(grid);
        let other = UnsafeCell::new(other);

        let other_cells = unsafe { &mut (*other.get()).cells };
        let count = other_cells.len();

        for cell in other_cells {
            cell.store(0b0001_0001);
        }

        let start = std::time::Instant::now();
        unsafe {
            let grid_cells = &mut (*grid.get()).cells;
            let other_cells = &*(*other.get()).cells;

            assert_eq!(
                grid_cells.len(),
                other_cells.len(),
                "Vectors must have the same length"
            );

            // SAFETY: The vectors have the same length and type
            std::ptr::copy_nonoverlapping(other_cells.as_ptr(), grid_cells.as_mut_ptr(), count);
        }
        let end = std::time::Instant::now();
        println!(
            "Unsafe: Time taken to copy the state of the other grid to the grid: {:?}",
            end - start
        );
    }

    #[test]
    fn test_unsafe_copy() {
        let grid = AtomicGrid::<4, 4>::new();
        let mut other = AtomicGrid::<4, 4>::new();

        // Set the state of the other grid to alive and 8 neighbors
        for i in 0..other.cells.len() {
            set_0b0001_0001(&mut other, i);
        }

        let start = std::time::Instant::now();
        // Copy the state of the other grid to the grid
        unsafe {
            grid.unsafe_copy_from(&other);
        }
        let end = std::time::Instant::now();
        println!(
            "Unsafe: Time taken to copy the state of the other grid to the grid: {:?}",
            end - start
        );

        // Check if the state of the grid is the same as the other grid
        for i in 0..grid.cells.len() {
            let cell = &grid.cells[i];
            assert!(cell.alive());
            assert_eq!(cell.neighbors(), 8);
            assert_eq!(cell.fetch(), 0b0001_0001);
        }
    }

    #[test]
    fn test_threading() {
        let grid = AtomicGrid::<4, 4>::new();
        let grid = Arc::new(grid);

        let grid_clone = Arc::clone(&grid);
        let t1 = thread::spawn(move || {
            grid_clone.spawn_shape((0, 0), &BLOCK_SHAPE_OFFSETS);
        });

        let grid_clone = Arc::clone(&grid);
        let t2 = thread::spawn(move || {
            grid_clone.spawn_shape((2, 2), &BLOCK_SHAPE_OFFSETS);
        });

        let grid_clone = Arc::clone(&grid);
        let t3 = thread::spawn(move || {
            grid_clone.spawn_shape((2, 0), &BLOCK_SHAPE_OFFSETS);
        });

        let grid_clone = Arc::clone(&grid);
        let t4 = thread::spawn(move || {
            grid_clone.spawn_shape((0, 2), &BLOCK_SHAPE_OFFSETS);
        });

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        t4.join().unwrap();

        for cell in grid.cells.iter() {
            assert!(cell.alive());
            assert!(cell.neighbors() == 8);
        }
    }
}

// TODO: Remove me
#[cfg(test)]
mod t {
    #![allow(warnings)]

    use std::sync::Mutex;
    use std::thread;
    use std::{cell::UnsafeCell, fmt::Debug, sync::Arc};

    struct Wrapper<T>(UnsafeCell<Vec<T>>);
    impl<T> Wrapper<T> {
        fn get(&self) -> *mut Vec<T> {
            self.0.get()
        }

        fn new(vec: Vec<T>) -> Self {
            Self(UnsafeCell::new(vec))
        }

        fn split_at(&self, idx: usize) -> (&[T], &[T]) {
            let x = self.0.get();
            let y = unsafe { &*x };
            y.split_at(idx)
        }
    }
    unsafe impl<T> Sync for Wrapper<T> {}

    fn foo<T: Debug>(slice: &[T], vec: Arc<Wrapper<T>>) -> Arc<Wrapper<T>> {
        let x = vec.0.get();
        let y = unsafe { &*x };
        for i in y.iter() {
            println!("{:?}", i);
        }

        vec
    }

    #[test]
    pub fn move_ownership_of_cache_share_state() {
        // Must be shared between threads
        let vec = vec![1, 2, 3, 4];
        let vec = Wrapper::new(vec);

        // Split the vector into slices
        // TODO: Currently limited at 2
        let (slice1, slice2) = vec.split_at(2);

        // Clone the slices to move them into threads
        let slice1 = slice1.to_vec();
        let slice2 = slice2.to_vec();

        // Reference to the original vector with interior mutability
        let vec = Arc::new(vec);

        // Spawn threads
        let v1 = Arc::clone(&vec);
        let t1 = thread::spawn(move || foo(&slice1, v1));
        let v2 = Arc::clone(&vec);
        let t2 = thread::spawn(move || foo(&slice2, v2));

        // Join threads and collect results
        let result1 = t1.join().unwrap().to_owned().0.get();
        let result2 = t2.join().unwrap().to_owned().0.get();

        println!("Result from foo: {:?}", result1);
        println!("Result from bar: {:?}", result2);

        let vec = vec![1, 2, 3, 4];
        let v1 = &vec[0..2];
        let v2 = &vec[2..4];
    }

    #[test]
    fn move_ownership_of_cache_and_state() {
        // Each thread sets a range of the state and cache to 1 and 0 respectively
        fn spawn_thread(
            state: &Arc<Mutex<Vec<i32>>>,
            cache: &Arc<Mutex<Vec<i32>>>,
        ) -> thread::JoinHandle<()> {
            let state = Arc::clone(state);
            let cache = Arc::clone(cache);

            let t1 = thread::spawn(move || {
                let mut cache = cache.lock().unwrap();
                let mut state = state.lock().unwrap();

                for c in cache.iter_mut() {
                    *c = 0;
                }
                for s in state.iter_mut() {
                    *s = 1;
                }
            });
            t1
        }

        let state = vec![9; 4];
        let cache = vec![9; 4];

        // Cache ranges
        let (c1, c2) = cache.split_at(2);;
        let c1 = Arc::new(Mutex::new(c1.to_vec()));
        let c2 = Arc::new(Mutex::new(c2.to_vec()));

        // State ranges
        let (s1, s2) = state.split_at(2);
        let s1 = Arc::new(Mutex::new(s1.to_vec()));
        let s2 = Arc::new(Mutex::new(s2.to_vec()));

        let t1 = spawn_thread(&s1, &c1);
        let t2 = spawn_thread(&s2, &c2);

        t1.join().unwrap();
        t2.join().unwrap();

        let s1 = Arc::clone(&s1);
        let state = s1.lock().unwrap();
        assert_eq!(state.iter().sum::<i32>(), state.len() as i32);

        let c1 = Arc::clone(&c1);
        let cache = c1.lock().unwrap();
        assert_eq!(cache.iter().sum::<i32>(), 0);

        let s2 = Arc::clone(&s2);
        let state = s2.lock().unwrap();
        assert_eq!(state.iter().sum::<i32>(), state.len() as i32);

        let c2 = Arc::clone(&c2);
        let cache = c2.lock().unwrap();
        assert_eq!(cache.iter().sum::<i32>(), 0);
    }


    #[test]
    //TODO: Benchmark with other cache options
    pub fn test_memswap_vectors() {
        const SIZE: usize = 1000;
        let mut cells: Vec<u8> = vec![1; SIZE];
        let mut cache: Vec<u8> = vec![0; SIZE];

        std::mem::swap(&mut cells, &mut cache);
        assert!(cells.iter().all(|&x| x == 0));
    }
}
