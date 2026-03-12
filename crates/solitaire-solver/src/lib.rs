//! # solitaire-solver
//!
//! Library for solving Thoughtful Solitaire
//!
//!

use solitaire_game::kplus::action::Action;

pub mod heuristic;
pub mod move_generation;
pub mod greedy;

#[derive(Debug)]
pub struct Solution {
    pub moves: Vec<Action>,
}
