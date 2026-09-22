use std::collections::HashMap;

use solitaire_game::kplus::{action::Action, state::State};

use crate::{heuristic::h2, move_generation::generate_moves, Eval, Solver};

pub struct GreedySolver {
    // cache: LruCache<State, isize>,
}

impl GreedySolver {
    pub fn new(_capacity: usize) -> Self {
        Self {
            // cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn eval(&self, mut root_path: HashMap<State, ()>, mut state: State) -> Eval {
        // don't waste an allocation
        let mut actions = Vec::with_capacity(0);
        while !state.is_win() {
            root_path.insert(state, ());

            let mut max = (isize::MIN, None);
            actions = generate_moves(&state);
            for a in &actions {
                let new = state.apply(*a);
                // we're repeating states
                if root_path.contains_key(&new) {
                    continue;
                }
                let candidate_moves = generate_moves(&new);

                let h = h2(&new, &candidate_moves);
                if h > max.0 {
                    max = (h, Some(new));
                }
            }
            if let (_, Some(new)) = max {
                state = new;
            } else {
                // we ran out of unexplored moves
                return Eval::Loss;
            }
        }
        // whether we won, or ran out of moves, return h2
        if state.is_win() {
            Eval::Win
        } else {
            Eval::H(h2(&state, &actions))
        }
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
        root_path: &HashMap<State, ()>,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action> {
        let mut max = (isize::MIN, None);
        for a in actions {
            let new = state.apply(*a);
            // already been to this state in our path
            if root_path.contains_key(&new) {
                continue;
            }
            // let h = if let Some(h) = self.cache.get(&new) {
            //     *h
            // } else {
            let eval = self.eval(root_path.clone(), new);
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
