use std::num::NonZeroUsize;

use lru::LruCache;
use solitaire_game::kplus::{KPlusSolitaire, state::State};

use crate::{Solution, heuristic::h_greed, move_generation::generate_moves};

pub fn greedy_solve(mut game: KPlusSolitaire) -> Option<Solution> {
    let mut moves = Vec::new();
    let mut actions = generate_moves(&game.state);
    // every heuristic level needs its own cache
    let mut cache = LruCache::new(NonZeroUsize::new(50_000).unwrap());
    let mut greedy_cache = LruCache::new(NonZeroUsize::new(50_000).unwrap());
    while !game.state.is_win() && actions.len() > 0 {
        let mut max = (isize::MIN, None);
        for a in actions {
            let n = game.state.apply(a);
            // don't revisit nodes
            if let Some(_) = cache.get(&n) {
                continue;
            }
            let h = greedy(n, &mut greedy_cache);
            cache.put(n, h);
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // we've hit a dead end and are just going in circles
        let Some(a) = max.1 else {
            return None;
        };
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

fn greedy(mut state: State, cache: &mut LruCache<State, isize>) -> isize {
    let mut actions = generate_moves(&state);
    while !state.is_win() && actions.len() > 0 {
        let mut max = (isize::MIN, None);
        for a in actions {
            let n = state.apply(a);
            // we've already visited this node, so we're in a loop
            if let Some(_) = cache.get(&n) {
                continue;
            }
            let h = h_greed(&n);
            cache.put(n, h);
            if max.0 < h {
                max = (h, Some(a));
            }
        }
        // every action takes us back somewhere we've been, it's a dead end
        // or we are just researching here which is bad
        let Some(a) = max.1 else {
            return isize::MIN;
        };
        state = state.apply(a);
        actions = generate_moves(&state);
    }

    h_greed(&state)
}
