use crate::SimpleCell;

// Uses a Vec instead of a manually allocated array
pub struct SimpleGridWithVec<const H: usize, const W: usize> {
    cells: Vec<SimpleCell>,
}

// Implement SimpleGridWithVec
impl<const H: usize, const W: usize> SimpleGridWithVec<H, W> {
    pub fn new() -> SimpleGridWithVec<H, W> {
        let cells = vec![SimpleCell::new(); H * W];
        SimpleGridWithVec { cells }
    }

    #[inline]
    pub fn get(&self, x: isize, y: isize) -> &SimpleCell {
        let wrapped_x = ((x % W as isize + W as isize) % W as isize) as usize;
        let wrapped_y = ((y % H as isize + H as isize) % H as isize) as usize;
        &self.cells[wrapped_y * W + wrapped_x]
    }

    #[inline]
    pub fn get_mut(&mut self, x: isize, y: isize) -> &mut SimpleCell {
        let wrapped_x = ((x % W as isize + W as isize) % W as isize) as usize;
        let wrapped_y = ((y % H as isize + H as isize) % H as isize) as usize;
        &mut self.cells[wrapped_y * W + wrapped_x]
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
    pub fn cells(&self) -> &Vec<SimpleCell> {
        &self.cells
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<SimpleCell> {
        self.cells.iter()
    }

    pub fn print(&self) {}
}

#[cfg(test)]
mod test_simple_grid_with_vec {
    use super::*;

    const H: usize = 100;
    const W: usize = 100;

    #[test]
    pub fn test_simple_grid_with_vec() {
        #![allow(warnings)]

        let mut cell_array = SimpleGridWithVec::<H, W> {
            cells: vec![SimpleCell::new(); H * W],
        };

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
}
