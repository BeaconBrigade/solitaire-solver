//! # solitaire-solver
//!
//! Library for solving Thoughtful Solitaire
//!
//!

use solitaire_game::kplus::{action::Action, state::State, KPlusSolitaire};

use crate::{heuristic::h0, move_generation::generate_moves};

pub mod heuristic;
pub mod move_generation;

#[derive(Debug)]
pub struct Solution {
    pub moves: Vec<Action>,
}

pub fn greedy_solve(mut game: KPlusSolitaire) -> Option<Solution> {
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    while !game.state.is_win() && actions.len() > 0 {
        let mut max = (isize::MIN, None);
        for a in actions {
            let h = greedy(game.state);
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // max.1 will have to be set, because we're guaranteed one iteration
        // and anything is better than -12233333 and so on
        game.do_move(max.1.unwrap());
        moves.push(max.1.unwrap());
        actions = generate_moves(&game.state);
    }

    if game.state.is_win() {
        Some(Solution { moves })
    } else {
        None
    }
}

pub fn greedy(mut state: State) -> isize {
    let mut actions = generate_moves(&state);
    while !state.is_win() && actions.len() > 0 {
        let mut max = (isize::MIN, None);
        for a in actions {
            let h = h0(&state.apply(a));
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // max.1 will have to be set, because we're guaranteed one iteration
        // and anything is better than -12233333 and so on
        state = state.apply(max.1.unwrap());
        actions = generate_moves(&state);
    }

    h0(&state)
}
