//! # solitaire-solver
//!
//! Library for solving Thoughtful Solitaire
//!
//!

use std::cmp::Ordering;

use lru::LruCache;
use rustc_hash::FxHashSet;
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
        root_path: &mut RootPath,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action>;

    fn play_game(&mut self, mut game: KPlusSolitaire) -> Option<Solution> {
        let mut root_path = RootPath::new();
        let mut moves = Vec::new();
        // slight waste
        let mut actions = generate_moves(&game.state);
        while !game.state.is_win() && !actions.is_empty() {
            actions = generate_moves(&game.state);
            root_path.insert(game.state);
            // horizon after the insert, because we want this state to stay in the root_path
            let horizon = root_path.len();

            // if there's no new moves, we have basically lost
            let Some(a) = self.next_move(&mut root_path, &game.state, &actions) else {
                break;
            };
            // remove extra stuff added by the solver in case they didn't remove everything
            root_path.rollback_to(horizon);
            moves.push(a);
            game.do_move_sorted(a);
        }

        if game.state.is_win() {
            Some(Solution { moves })
        } else {
            None
        }
    }
}

/// Vec + HashSet to keep track of visited states without having to clone a ton
#[derive(Default)]
pub struct RootPath {
    history: Vec<State>,
    visited: FxHashSet<State>,
}

impl RootPath {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn contains(&self, state: &State) -> bool {
        self.visited.contains(state)
    }

    pub fn insert(&mut self, state: State) {
        self.history.push(state);
        self.visited.insert(state);
    }

    pub fn rollback_to(&mut self, horizon: usize) {
        let l = self.len();
        for _ in horizon..l {
            let s = self
                .history
                .pop()
                .expect("i<self.len() so pop should always have an element");
            self.visited.remove(&s);
        }
    }

    /// Rolls the RootPath back a horizon while caching the final heuristic for each popped state
    pub fn rollback_with_cache(
        &mut self,
        horizon: usize,
        cache: &mut LruCache<State, Eval>,
        eval: Eval,
    ) {
        let l = self.len();
        for _ in horizon..l {
            let s = self.history.pop().unwrap();
            cache.put(s, eval);
            self.visited.remove(&s);
        }
    }
}
