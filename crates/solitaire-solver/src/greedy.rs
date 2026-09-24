use std::num::NonZeroUsize;

use lru::LruCache;
use solitaire_game::kplus::{action::Action, state::State};

use crate::{heuristic::h2, move_generation::generate_moves, Eval, RootPath, Solver};

/// Solve the game using greedy rollouts
///
/// PLAY - defines whether eval returns Eval::Loss on dead ends (PLAY)
///        or heuristic score (!PLAY, better for nested search)
pub struct GreedySolver {
    // cache: LruCache<State, isize>,
}

pub static mut MOVE_CACHE_HITS: usize = 0;
pub static mut MOVE_CACHE_MISS: usize = 0;
pub static mut MOVE_CACHE_AVG_SIZE: f32 = 0.;

impl GreedySolver {
    pub fn new(_capacity: usize) -> Self {
        Self {
            // cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn eval(
        &self,
        root_path: &mut RootPath,
        mut state: State,
        move_cache: &mut LruCache<State, Vec<Action>>,
    ) -> Eval {
        let horizon = root_path.len();
        while !state.is_win() {
            root_path.insert(state);

            let mut max = (isize::MIN, None);
            // let actions = move_cache.get_or_insert(state, || generate_moves(&state));
            // try to get an idea of how useful the move_cache is
            unsafe {
                MOVE_CACHE_AVG_SIZE += 1.0 / (MOVE_CACHE_HITS as f32 + MOVE_CACHE_MISS as f32 + 1.0)
                    * (move_cache.len() as f32 - MOVE_CACHE_AVG_SIZE)
            }
            let actions = match move_cache.get(&state) {
                Some(a) => {
                    unsafe { MOVE_CACHE_HITS += 1 }
                    a
                }
                None => {
                    unsafe { MOVE_CACHE_MISS += 1 }
                    // guaranteed to insert, but it nicely returns what we put in
                    move_cache.get_or_insert(state, || generate_moves(&state))
                }
            };
            for a in actions {
                let new = state.apply_sorted(*a);
                // we're repeating states
                if root_path.contains(&new) {
                    continue;
                }
                let candidate_moves = generate_moves(&new);

                let h = h2(&new, &candidate_moves);
                if h > max.0 {
                    max = (h, Some(new));
                }
            }
            if let Some(new) = max.1 {
                state = new;
            } else {
                // we ran out of unexplored moves
                root_path.rollback_to(horizon);
                return Eval::H(h2(&state, &actions));
            }
        }
        root_path.rollback_to(horizon);

        Eval::Win
    }
}

impl Default for GreedySolver {
    fn default() -> Self {
        Self::new(50_000)
    }
}

impl Solver for GreedySolver {
    fn next_move(
        &mut self,
        root_path: &mut RootPath,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action> {
        let mut max = (isize::MIN, None);
        for a in actions {
            let new = state.apply_sorted(*a);
            // already been to this state in our path
            if root_path.contains(&new) {
                continue;
            }
            // let h = if let Some(h) = self.cache.get(&new) {
            //     *h
            // } else {
            let eval = self.eval(
                root_path,
                new,
                &mut LruCache::new(NonZeroUsize::new(1).unwrap()),
            );
            let h = match eval {
                Eval::Loss => isize::MIN + 1,
                Eval::Win => isize::MAX,
                Eval::H(h) => h,
            };
            // };

            // self.cache.put(new, h);
            if h > max.0 {
                max = (h, Some(*a));
            }
        }

        max.1
    }
}
