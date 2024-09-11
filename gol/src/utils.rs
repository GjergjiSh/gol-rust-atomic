use std::fs::File;

use crate::grid::AtomicGrid;

use csv::ReaderBuilder;
use rand::random;

pub fn randomize_grid<const H: usize, const W: usize>(grid: &AtomicGrid<H, W>) {
    for x in 0..H {
        for y in 0..W {
            if random() {
                let x = x as isize;
                let y = y as isize;
                grid.spawn(x, y);
            }
        }
    }
}

pub fn create_atomic_grid_from_file<const H: usize, const W: usize>(
    path: &str,
) -> AtomicGrid<H, W> {
    let grid = AtomicGrid::<H, W>::new();
    let file = File::open(path).unwrap();
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    for (y, result) in rdr.records().enumerate() {
        let record = result.unwrap();
        for (x, value) in record.iter().enumerate() {
            let x = x as isize;
            let y = y as isize;
            if value == "1" {
                grid.spawn(x, y);
            }
        }
    }

    grid
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_randomize_grid() {
        let grid = AtomicGrid::<10, 10>::new();
        let state = grid.clone();
        randomize_grid(&grid);
        let randomized_grid = grid.clone();
        assert_ne!(randomized_grid, state);
        // TODO
        // randomize_grid(&grid);
        // assert_ne!(randomized_grid, grid.clone());
    }

    #[test]
    fn test_create_atomic_grid_from_file() {
        const H: usize = 6;
        const W: usize = 6;
        //TODO: Fix path
        let test_grid = r#"../../resources/test/corner_block.csv"#;
        let test_grid = create_atomic_grid_from_file::<H, W>(&test_grid);

        let c1 = test_grid.get(0, 0);
        let c2 = test_grid.get(0, 1);
        let c3 = test_grid.get(1, 0);
        let c4 = test_grid.get(1, 1);

        assert_eq!(c1.neighbors(), 3);
        assert_eq!(c2.neighbors(), 3);
        assert_eq!(c3.neighbors(), 3);
        assert_eq!(c4.neighbors(), 3);

        assert!(c1.alive());
        assert!(c2.alive());
        assert!(c3.alive());
        assert!(c4.alive());

        assert_eq!(c1.fetch(), 0b000_0011_1);
        assert_eq!(c2.fetch(), 0b000_0011_1);
        assert_eq!(c3.fetch(), 0b000_0011_1);
        assert_eq!(c4.fetch(), 0b000_0011_1);
    }
}
