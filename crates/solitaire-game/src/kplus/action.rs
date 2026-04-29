use crate::common::Coord;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Action {
    pub from: Coord,
    pub to: Coord,
}

impl Action {
    pub fn new(from: Coord, to: Coord) -> Self {
        Self { from, to }
    }
}
