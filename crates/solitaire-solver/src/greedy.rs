use std::{collections::HashMap, num::NonZeroUsize};

use lru::LruCache;
use solitaire_game::kplus::{KPlusSolitaire, action::Action, state::State};

use crate::{
    heuristic::h2,
    move_generation::generate_moves,
    Eval, Solution,
};

pub fn greedy_solve(mut game: KPlusSolitaire) -> Option<Solution> {
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    let mut root_path = HashMap::new();
    // every heuristic level needs its own cache
    let mut cache = LruCache::new(NonZeroUsize::new(50_000).unwrap());
    while !game.state.is_win() && !actions.is_empty() {
        let mut max = (isize::MIN, None);
        root_path.insert(game.state, (0, 0));
        for a in actions {
            let n = game.state.apply(a);
            // don't revisit nodes
            if cache.get(&n).is_some() {
                continue;
            }
            let eval = greedy(n, root_path.clone(), &h2);
            let h = match eval {
                Eval::Loss => continue,
                Eval::Win(mut rest_of_moves) => {
                    moves.push(a);
                    moves.append(&mut rest_of_moves);
                    return Some(Solution { moves });
                }
                Eval::H(h) => h,
            };
            cache.put(n, ());
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // we've hit a dead end and are just going in circles
        let a = max.1?;
        game.do_move(a);
        moves.push(a);
        actions = generate_moves(&game.state);
    }

    if game.state.is_win() {
        Some(Solution { moves })
    } else {
        None
    }
}

pub fn greedy(
    mut state: State,
    mut root_path: HashMap<State, (usize, usize)>,
    heuristic: &dyn Fn(&State, &[Action]) -> isize,
) -> Eval {
    let mut moves = Vec::new();
    let mut actions = generate_moves(&state);
    while !state.is_win() && !actions.is_empty() {
        // loop prevention
        if root_path.contains_key(&state) {
            return Eval::Loss;
        }
        root_path.insert(state, (0, 0));
        let mut max = (isize::MIN, None);
        for a in &actions {
            let n = state.apply(*a);
            // we've already visited this node, so we're in a loop
            if root_path.contains_key(&n) {
                continue;
            }
            let h = heuristic(&n, &actions);
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // every action takes us back somewhere we've been, it's a dead end
        // or we are just researching here which is bad
        let Some(a) = max.1 else {
            return Eval::Loss;
        };
        moves.push(*a);
        state = state.apply(*a);
        actions = generate_moves(&state);
    }
    if state.is_win() {
        Eval::Win(moves)
    } else {
        Eval::H(h2(&state, &generate_moves(&state)))
    }
}
