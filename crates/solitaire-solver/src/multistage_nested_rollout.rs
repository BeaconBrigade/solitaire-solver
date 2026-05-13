use std::{array, collections::HashMap, num::NonZeroUsize};

use lru::LruCache;
use solitaire_game::kplus::{state::State, KPlusSolitaire};

use crate::{Eval, Solution, greedy::greedy, move_generation::generate_moves};

/// Multistage rollout algorithm from Bjarnason
/// H: number of stages
/// n: nest level for each stage
/// heuristics: the heuristics for each stage
pub fn multistage_rollout_solve<const H: usize>(
    mut game: KPlusSolitaire,
    n: &[usize; H],
    heuristics: &[&dyn Fn(&State) -> isize; H],
) -> Option<Solution> {
    if game.state.is_win() {
        return Some(Solution { moves: Vec::new() });
    }
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    let mut caches: [Vec<LruCache<State, (), _>>; H] = array::from_fn(|_| Vec::new());
    let mut caches: Vec<&mut Vec<LruCache<State, (), _>>> = caches.iter_mut().collect();
    let mut root_path = HashMap::new();
    // array of caches for each stage
    for (c, n) in caches.iter_mut().zip(n.iter().copied()) {
        for _ in 0..n {
            c.push(LruCache::new(NonZeroUsize::new(50_000).unwrap()));
        }
    }
    while !game.state.is_win() && !actions.is_empty() {
        let mut max = (Eval::Loss, None);
        root_path.insert(game.state, (0, n[0]));
        for a in actions {
            let next = game.state.apply(a);
            // if caches[0].get(&next).is_some() {
            //     continue;
            // }
            let eval = multistage_nested_rollout(
                next,
                0,
                &mut caches,
                n.to_vec(),
                heuristics,
                root_path.clone(),
            );
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
        // caches[0][n[0]-1].put(game.state, ());
        moves.push(max.1.unwrap());
        actions = generate_moves(&game.state);
    }

    if game.state.is_win() {
        Some(Solution { moves })
    } else {
        None
    }
}

// used for getting state hashes to see if states are repeating
#[allow(unused)]
macro_rules! hash {
    ($x:ident) => {
        {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::hash::DefaultHasher::new();
            $x.hash(&mut hasher);
            hasher.finish()
        }
    };
}

fn multistage_nested_rollout(
    start: State,
    stage: usize,
    caches: &mut [&mut Vec<LruCache<State, ()>>],
    n: Vec<usize>,
    heuristics: &[&dyn Fn(&State) -> isize],
    // has (stage, n) pair
    mut root_path: HashMap<State, (usize, usize)>,
) -> Eval {
    let mut state = start;
    if state.is_win() {
        return Eval::Win(Vec::new());
    } else if root_path.get(&state).copied() == Some((stage, n[0])) {
        // we're in an infinite loop
        return Eval::Loss;
    }

    let mut actions = generate_moves(&state);
    if actions.is_empty() {
        return Eval::H(heuristics[0](&state));
    }
    let mut moves = Vec::new();

    // return heuristic when we're out of levels
    if n[0] == 0 {
        let res = greedy(state, root_path, heuristics[0]);
        return res;
    }

    if n[0] != usize::MAX && caches[0][n[0] - 1].contains(&state) {
        // if this is the last heuristic
        if n.len() == 1 {
            return Eval::H(heuristics[0](&state));
        } else {
            return multistage_nested_rollout(
                state,
                stage + 1,
                &mut caches[1..],
                n[1..].to_vec(),
                &heuristics[1..],
                root_path,
            );
        }
    }

    let result = loop {
        if !state.is_win() && actions.is_empty() {
            break Eval::H(heuristics[0](&state));
        }
        root_path.insert(state, (stage, n[0]));
        let mut max = (Eval::Loss, None);
        for a in actions {
            let next = state.apply(a);
            let mut q = n.clone();
            q[0] -= 1;
            let eval =
                multistage_nested_rollout(next, stage, caches, q, heuristics, root_path.clone());
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
                break Eval::Win(moves);
            }
            // doesn't matter if we have no moves left or not
            (Eval::Loss, _) => {
                // last heuristic
                if n.len() == 1 {
                    break Eval::H(heuristics[0](&state));
                } else {
                    break multistage_nested_rollout(
                        state,
                        stage + 1,
                        &mut caches[1..],
                        n[1..].to_vec(),
                        &heuristics[1..],
                        root_path,
                    );
                }
            }
            // same as before, this is for local minimums
            (Eval::H(h), _) if n.len() > 1 && h < heuristics[0](&state) => {
                break multistage_nested_rollout(
                    state,
                    stage + 1,
                    &mut caches[1..],
                    n[1..].to_vec(),
                    &heuristics[1..],
                    root_path,
                );
            }
            _ => {}
        }

        state = state.apply(max.1.unwrap());
        moves.push(max.1.unwrap());
        actions = generate_moves(&state);
    };

    caches[0][n[0] - 1].put(start, ());

    result
}

