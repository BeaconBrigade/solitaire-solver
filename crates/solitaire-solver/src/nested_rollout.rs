use std::{collections::HashMap, num::NonZeroUsize};

use lru::LruCache;
use solitaire_game::kplus::{state::State, KPlusSolitaire};

use crate::{Eval, Solution, greedy::greedy, heuristic::h2, move_generation::generate_moves};

/// Implements nested rollouts using h2, n is the level of nesting to use
pub fn nested_rollout_solve(mut game: KPlusSolitaire, n: usize) -> Option<Solution> {
    if game.state.is_win() {
        return Some(Solution { moves: Vec::new() });
    }
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    let mut caches = Vec::new();
    let mut root_path = HashMap::new();
    // extra cache for this outer level + n for the nested levels
    for _ in 0..=n {
        caches.push(LruCache::new(NonZeroUsize::new(50_000).unwrap()));
    }
    let mut caches = caches.iter_mut().collect::<Vec<&mut _>>();
    while !game.state.is_win() && !actions.is_empty() {
        let mut max = (Eval::Loss, None);
        root_path.insert(game.state, (0, n));
        for a in actions {
            let next = game.state.apply(a);
            // don't revisit nodes
            if caches[0].get(&next).is_some() {
                continue;
            }
            let eval = nested_rollout(next, &mut caches[1..], n, root_path.clone());
            if max.0 < eval {
                max = (eval, Some(a));
            }
        }
        match max.0 {
            Eval::Win(mut actions) => {
                moves.push(max.1.unwrap());
                moves.append(&mut actions);
                return Some(Solution { moves });
            }
            Eval::Loss => return None,
            Eval::H(_) => {}
        }
        game.do_move(max.1.unwrap());
        caches[0].put(game.state, ());
        moves.push(max.1.unwrap());
        actions = generate_moves(&game.state);
    }

    if game.state.is_win() {
        Some(Solution { moves })
    } else {
        None
    }
}

fn nested_rollout(
    mut state: State,
    caches: &mut [&mut LruCache<State, ()>],
    n: usize,
    mut root_path: HashMap<State, (usize, usize)>,
) -> Eval {
    if state.is_win() {
        return Eval::Win(Vec::new());
    } else if root_path.get(&state).copied() == Some((0, n)) {
        // we're in an infinite loop
        return Eval::Loss;
    }

    // we've already evaluated this position
    if n > 0 && caches[0].get(&state).is_some() {
        return Eval::H(h2(&state));
    }

    let mut actions = generate_moves(&state);
    let mut moves = Vec::new();

    while !state.is_win() && !actions.is_empty() {
        root_path.insert(state, (0, n));
        let mut max = (Eval::Loss, None);
        for a in actions {
            let next = state.apply(a);
            let eval = if n == 0 {
                greedy(next, root_path.clone(), &h2)
            } else {
                nested_rollout(next, &mut caches[1..], n - 1, root_path.clone())
            };

            // use the 'or' so if there's at least one move even if it results
            // in a loss, it is stored there
            if max.0 < eval || max.0 == Eval::Loss {
                max = (eval, Some(a));
            }
        }
        match max {
            (Eval::Win(mut actions), a) => {
                moves.push(a.unwrap());
                moves.append(&mut actions);
                return Eval::Win(moves);
            }
            (Eval::Loss, None) => return Eval::H(h2(&state)),
            (Eval::Loss, Some(_)) => {}
            (Eval::H(_), _) => {}
        }
        if n > 0 {
            caches[0].put(state, ());
        }
        state = state.apply(max.1.unwrap());
        moves.push(max.1.unwrap());
        actions = generate_moves(&state);
    }

    Eval::H(h2(&state))
}
