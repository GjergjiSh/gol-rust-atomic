//TODO: Remove me
#![allow(warnings)]

use super::*;

// Caching strategy for the grid
// The cache should be updated after each generation
pub trait CachingStrategy<const H: usize, const W: usize> {
    #[inline]
    fn update_cache(&mut self);
}

// Unsafe caching strategy for the grid
// The cache should be updated after each generation
pub trait UnsafeCachingStrategy<const H: usize, const W: usize> {
    #[inline]
    unsafe fn u_update_cache(&self);
}

#[cfg(test)]
mod test {
    use crate::{generator::atomic_generator, AtomicGenerator};

    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_caching_strategy() {
        // 0: State 1: Cache
        struct TestCachingStrategy((u8, u8));

        impl CachingStrategy<0, 0> for TestCachingStrategy {
            #[inline]
            fn update_cache(&mut self) {
                self.0 .1 = 1; // Cache becomes 1
                self.0 .0 = 0; // State becomes 0

                assert_ne!(self.0 .0, self.0 .1);
                self.0 .1 = self.0 .0.clone();
                assert_eq!(self.0 .0, self.0 .1);
            }
        }
    }

    #[test]
    fn test_unsafe_caching_strategy() {
        // 0: State 1: Cache
        struct TestCachingStrategy((u8, u8));

        impl UnsafeCachingStrategy<0, 0> for TestCachingStrategy {
            #[inline]
            unsafe fn u_update_cache(&self) {

            }
        }
    }
}
