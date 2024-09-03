use std::{
    alloc::{alloc, Layout},
    fmt,
};

use crate::cell::SimpleCell;

// 2D interface to a heap allocated array of cells
// Changes to the contained cells require a mutable reference
// to the grid
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleGrid<const H: usize, const W: usize>(Box<[SimpleCell]>);

// Impl: SimpleGrid
impl<const H: usize, const W: usize> SimpleGrid<H, W> {
    pub fn new() -> SimpleGrid<H, W> {
        let layout = Layout::array::<SimpleCell>(H * W).unwrap();

        let ptr = unsafe { alloc(layout) as *mut SimpleCell };

        if ptr.is_null() {
            panic!("Memory allocation failed");
        }

        unsafe {
            std::ptr::write_bytes(ptr, 0b00000000, H * W);
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, H * W) };
        let data = unsafe { Box::from_raw(slice as *mut [SimpleCell]) };

        SimpleGrid(data)
    }

    #[inline]
    pub fn get(&self, x: isize, y: isize) -> &SimpleCell {
        let wrapped_x = ((x % W as isize + W as isize) % W as isize) as usize;
        let wrapped_y = ((y % H as isize + H as isize) % H as isize) as usize;
        &self.0[wrapped_y * W + wrapped_x]
    }

    #[inline]
    pub fn get_mut(&mut self, x: isize, y: isize) -> &mut SimpleCell {
        let wrapped_x = ((x % W as isize + W as isize) % W as isize) as usize;
        let wrapped_y = ((y % H as isize + H as isize) % H as isize) as usize;
        &mut self.0[wrapped_y * W + wrapped_x]
    }

    #[inline]
    pub fn rows(&self) -> usize {
        H
    }

    #[inline]
    pub fn cols(&self) -> usize {
        W
    }

    #[inline]
    pub fn spawn(&mut self, x: isize, y: isize) {
        let neighbor_coordinates = self.neighbor_coordinates(x, y);

        let cell = self.get_mut(x, y);
        cell.spawn();

        for (nx, ny) in neighbor_coordinates.iter() {
            let neighbor = self.get_mut(*nx, *ny);
            neighbor.add_neighbor();
        }
    }

    #[inline]
    pub fn kill(&mut self, x: isize, y: isize) {
        let neighbour_coordinates = self.neighbor_coordinates(x, y);

        let cell = self.get_mut(x, y);
        cell.kill();

        for (nx, ny) in neighbour_coordinates.iter() {
            let neighbour_cell = self.get_mut(*nx, *ny);
            neighbour_cell.remove_neighbor();
        }
    }

    #[inline]
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
    pub fn cells(&self) -> &Box<[SimpleCell]> {
        &self.0
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<SimpleCell> {
        self.0.iter()
    }

    pub fn print(&self) {
        // Print the top border with column indices
        print!("   "); // Space for row indices
        println!();

        // Print the top border of the grid with column numbers
        print!("  +");
        for x in 0..W {
            print!("-{}-+", x); // Col index
        }
        println!();

        // Print the field with side borders and row indices
        for y in 0..H {
            print!("{:2}|", y); // Row index
            for x in 0..W {
                let cell = self.get(x as isize, y as isize);
                let symbol = if cell.alive() { '*' } else { ' ' };
                print!(" {} |", symbol);
            }
            println!(); // End of the row with a side border

            // Print the horizontal border between rows without column numbers
            print!("  +");
            for _ in 0..H {
                print!("---+");
            }
            println!();
        }

        println!();
    }
}

