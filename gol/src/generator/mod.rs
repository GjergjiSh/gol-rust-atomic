pub mod atomic_generator;

pub use atomic_generator::*;

pub trait SafeGenerator<const H: usize, const W: usize> {
    fn generate(&mut self);
}

pub trait UnsafeGenerator<const H: usize, const W: usize> {
    unsafe fn u_generate(&self);
}
