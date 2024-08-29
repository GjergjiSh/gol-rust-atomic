use crate::gol::cell::Cell;

pub struct Grid<const H: usize, const W: usize> {
    cells: Vec<Cell>,
}

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

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        let wrapped_x = x % W;
        let wrapped_y = y % H;
        &self.cells[wrapped_y * W + wrapped_x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let grid = Grid::<3, 3>::new();
        assert_eq!(grid.cells.len(), 9);
    }

    #[test]
    fn test_get() {
        /*
           *  *  *
           * [*][*][0][0]
           * [*][*][0][0]
             [0][0][0][*]* *
                       *
                       *
        */

        let grid = Grid::<4, 4>::new();
        let cell = grid.get(4, 4);
        assert_eq!(cell.load(), 0);
    }
}