// Impl: Display for SimpleGrid
impl<const H: usize, const W: usize> fmt::Display for SimpleGrid<H, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..H {
            for j in 0..W {
                write!(f, "{} ", self.0[i * W + j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_simple_grid {
    use super::SimpleGrid;

    const ARRAY_H: usize = 5;
    const ARRAY_W: usize = 5;

    fn setup() -> SimpleGrid<ARRAY_H, ARRAY_W> {
        SimpleGrid::<ARRAY_H, ARRAY_W>::new()
    }

    #[test]
    fn test_create() {
        let mut cell_array = setup();
        for x in 0..cell_array.rows() {
            for y in 0..cell_array.cols() {
                let cell = cell_array.get_mut(x as isize, y as isize);
                assert_eq!(cell.alive(), false);
                assert_eq!(cell.neighbors(), 0);
                assert_eq!(cell.to_string(), "00000000");
                assert_eq!(*cell, 0b00000000);

                cell.spawn();
                for _ in 0..8 {
                    cell.add_neighbor();
                }

                assert_eq!(cell.alive(), true);
                assert_eq!(cell.neighbors(), 8);
                assert_eq!(cell.to_string(), "00010001");
                assert_eq!(*cell == 0b00010001, true);
            }
        }
    }

    #[test]
    fn test_wrapping_top_left_to_top_right() {
        let mut cell_array = setup();

        let x = -1;
        let cell = cell_array.get_mut(x, 0);
        cell.spawn();

        let destination = cell_array.get(4, 0);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_top_right_to_top_left() {
        let mut cell_array = setup();

        let x = ARRAY_W as isize;
        let cell = cell_array.get_mut(x, 0);
        cell.spawn();

        let destination = cell_array.get(0, 0);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_bottom_left_to_bottom_right() {
        let mut cell_array = setup();

        let x = -1;
        let y = (ARRAY_H - 1) as isize;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get((ARRAY_W - 1) as isize, (ARRAY_H - 1) as isize); // Bottom-right cell
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_bottom_right_to_bottom_left() {
        let mut cell_array = setup();

        let x = ARRAY_W as isize;
        let y = (ARRAY_H - 1) as isize;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(0, (ARRAY_H - 1) as isize);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_top_right_to_bottom_left_corner() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = -1;
        let y = -1;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get((ARRAY_W - 1) as isize, (ARRAY_H - 1) as isize); // Bottom-right cell
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_bottom_left_to_top_right_corner() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = ARRAY_W as isize;
        let y = ARRAY_H as isize;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(0, 0);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_top_left_to_bottom_left_corner() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = 0;
        let y = -1;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(0, (ARRAY_H - 1) as isize);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_top_right_to_bottom_right() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = ARRAY_W as isize - 1;
        let y = -1;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(ARRAY_W as isize - 1, ARRAY_H as isize - 1);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_bottom_left_to_top_left() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = 0;
        let y = ARRAY_H as isize;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(0, 0);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_wrapping_bottom_right_to_top_right() {
        let mut cell_array = SimpleGrid::<ARRAY_H, ARRAY_W>::new();

        let x = ARRAY_W as isize - 1;
        let y = ARRAY_H as isize;
        let cell = cell_array.get_mut(x, y);
        cell.spawn();

        let destination = cell_array.get(ARRAY_W as isize - 1, 0);
        assert_eq!(destination.alive(), true);
    }

    #[test]
    fn test_glider() {
        let mut cell_array = setup();

        let x = 0;
        let y = 0;
        let pattern_coords = [
            (x + 2, y),
            (x + 2, y + 1),
            (x + 2, y + 2),
            (x + 1, y + 2),
            (x, y + 1),
        ];

        for &(x, y) in &pattern_coords {
            cell_array.spawn(x, y);
        }

        cell_array.print();

        //First column
        let c1 = cell_array.get_mut(0, 0);
        let c1_neighbors = c1.neighbors();

        assert_eq!(c1.alive(), false);
        assert_eq!(c1_neighbors, 1);
        assert_eq!(c1.to_string(), "00000010");
        assert_eq!(*c1 == 0b00000010, true);

        let c2 = cell_array.get_mut(0, 1);
        let c2_neighbors = c2.neighbors();

        assert_eq!(c2.alive(), true);
        assert_eq!(c2_neighbors, 1);
        assert_eq!(c2.to_string(), "00000011");
        assert_eq!(*c2 == 0b00000011, true);

        let c3 = cell_array.get(0, 2);
        let c3_neighbors = c3.neighbors();
        assert_eq!(c3.alive(), false);
        cell_array.print();
        assert_eq!(c3_neighbors, 2);

        let c4 = cell_array.get(0, 3);
        let c4_neighbors = c4.neighbors();
        assert_eq!(c4.alive(), false);
        assert_eq!(c4_neighbors, 1);

        let c5 = cell_array.get(0, 4);
        let c5_neighbors = c5.neighbors();
        assert_eq!(c5.alive(), false);
        assert_eq!(c5_neighbors, 0);

        //Second column
        let c6 = cell_array.get(1, 0);
        let c6_neighbors = c6.neighbors();
        assert_eq!(c6.alive(), false);
        assert_eq!(c6_neighbors, 3);

        let c7 = cell_array.get(1, 1);
        let c7_neighbors = c7.neighbors();
        assert_eq!(c7.alive(), false);
        assert_eq!(c7_neighbors, 5);

        let c8 = cell_array.get(1, 2);
        let c8_neighbors = c8.neighbors();
        assert_eq!(c8.alive(), true);
        assert_eq!(c8_neighbors, 3);

        let c9 = cell_array.get(1, 3);
        let c9_neighbors = c9.neighbors();
        assert_eq!(c9.alive(), false);
        assert_eq!(c9_neighbors, 2);

        let c10 = cell_array.get(1, 4);
        let c10_neighbors = c10.neighbors();
        assert_eq!(c10.alive(), false);
        assert_eq!(c10_neighbors, 1);

        //Third column
        let c11 = cell_array.get(2, 0);
        let c11_neighbors = c11.neighbors();
        assert_eq!(c11.alive(), true);
        assert_eq!(c11_neighbors, 1);

        let c12 = cell_array.get(2, 1);
        let c12_neighbors = c12.neighbors();
        assert_eq!(c12.alive(), true);
        assert_eq!(c12_neighbors, 3);

        let c13 = cell_array.get(2, 2);
        let c13_neighbors = c13.neighbors();
        assert_eq!(c13.alive(), true);
        assert_eq!(c13_neighbors, 2);

        let c14 = cell_array.get(2, 3);
        let c14_neighbors = c14.neighbors();
        assert_eq!(c14.alive(), false);
        assert_eq!(c14_neighbors, 2);

        let c15 = cell_array.get(2, 4);
        let c15_neighbors = c15.neighbors();
        assert_eq!(c15.alive(), false);
        assert_eq!(c15_neighbors, 1);

        //Fourth column
        let c16 = cell_array.get(3, 0);
        let c16_neighbors = c16.neighbors();
        assert_eq!(c16.alive(), false);
        assert_eq!(c16_neighbors, 2);

        let c17 = cell_array.get(3, 1);
        let c17_neighbors = c17.neighbors();
        assert_eq!(c17.alive(), false);
        assert_eq!(c17_neighbors, 3);

        let c18 = cell_array.get(3, 2);
        let c18_neighbors = c18.neighbors();
        assert_eq!(c18.alive(), false);
        assert_eq!(c18_neighbors, 2);

        let c19 = cell_array.get(3, 3);
        let c19_neighbors = c19.neighbors();
        assert_eq!(c19.alive(), false);
        assert_eq!(c19_neighbors, 1);

        let c20 = cell_array.get(3, 4);

        let c20_neighbors = c20.neighbors();
        assert_eq!(c20.alive(), false);
        assert_eq!(c20_neighbors, 1);

        //Fifth column
        let c21 = cell_array.get(4, 0);
        let c21_neighbors = c21.neighbors();
        assert_eq!(c21.alive(), false);
        assert_eq!(c21_neighbors, 1);

        let c22 = cell_array.get(4, 1);
        let c22_neighbors = c22.neighbors();
        assert_eq!(c22.alive(), false);
        assert_eq!(c22_neighbors, 1);

        let c23 = cell_array.get(4, 2);
        let c23_neighbors = c23.neighbors();
        assert_eq!(c23.alive(), false);
        assert_eq!(c23_neighbors, 1);

        let c24 = cell_array.get(4, 3);
        let c24_neighbors = c24.neighbors();
        assert_eq!(c24.alive(), false);
        assert_eq!(c24_neighbors, 0);

        let c25 = cell_array.get(4, 4);
        let c25_neighbors = c25.neighbors();
        assert_eq!(c25.alive(), false);
        assert_eq!(c25_neighbors, 0);
    }
}