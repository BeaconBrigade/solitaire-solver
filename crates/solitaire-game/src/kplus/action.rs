use crate::common::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action {
    pub from: Coord,
    pub to: Coord,
}

impl Action {
    pub fn new(from: Coord, to: Coord) -> Self {
        Self { from, to }
    }
}
