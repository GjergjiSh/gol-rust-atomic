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
