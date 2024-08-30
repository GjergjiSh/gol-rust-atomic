use crate::gol::cell::Cell;

// 2D: Interface to a vector of cells
pub struct Grid<const H: usize, const W: usize> {
    cells: Vec<Cell>,
}

// Implement Grid
impl<const H: usize, const W: usize> Grid<H, W> {
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(H * W);
        for _ in 0..H {
            for _ in 0..W {
                cells.push(Cell::default());
            }
        }
        Self { cells }
    }

    pub fn get(&self, x: isize, y: isize) -> &Cell {
        let wrapped_x = ((x % W as isize + W as isize) % W as isize) as usize;
        let wrapped_y = ((y % H as isize + H as isize) % H as isize) as usize;

        &self.cells[wrapped_y * W + wrapped_x]
    }

    // Testing and Debugging Only
    pub fn set(&mut self, x: isize, y: isize, value: u8) {
        let x = x as usize;
        let y = y as usize;

        let wrapped_x = x % W;
        let wrapped_y = y % H;
        self.cells[wrapped_y * W + wrapped_x].store(value);
    }
}

// Implement Display for Grid
impl<const H: usize, const W: usize> std::fmt::Display for Grid<H, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
                let index = y * W + x;
                let cell = &self.cells[index];
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
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use utils::{set_0b0001_0001, test_2d_index_translation};

    use super::*;

    mod utils {
        use super::*;

        // Set the cell at the given index to dead and 0 neighbors
        pub fn set_0b0000_0000<const H: usize, const W: usize>(grid: &mut Grid<H, W>, idx: usize) {
            let cell = &mut grid.cells[idx];

            while (cell.neighbors() > 0) {
                cell.remove_neighbor();
            }

            cell.kill();
        }

        // Set the cell at the given index to alive and 8 neighbors
        pub fn set_0b0001_0001<const H: usize, const W: usize>(grid: &mut Grid<H, W>, idx: usize) {
            let cell = &mut grid.cells[idx];

            while (cell.neighbors() < 8) {
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
            let mut grid = Grid::<H, W>::new();
            set_0b0001_0001(&mut grid, idx);

            let actual = grid.get(x, y);
            assert!(actual.alive());
            assert!(actual.neighbors() == 8);

            let expected = &grid.cells[idx];
            assert_eq!(actual.fetch(), expected.fetch());
            // println!("{}", grid); // Debugging
        }
    }

    #[test]
    fn test_new() {
        let grid = Grid::<3, 3>::new();
        assert_eq!(grid.cells.len(), 9);
        println!("{}", grid);

        // For each cell: Spawn and set neighbors to 8
        for i in 0..grid.cells.len() {
            // Initial State: 0b0000_0000
            let cell = &grid.cells[i];
            assert!(!cell.alive());
            assert!(cell.neighbors() == 0);
            assert!(cell.fetch() == 0b0000_0000);

            cell.spawn();

            for _ in 0..8 {
                cell.add_neighbor();
            }

            assert!(cell.alive());
            assert!(cell.neighbors() == 8);
            assert!(cell.fetch() == 0b0001_0001);
        }

        for i in 0..grid.cells.len() {
            let cell = &grid.cells[i];

            cell.kill();
            for _ in 0..8 {
                cell.remove_neighbor();
            }

            assert!(!cell.alive());
            assert!(cell.neighbors() == 0);
            assert!(cell.fetch() == 0b0000_0000);
        }
    }

    #[test]
    fn test_get() {
        let mut grid = Grid::<4, 4>::new();
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
    fn test_wrapping() {
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
}
