use std::num::NonZeroUsize;

use lru::LruCache;
use solitaire_game::kplus::{action::Action, state::State};

use crate::{
    greedy::GreedySolver, heuristic::h2, move_generation::generate_moves, Eval, RootPath, Solver,
};

pub struct NestedRolloutSolver {
    eval_caches: Vec<LruCache<State, Eval>>,
    move_caches: LruCache<State, Vec<Action>>,
    max_depth: usize,
    greedy: GreedySolver,
}

impl NestedRolloutSolver {
    pub fn new(capacity: usize, max_depth: usize) -> Self {
        Self {
            eval_caches: Vec::from_iter(
                std::iter::repeat_with(|| LruCache::new(NonZeroUsize::new(capacity).unwrap()))
                    .take(max_depth),
            ),
            move_caches: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            max_depth,
            greedy: GreedySolver::new(1),
        }
    }

    pub fn eval(&mut self, root_path: &mut RootPath, mut state: State, depth: usize) -> Eval {
        // depth zero falls back to greedy
        if depth == self.max_depth {
            return self.greedy.eval(root_path, state, &mut self.move_caches);
        }
        if let Some(eval) = self.eval_caches[depth].get(&state) {
            return *eval;
        }
        let horizon = root_path.len();
        while !state.is_win() {
            root_path.insert(state);

            let mut max = (Eval::Loss, None);
            // for now, only greedy gets to use the move cache
            let actions = generate_moves(&state);
            for a in &actions {
                let new = state.apply_sorted(*a);
                // we're repeating states
                if root_path.contains(&new) {
                    continue;
                }
                let eval = self.eval(root_path, new, depth + 1);
                if eval > max.0 {
                    max = (eval, Some(new));
                    // early terminate search
                    if eval == Eval::Win {
                        break;
                    }
                }
            }
            if let Some(new) = max.1 {
                state = new;
            } else {
                // we ran out of unexplored moves
                let eval = Eval::H(h2(&state, &actions));
                root_path.rollback_with_cache(horizon, &mut self.eval_caches[depth], eval);
                return eval;
            }
        }

        root_path.rollback_with_cache(horizon, &mut self.eval_caches[depth], Eval::Win);
        Eval::Win
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
        root_path: &mut RootPath,
        state: &State,
        actions: &Vec<Action>,
    ) -> Option<Action> {
        let mut max = (Eval::Loss, None);
        for a in actions {
            let new = state.apply_sorted(*a);
            // already been to this state in our path
            if root_path.contains(&new) {
                continue;
            }
            // let eval = if let Some(eval) = self.caches[0].get(&new) {
            //     *eval
            // } else {
            let eval = self.eval(root_path, new, 0);
            // };

            // self.caches[0].put(new, eval);
            if eval > max.0 {
                max = (eval, Some(*a));
                // early break for wins
                if eval == Eval::Win {
                    break;
                }
            }
        }

        max.1
    }
}
