use crate::common::Coord;

/// Moves to apply to the solitaire board
///

#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// draw a card from stock
    TurnStock,
    /// Move a card from talon, tableau or foundation to another location
    Move(Coord, Coord),
}
