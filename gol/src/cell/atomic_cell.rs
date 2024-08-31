use std::{
    fmt,
    sync::atomic::{
        AtomicU8,
        Ordering::{self, AcqRel, Acquire, Release},
    },
};

// Wrapper around an AtomicU8 to represent a cell in the grid
pub struct AtomicCell {
    state: AtomicU8,
    fetch: Ordering,
    store: Ordering,
}

// Implement Cell: All functions are atomic operations
impl AtomicCell {
    // Creates a new cell with the specified load and store orderings
    pub fn new(fetch: Ordering, store: Ordering) -> Self {
        assert_ne!(fetch, AcqRel, "Fetch ordering for AtomicCell cannot be AcqRel");
        assert_ne!(store, AcqRel, "Store ordering for AtomicCell cannot be AcqRel");
        assert_ne!(fetch, Release, "Fetch ordering for AtomicCell cannot be Release");
        assert_ne!(store, Acquire, "Store ordering for AtomicCell cannot be Acquire");
        AtomicCell {
            state: AtomicU8::new(0),
            fetch,
            store,
        }
    }

    #[inline]
    // Bitwise atomic operation to set the first bit to 1
    pub fn spawn(&self) {
        self.state
            .fetch_update(self.store, self.fetch, |old| Some(old | 1))
            .unwrap();
    }

    #[inline]
    // Bitwise atomic operation to set the first bit to 0
    pub fn kill(&self) {
        self.state
            .fetch_update(self.store, self.fetch, |old| Some(old & !1))
            .unwrap();
    }

    #[inline]
    // Bitwise atomic operation to get the number of neighbors
    pub fn neighbors(&self) -> u8 {
        (self.state.load(self.fetch) >> 1) & 0b0000_1111
    }

    #[inline]
    // Bitwise atomic operation to increment the number of neighbors
    pub fn add_neighbor(&self) {
        self.state
            .fetch_update(self.store, self.fetch, |mut old| {
                let count = (old >> 1) & 0b1111;
                if count + 1 <= 8 {
                    old = (old & 0b0000_0001) | ((count + 1) << 1);
                    Some(old)
                } else {
                    None
                }
            })
            .expect(&format!(
                "Add: Neighbor count must be between 0 and 8, is currently {}",
                self.neighbors()
            ));
    }

    #[inline]
    // Bitwise atomic operation to decrement the number of neighbors
    pub fn remove_neighbor(&self) {
        self.state
            .fetch_update(self.store, self.fetch, |mut old| {
                let count = (old >> 1) & 0b1111;
                if count > 0 {
                    old = (old & 0b0000_0001) | ((count - 1) << 1);
                    Some(old)
                } else {
                    None
                }
            })
            .expect(&format!(
                "Remove: Neighbor count must be between 0 and 8, is currently {}",
                self.neighbors()
            ));
    }

    #[inline]
    // Bitwise atomic operation, returns true if the first bit is 1
    pub fn alive(&self) -> bool {
        self.state.load(self.fetch) & 1 == 1
    }

    #[inline]
    // Atomically loads the value of the cell with the specified ordering
    pub fn fetch(&self) -> u8 {
        self.state.load(self.fetch)
    }

    #[inline]
    // Atomically stores the value of the cell with the specified ordering
    pub fn store(&self, value: u8) {
        self.state.store(value, self.store);
    }

    #[inline]
    // Atomically exchange the value of the cell with another cell
    pub fn compare_and_exchange(&self, other: &AtomicCell) {
        let _ = self.state.compare_exchange(
            self.state.load(self.fetch),
            other.fetch(),
            self.fetch,
            self.store,
        );
    }
}

// Implement Default for AtomicCell
impl Default for AtomicCell {
    fn default() -> Self {
        AtomicCell::new(Ordering::Acquire, Ordering::Release)
    }
}

// Implement PartialEq<u8> for AtomicCell
impl PartialEq<u8> for AtomicCell {
    fn eq(&self, other: &u8) -> bool {
        self.state.load(self.fetch) == *other
    }
}

// TODO: Theory Implement Clone for AtomicCell
impl Clone for AtomicCell {
    fn clone(&self) -> Self {
        AtomicCell {
            state: AtomicU8::new(self.state.load(self.fetch)),
            fetch: self.fetch,
            store: self.store,
        }
    }
}

