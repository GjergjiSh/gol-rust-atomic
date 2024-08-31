use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleCell(u8);

impl SimpleCell {
    #[allow(dead_code)]
    pub fn new() -> SimpleCell {
        SimpleCell(0)
    }

    #[inline]
    // Bitwise operation to set the first bit to 1
    pub fn spawn(&mut self) {
        self.0 |= 1;
    }

    #[inline]
    // Bitwise operation to set the first bit to 0
    pub fn kill(&mut self) {
        self.0 &= !1;
    }

    #[inline]
    // Bitwise operation to check if the first bit is 1
    pub fn alive(&self) -> bool {
        self.0 & 1 == 1
    }

    #[inline]
    // Bitwise operation to get the number of neighbors
    pub fn neighbors(&self) -> u8 {
        (self.0 >> 1) & 0b0000_1111
    }

    #[inline]
    // Bitwise operation to increment the number of neighbors
    pub fn add_neighbor(&mut self) {
        let count = (self.0 >> 1) & 0b1111;
        assert!(count + 1 <= 8, "Neighbor count must be between 0 and 8");
        self.0 = (self.0 & 0b0000_0001) | ((count + 1) << 1);
    }

    #[inline]
    // Bitwise operation to decrement the number of neighbors
    pub fn remove_neighbor(&mut self) {
        let count = (self.0 >> 1) & 0b1111;
        self.0 = (self.0 & 0b0000_0001) | ((count - 1) << 1);
    }

    #[inline]
    pub fn store(&mut self, value: u8) {
        self.0 = value;
    }

    #[inline]
    pub fn fetch(&self) -> u8 {
        self.0
    }
}

// Implement PartialEq<u8> for SimpleCell
impl PartialEq<u8> for SimpleCell {
    fn eq(&self, other: &u8) -> bool {
        &self.0 == other
    }
}

// Implement Display for SimpleCell
impl fmt::Display for SimpleCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08b}", self.0)
    }
}

mod test_cell {
    #[allow(unused_imports)]
    use crate::cell::SimpleCell;

    #[test]
    fn test_spawn() {
        let mut cell = SimpleCell::new();
        cell.add_neighbor();
        assert_eq!(cell, 0b00000010);
        assert_eq!(cell.alive(), false);
        assert_eq!(cell.neighbors(), 1);
        assert_eq!(cell.to_string(), "00000010");
        cell.spawn();
        assert_eq!(cell.alive(), true);
        assert_eq!(cell.neighbors(), 1);
        assert_eq!(cell.to_string(), "00000011");
        assert_eq!(cell == 0b00000011, true);
    }

    #[test]
    fn test_kill() {
        let mut cell = SimpleCell::new();
        cell.spawn();
        cell.add_neighbor();
        cell.kill();
        assert_eq!(cell.alive(), false);
        assert_eq!(cell.to_string(), "00000010");
        assert_eq!(cell == 0b00000010, true);
    }

    #[test]
    fn test_increment_neighbors() {
        let mut cell = SimpleCell::new();
        cell.add_neighbor();
        assert_eq!(cell.neighbors(), 1);
        assert_eq!(cell.to_string(), "00000010");
        assert_eq!(cell == 0b00000010, true);

        cell.add_neighbor();
        assert_eq!(cell.neighbors(), 2);
        assert_eq!(cell.to_string(), "00000100");
        assert_eq!(cell == 0b00000100, true);

        cell.add_neighbor();
        assert_eq!(cell.neighbors(), 3);
        assert_eq!(cell.to_string(), "00000110");
        assert_eq!(cell == 0b00000110, true);
    }

    #[test]
    fn test_decrement_neighbors() {
        let mut cell = SimpleCell::new();
        cell.add_neighbor();
        assert_eq!(cell.neighbors(), 1);
        assert_eq!(cell.to_string(), "00000010");
        assert_eq!(cell == 0b00000010, true);
        cell.remove_neighbor();
        assert_eq!(cell.neighbors(), 0);
        assert_eq!(cell.to_string(), "00000000");
        assert_eq!(cell == 0b00000000, true);
    }
}
