use std::collections::HashMap;

use solitaire_game::kplus::{action::Action, state::State};

use crate::{greedy::GreedySolver, heuristic::h2, move_generation::generate_moves, Eval, Solver};

pub struct NestedRolloutSolver {
    // caches: Vec<LruCache<State, Eval>>,
    depth: usize,
}

impl NestedRolloutSolver {
    pub fn new(_capacity: usize, depth: usize) -> Self {
        Self {
            // caches: Vec::from_iter(
            //     std::iter::repeat_with(|| LruCache::new(NonZeroUsize::new(capacity).unwrap()))
            //         .take(depth),
            // ),
            depth,
        }
    }

    pub fn eval(
        &mut self,
        mut root_path: HashMap<State, ()>,
        mut state: State,
        depth: usize,
    ) -> Eval {
        let mut actions = Vec::with_capacity(0);
        while !state.is_win() {
            root_path.insert(state, ());

            let mut max = (Eval::Loss, None);
            actions = generate_moves(&state);
            for a in &actions {
                let new = state.apply(*a);
                // we're repeating states
                if root_path.contains_key(&new) {
                    continue;
                }
                let eval = if depth >= self.depth {
                    // fall back to greedy eval on the last level
                    GreedySolver::new(1).eval(root_path.clone(), new)
                }
                /*else if let Some(eval) = self.caches[n].get(&new) {
                    *eval
                } */
                else {
                    self.eval(root_path.clone(), new, depth + 1)
                };
                if eval > max.0 {
                    max = (eval, Some(new));
                }
            }
            if let (_, Some(new)) = max {
                state = new;
            } else {
                // we ran out of unexplored moves
                return Eval::Loss;
            }
        }
        // if n < self.depth {
        //     self.caches[n].put(original_state, eval);
        // }
        if state.is_win() {
            Eval::Win
        } else {
            Eval::H(h2(&state, &actions))
        }
    }
}

impl Default for NestedRolloutSolver {
    fn default() -> Self {
        Self::new(50_000, 3)
    }
}

impl Solver for NestedRolloutSolver {
    fn next_move(
        &mut self,
        root_path: &HashMap<State, ()>,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action> {
        let mut max = (Eval::Loss, None);
        for a in actions {
            let new = state.apply(*a);
            // already been to this state in our path
            if root_path.contains_key(&new) {
                continue;
            }
            // let eval = if let Some(eval) = self.caches[0].get(&new) {
            //     *eval
            // } else {
            let eval = self.eval(root_path.clone(), new, 0);
            // };

            // self.caches[0].put(new, eval);
            if eval > max.0 {
                max = (eval, Some(*a));
            }
        }

        max.1
    }
}
