use std::num::NonZeroUsize;

use lru::LruCache;
use solitaire_game::kplus::{state::State, KPlusSolitaire};

use crate::{greedy::greedy, heuristic::h_greed, move_generation::generate_moves, Eval, Solution};

/// Implements nested rollouts using h_greed, n is the level of nesting to use
pub fn nested_rollout_solve(mut game: KPlusSolitaire, n: usize) -> Option<Solution> {
    if game.state.is_win() {
        return Some(Solution { moves: Vec::new() });
    }
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    let mut caches = Vec::new();
    let mut root_path = Vec::new();
    // extra cache for this outer level + n for the nested levels
    for _ in 0..=n {
        caches.push(LruCache::new(NonZeroUsize::new(50_000).unwrap()));
    }
    let mut caches = caches.iter_mut().collect::<Vec<&mut _>>();
    while !game.state.is_win() && !actions.is_empty() {
        let mut max = (Eval::Loss, None);
        root_path.push(game.state);
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
    mut root_path: Vec<State>,
) -> Eval {
    if state.is_win() {
        return Eval::Win(Vec::new());
    } else if root_path.contains(&state) {
        // we're in an infinite loop
        return Eval::Loss;
    }

    // we've already evaluated this position
    if n > 0 && caches[0].get(&state).is_some() {
        return Eval::H(h_greed(&state));
    }

    let mut actions = generate_moves(&state);
    let mut moves = Vec::new();

    while !state.is_win() && !actions.is_empty() {
        root_path.push(state);
        let mut max = (Eval::Loss, None);
        for a in actions {
            let next = state.apply(a);
            let eval = if n == 0 {
                greedy(next, root_path.clone())
            } else {
                nested_rollout(next, &mut caches[1..], n - 1, root_path.clone())
            };

            if max.0 < eval {
                max = (eval, Some(a));
            }
        }
        match max.0 {
            Eval::Win(mut actions) => {
                moves.push(max.1.unwrap());
                moves.append(&mut actions);
                return Eval::Win(moves);
            }
            Eval::Loss => return Eval::H(h_greed(&state)),
            Eval::H(_) => {}
        }
        if n > 0 {
            caches[0].put(state, ());
        }
        state = state.apply(max.1.unwrap());
        moves.push(max.1.unwrap());
        actions = generate_moves(&state);
    }

    Eval::H(h_greed(&state))
}
