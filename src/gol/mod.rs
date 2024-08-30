//TODO: Remove me
#![allow(warnings)]

pub mod cell;
pub mod grid;
pub mod generator;
pub mod display;
pub mod utils;
pub mod common;
pub mod launcher;

pub use cell::{AtomicCell, CellType};
pub use grid::AtomicGrid;
pub use generator::SingleThreadedGenerator;
pub use display::Display;
pub use utils::randomize_grid;
pub use common::{Generator, Cell};

pub use std::sync::Arc;