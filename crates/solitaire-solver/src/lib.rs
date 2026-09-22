//! # solitaire-solver
//!
//! Library for solving Thoughtful Solitaire
//!
//!

use std::{cmp::Ordering, collections::HashMap};

use serde::{Deserialize, Serialize};
use solitaire_game::kplus::{action::Action, state::State, KPlusSolitaire};

use crate::move_generation::generate_moves;

pub mod greedy;
pub mod heuristic;
pub mod move_generation;
// pub mod multistage_nested_rollout;
pub mod nested_rollout;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Solution {
    pub moves: Vec<Action>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Eval {
    Win,
    Loss,
    H(isize),
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Eval::*;
        match (self, other) {
            (Win, Win) => Some(Ordering::Equal),
            (Win, _) => Some(Ordering::Greater),
            (_, Win) => Some(Ordering::Less),
            (Loss, Loss) => Some(Ordering::Equal),
            (Loss, _) => Some(Ordering::Less),
            (_, Loss) => Some(Ordering::Greater),
            (H(s), H(o)) => s.partial_cmp(o),
        }
    }
}

pub trait Solver {
    fn next_move(
        &mut self,
        root_path: &HashMap<State, ()>,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action>;

    fn play_game(&mut self, mut game: KPlusSolitaire) -> Option<Solution> {
        let mut root_path = HashMap::new();
        let mut moves = Vec::new();
        // slight waste
        let mut actions = generate_moves(&game.state);
        while !game.state.is_win() && !actions.is_empty() {
            actions = generate_moves(&game.state);
            root_path.insert(game.state, ());

            // if there's no new moves, we have basically lost
            let Some(a) = self.next_move(&root_path, &game.state, &actions) else {
                break;
            };
            moves.push(a);
            game.do_move(a);
        }

        if game.state.is_win() {
            Some(Solution { moves })
        } else {
            None
        }
    }
}
