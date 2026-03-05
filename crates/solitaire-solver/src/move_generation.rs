//! # move-generation
//!
//! This is where the solver generates the moves to evaluate

use solitaire_game::{
    common::{find_last_idx, Coord, Location},
    kplus::{action::Action, KPlusSolitaire},
};

pub fn generate_moves(game: &KPlusSolitaire) -> Vec<Action> {
    // for each available card in the talon we need to check:
    // - can it move to any column in the foundation (max 1)
    // - can it move to any column in the tableau
    // for each card in the tableau we have to check:
    // - can it go to the foundation (only the top card)
    // - can it go to another tableau pile (all face up cards)
    // for each top card in the foundation
    // - can it go to the top of any tableau pile
    // TODO: implement pruning of stupid moves
    // e.g.: moving an ace from foundation to tableau
    let mut moves = Vec::new();
    let mut from = Coord::new(Location::Talon, 0);
    for card in game.state.talon.0.iter() {
        if card.is_none() {
            from.idx += 1;
            continue;
        }
        // check foundation
        for (p, pile) in game.state.foundation.iter().enumerate() {
            let to = Coord::new(
                Location::Foundation(p as u8),
                find_last_idx(pile.iter(), |c| c.is_some())
                    .map(|i| i as u8 + 1)
                    .unwrap_or(0),
            );
            // the pile is full
            if to.idx > 12 {
                continue;
            }
            let a = Action::new(from, to);
            if game.state.is_valid_move(a) {
                moves.push(a);
                // can only move to one column at a time
                break;
            }
        }
        // check tableau
        for (p, pile) in game.state.tableau.iter().enumerate() {
            let to = Coord::new(
                Location::Tableau(p as u8),
                find_last_idx(pile.0.iter(), |c| c.is_some())
                    .map(|i| i as u8 + 1)
                    .unwrap_or(0),
            );
            let a = Action::new(from, to);
            if game.state.is_valid_move(a) {
                moves.push(a);
                // can only move into one pile unless we're a king and
                // then it doesn't matter
                break;
            }
        }

        from.idx += 1;
    }

    // check tableau
    let mut from = Coord::new(Location::Tableau(0), 0);
    for (p, pile) in game.state.tableau.iter().enumerate() {
        from.location = Location::Tableau(p as u8);
        from.idx = pile.1;
        for _ in pile.0[pile.1 as usize..].iter().flatten() {
            // check moves into the foundation
            for (p_f, pile_f) in game.state.foundation.iter().enumerate() {
                let to = Coord::new(
                    Location::Foundation(p_f as u8),
                    find_last_idx(pile_f.iter(), |c| c.is_some())
                        .map(|i| i as u8 + 1)
                        .unwrap_or(0),
                );
                let a = Action::new(from, to);
                if game.state.is_valid_move(a) {
                    moves.push(a);
                    // can only move to one column at a time
                    break;
                }
            }
            // check moves into the tableau
            for (p_t, pile_t) in game.state.tableau.iter().enumerate() {
                // don't search within our pile
                if p_t == p {
                    continue;
                }
                let to = Coord::new(
                    Location::Tableau(p_t as u8),
                    find_last_idx(pile_t.0.iter(), |c| c.is_some())
                        .map(|i| i as u8 + 1)
                        .unwrap_or(0),
                );
                let a = Action::new(from, to);
                if game.state.is_valid_move(a) {
                    moves.push(a);
                }
            }
            from.idx += 1;
        }
    }

    // check foundation
    for (p, pile) in game.state.foundation.iter().enumerate() {
        let Some(idx) = find_last_idx(pile.iter(), |c| c.is_some()) else {
            continue;
        };
        let from = Coord::new(Location::Foundation(p as u8), idx as u8);
        for (p_t, pile_t) in game.state.tableau.iter().enumerate() {
            let to = Coord::new(
                Location::Tableau(p_t as u8),
                find_last_idx(pile_t.0.iter(), |c| c.is_some())
                    .map(|i| i as u8 + 1)
                    .unwrap_or(0),
            );
            // pile is full
            if to.idx > 12 {
                continue;
            }
            let a = Action::new(from, to);
            if game.state.is_valid_move(a) {
                moves.push(a);
                // if we can move to one pile, we can't move to another
                // or: we are a king and it doesn't matter
                break;
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;

    use solitaire_game::common::*;
    use solitaire_game::deck::*;
    use solitaire_game::kplus::{action::*, KPlusSolitaire};

    use super::generate_moves;

    macro_rules! ta {
        ($i:expr) => {
            Coord::new(Location::Talon, $i as u8)
        };
    }

    macro_rules! tb {
        ($p:expr, $i:expr) => {
            Coord::new(Location::Tableau($p as u8), $i as u8)
        };
    }

    macro_rules! fd {
        ($p:expr, $i:expr) => {
            Coord::new(Location::Foundation($p as u8), $i as u8)
        };
    }

    macro_rules! a {
        ($f:expr, $t:expr) => {
            Action::new($f, $t)
        };
    }

    #[test]
    fn eleven() {
        let d = Deck::from_str(
            "Clubs Ten\nDiamonds Seven\nClubs Three\nDiamonds Eight\nSpades Three\nSpades Ace\nSpades Queen\nClubs Ace\nDiamonds Ace\nClubs Two\nDiamonds Nine\nDiamonds Three\nSpades Seven\nDiamonds Jack\nSpades Nine\nDiamonds Ten\nDiamonds Four\nDiamonds Five\nHearts Ten\nSpades Six\nClubs King\nHearts Ace\nSpades Eight\nHearts Nine\nClubs Nine\nClubs Queen\nSpades Four\nSpades Two\nSpades Jack\nDiamonds Two\nHearts Six\nSpades Five\nSpades King\nClubs Seven\nHearts Jack\nHearts Seven\nClubs Six\nClubs Jack\nHearts Queen\nHearts King\nDiamonds Six\nClubs Eight\nDiamonds Queen\nDiamonds King\nHearts Four\nHearts Five\nClubs Five\nClubs Four\nHearts Two\nHearts Three\nHearts Eight\nSpades Ten\n",
        )
        .unwrap();
        let game = KPlusSolitaire::with_deck(d);
        let moves = generate_moves(&game);
        let set: HashSet<Action> = HashSet::from_iter(moves.into_iter());
        let required = [
            a!(ta!(14), tb!(5, 6)),
            a!(ta!(20), tb!(1, 2)),
            a!(tb!(2, 2), fd!(0, 0)),
        ]
        .into_iter()
        .collect();

        assert_eq!(set, required);
    }

    #[test]
    fn almost_done() {
        let game = KPlusSolitaire::new_almost_completed();
        let moves = generate_moves(&game);
        let set: HashSet<Action> = moves.into_iter().collect();
        let required = [
            a!(fd!(0, 12), tb!(0, 0)),
            a!(fd!(1, 12), tb!(0, 0)),
            a!(fd!(2, 12), tb!(0, 0)),
            a!(ta!(0), tb!(0, 0)),
            a!(ta!(0), fd!(3, 12)),
        ]
        .into_iter()
        .collect();

        assert_eq!(set, required);
    }
}
