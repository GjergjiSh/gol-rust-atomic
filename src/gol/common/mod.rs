use super::grid::Grid;

pub trait IGenerator<const H: usize, const W: usize> {
    fn generate(&self);
    fn grid(&self) -> &Grid<H, W>;
}

pub trait ICell {
    fn spawn(&self);
    fn kill(&self);
    fn neighbors(&self) -> u8;
    fn add_neighbor(&self);
    fn remove_neighbor(&self);
    fn alive(&self) -> bool;
    fn fetch(&self) -> u8;
    fn store(&self, value: u8);
}

pub trait IGrid<const H: usize, const W: usize> {
    fn spawn(&self, x: isize, y: isize);
    fn kill(&self, x: isize, y: isize);
    fn unsafe_copy_from(&self, other: &Grid<H, W>);
    fn spawn_shape(&self, start: (isize, isize), offsets: &[(isize, isize)]);

    //TODO: Coupling here and LISKOV violation
    // fn get(&self, x: isize, y: isize) -> &AtomicCell;
}
