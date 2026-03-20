//! # solitaire-solver
//!
//! Library for solving Thoughtful Solitaire
//!
//!

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use solitaire_game::kplus::action::Action;

pub mod greedy;
pub mod heuristic;
pub mod move_generation;
pub mod nested_rollout;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Solution {
    pub moves: Vec<Action>,
}

#[derive(PartialEq, Eq)]
pub enum Eval {
    Win(Vec<Action>),
    Loss,
    H(isize),
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Eval::*;
        match (self, other) {
            (Win(s), Win(o)) => s.len().partial_cmp(&o.len()),
            (Win(_), _) => Some(Ordering::Greater),
            (_, Win(_)) => Some(Ordering::Less),
            (Loss, Loss) => Some(Ordering::Equal),
            (Loss, _) => Some(Ordering::Less),
            (_, Loss) => Some(Ordering::Greater),
            (H(s), H(o)) => s.partial_cmp(o),
        }
    }
}
