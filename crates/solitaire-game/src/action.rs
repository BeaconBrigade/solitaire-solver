/// Moves to apply to the solitaire board
///

#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// draw a card from stock
    TurnStock,
    /// put all cards from talon back into the stock
    ClearTalon,
    /// Move a card from talon, tableau or foundation to another location
    Move(Coord, Coord),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Coord {
    pub location: Location,
    pub idx: u8,
}

impl Coord {
    pub fn new(location: Location, idx: u8) -> Self {
        Self { location, idx }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Location {
    /// Contains which foundation stack this coord is in
    Foundation(u8),
    /// Contains which tableau stack this coord is in
    Tableau(u8),
    Talon,
}