// Implement Display for AtomicCell
impl fmt::Display for AtomicCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08b}", self.fetch())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::UnsafeCell,
        sync::{atomic::AtomicI32, Arc},
    };

    use super::*;
    #[test]
    fn test_spawn_kill() {
        let cell = AtomicCell::default();
        cell.spawn();
        assert_eq!(cell.fetch(), 1);
        assert!(cell.alive());
        cell.kill();
        assert_eq!(cell.fetch(), 0);
        assert!(!cell.alive());
        cell.spawn();
        assert_eq!(cell.fetch(), 1);
        assert!(cell.alive());
    }

    #[test]
    fn test_neighbors() {
        let cell = AtomicCell::default();
        assert_eq!(cell.neighbors(), 0);
        assert!(!cell.alive());
        assert!(cell.fetch() == 0b0000_0000);

        // Spawn the cell to test if incrementing affects the first bit
        cell.spawn();

        let expected_values: [u8; 8] = [
            0b000_0001_1, // Alive and 1 neighbor
            0b000_0010_1, // Alive and 2 neighbors
            0b000_0011_1, // Alive and 3 neighbors
            0b000_0100_1, // Alive and 4 neighbors
            0b000_0101_1, // Alive and 5 neighbors
            0b000_0110_1, // Alive and 6 neighbors
            0b000_0111_1, // Alive and 7 neighbors
            0b000_1000_1, // Alive and 8 neighbors
        ];

        // Initially there are no neighbors and the cell is alive
        assert_eq!(cell.fetch(), 0b0000_0001);
        assert!(cell.alive());
        assert!(cell.neighbors() == 0);

        // Add neighbors starting from none to 8 and check the expected values
        for idx in 0..8 {
            cell.add_neighbor();
            let expected = expected_values[idx];
            assert_eq!(cell.fetch(), expected);
            assert_eq!(cell.neighbors(), (idx + 1) as u8);
            assert!(cell.alive());
        }

        // Kill the cell to test if decrementing affects the first bit
        cell.kill();

        let expected_values: [u8; 8] = [
            0b000_0111_0, // Alive and 7 neighbors
            0b000_0110_0, // Alive and 6 neighbors
            0b000_0101_0, // Alive and 5 neighbors
            0b000_0100_0, // Alive and 4 neighbors
            0b000_0011_0, // Alive and 3 neighbors
            0b000_0010_0, // Alive and 2 neighbors
            0b000_0001_0, // Alive and 1 neighbor
            0b000_0000_0, // Alive and 0 neighbors
        ];

        // Initially there are 8 neighbors and the cell is dead
        assert_eq!(cell.fetch(), 0b0001_0000);
        assert!(!cell.alive());
        assert!(cell.neighbors() == 8);

        // Remove neighbors starting from 8 to none
        for idx in 0..8 {
            cell.remove_neighbor();
            let expected = expected_values[idx];
            assert_eq!(cell.fetch(), expected);
            assert_eq!(cell.neighbors(), (7 - idx) as u8);
            assert!(!cell.alive());
        }
    }

    #[test]
    fn test_data_race() {
        use std::thread;
        use std::time::Duration;

        struct Wrapper(UnsafeCell<i32>);

        unsafe impl Sync for Wrapper {}

        fn get_unsafe_value(value: &Arc<Wrapper>) -> &mut i32 {
            unsafe { &mut *value.0.get() }
        }

        let value = Arc::new(Wrapper(UnsafeCell::new(0)));

        let iterations = 100_000;
        let thread_count = 4;
        let expected_value = iterations * thread_count;
        let mut handles = vec![];

        for _ in 0..thread_count {
            let value_clone = Arc::clone(&value);
            let handle = thread::spawn(move || {
                for _ in 0..iterations {
                    let value = get_unsafe_value(&value_clone);
                    *value += 1;
                    // Small sleep to increase chance of interleaving
                    thread::sleep(Duration::from_nanos(1));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // The value should not be what we expect because of the interleaving
        assert_ne!(unsafe { *value.0.get() }, expected_value);

        // Relaxed is enough for increments
        const FETCH: Ordering = Ordering::Relaxed;

        struct AtomicWrapper(AtomicI32);

        impl AtomicWrapper {
            fn increment(&self) {
                self.0.fetch_add(1, FETCH);
            }
        }

        let value = Arc::new(AtomicWrapper(AtomicI32::new(0)));
        let mut handles = vec![];

        // Spawn threads to increment the value
        for _ in 0..thread_count {
            let value_clone = Arc::clone(&value);
            let handle = thread::spawn(move || {
                for _ in 0..iterations {
                    value_clone.increment();
                    // Small sleep to increase chance of interleaving
                    thread::sleep(Duration::from_nanos(1));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // The value should be what we expect because of the atomic operations
        assert_eq!(value.0.load(FETCH), expected_value);
    }
}
