use std::{array, num::NonZeroUsize};

use lru::LruCache;
use solitaire_game::kplus::{action::Action, state::State};

use crate::{move_generation::generate_moves, Eval, RootPath, Solver};

pub struct MultistageNestedRolloutSolver<const N: usize> {
    eval_caches: [Vec<LruCache<State, Eval>>; N],
    heuristics: [fn(&State, &[Action]) -> isize; N],
    move_cache: LruCache<State, Vec<Action>>,
    max_depth: [usize; N],
}

impl<'a, const N: usize> MultistageNestedRolloutSolver<N> {
    pub fn new(
        capacity: usize,
        max_depth: [usize; N],
        heuristics: [fn(&State, &[Action]) -> isize; N],
    ) -> Self {
        Self {
            eval_caches: array::from_fn(|i| {
                Vec::from_iter(
                    std::iter::repeat_with(|| LruCache::new(NonZeroUsize::new(capacity).unwrap()))
                        .take(max_depth[i] + 1),
                )
            }),
            heuristics,
            move_cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            max_depth,
        }
    }

    pub fn eval(
        &mut self,
        root_path: &mut RootPath,
        mut state: State,
        depth: usize,
        // which heuristic we're on
        h: usize,
    ) -> Eval {
        let horizon = root_path.len();

        // we already have this state cached for this depth and heuristic
        if let Some(eval) = self.eval_caches[h][depth].get(&state).copied() {
            // no more heuristics, use cached value
            if h + 1 == N {
                root_path.rollback_with_cache(horizon, &mut self.eval_caches[h][depth], eval);
                return eval;
            } else {
                // use next heuristic
                let eval = self.eval(root_path, state, 0, h + 1);
                root_path.rollback_with_cache(horizon, &mut self.eval_caches[h][depth], eval);
                return eval;
            }
        }

        while !state.is_win() {
            root_path.insert(state);

            let mut max = (Eval::Loss, None);
            // for now, only greedy gets to use the move cache
            let actions = if depth < self.max_depth[h] {
                &generate_moves(&state)
            } else {
                // It's ok to borrow from move_cache because if we're on the
                // last depth, we're greedy search and therefore, we don't
                // recurse so we iterate through all actions in a row
                // without touching the move_cache. Then take a new set of
                // actions from the move_cache. So no reesizes or dangling
                // pointers happen.
                unsafe {
                    &*(match self.move_cache.get(&state) {
                        Some(a) => a,
                        None => {
                            let actions = generate_moves(&state);
                            self.move_cache.put(state, actions);
                            self.move_cache.get(&state).unwrap()
                        }
                    } as *const Vec<Action>)
                }
            };
            for a in actions {
                let new = state.apply_sorted(*a);
                // we're repeating states
                if root_path.contains(&new) {
                    continue;
                }
                let eval = if depth < self.max_depth[h] {
                    self.eval(root_path, new, depth + 1, h)
                } else {
                    // second access to move_cache should be ok since reads
                    // don't resize (allegedly). It checks the hashmap, and
                    // moves the lru entry to the front of the head
                    // just be switching pointers. No resizing (hopefully)
                    let new_actions = match self.move_cache.get(&new) {
                        Some(a) => a,
                        None => {
                            // don't add anything to the move_cache
                            // cause that could invalidate the `actions`
                            &generate_moves(&new)
                        }
                    };
                    Eval::H(self.heuristics[h](&new, &new_actions))
                };
                if eval > max.0 {
                    max = (eval, Some(new));
                    // early terminate search
                    if eval == Eval::Win {
                        break;
                    }
                }
            }
            // check dead end or for a local min
            if max.1.is_none()
                || (h + 1 < N && max.0 < Eval::H(self.heuristics[h](&state, &actions)))
            {
                // check if we have a local minimum and another heuristic
                // the main difference from normal nested rollouts
                let eval = if h + 1 == N {
                    Eval::H(self.heuristics[h](&state, &actions))
                } else {
                    // remove state off the root path
                    root_path.pop();
                    self.eval(root_path, state, 0, h + 1)
                };
                // we ran out of unexplored moves
                root_path.rollback_with_cache(horizon, &mut self.eval_caches[h][depth], eval);
                return eval;
            }

            // already checked we had a new state earlier
            state = max.1.unwrap();
        }

        root_path.rollback_with_cache(horizon, &mut self.eval_caches[h][depth], Eval::Win);
        Eval::Win
    }
}

impl<const N: usize> Solver for MultistageNestedRolloutSolver<N> {
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
            let eval = self.eval(root_path, new, 0, 0);

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
