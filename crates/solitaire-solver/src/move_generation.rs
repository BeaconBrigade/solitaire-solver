//! # move-generation
//!
//! This is where the solver generates the moves to evaluate

use solitaire_game::{
    common::{Coord, Location, find_last_idx},
    kplus::{action::Action, state::State},
};

pub fn generate_moves(state: &State) -> Vec<Action> {
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
    for card in state.talon.0.iter() {
        if card.is_none() {
            from.idx += 1;
            continue;
        }
        // check foundation
        for (p, pile) in state.foundation.iter().enumerate() {
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
            if state.is_valid_move(a) {
                moves.push(a);
                // can only move to one column at a time
                break;
            }
        }
        // check tableau
        for (p, pile) in state.tableau.iter().enumerate() {
            let to = Coord::new(
                Location::Tableau(p as u8),
                find_last_idx(pile.0.iter(), |c| c.is_some())
                    .map(|i| i as u8 + 1)
                    .unwrap_or(0),
            );
            let a = Action::new(from, to);
            if state.is_valid_move(a) {
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
    for (p, pile) in state.tableau.iter().enumerate() {
        from.location = Location::Tableau(p as u8);
        from.idx = pile.1;
        for _ in pile.0[pile.1 as usize..].iter().flatten() {
            // check moves into the foundation
            for (p_f, pile_f) in state.foundation.iter().enumerate() {
                let to = Coord::new(
                    Location::Foundation(p_f as u8),
                    find_last_idx(pile_f.iter(), |c| c.is_some())
                        .map(|i| i as u8 + 1)
                        .unwrap_or(0),
                );
                let a = Action::new(from, to);
                if state.is_valid_move(a) {
                    moves.push(a);
                    // can only move to one column at a time
                    break;
                }
            }
            // check moves into the tableau
            for (p_t, pile_t) in state.tableau.iter().enumerate() {
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
                if state.is_valid_move(a) {
                    moves.push(a);
                }
            }
            from.idx += 1;
        }
    }

    // check foundation
    for (p, pile) in state.foundation.iter().enumerate() {
        let Some(idx) = find_last_idx(pile.iter(), |c| c.is_some()) else {
            continue;
        };
        let from = Coord::new(Location::Foundation(p as u8), idx as u8);
        for (p_t, pile_t) in state.tableau.iter().enumerate() {
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
            if state.is_valid_move(a) {
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
        let game = game.state;
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
        let game = game.state;
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

    #[test]
    fn one() {
        let d = Deck::from_str(
            "Spades Three\nDiamonds Ace\nClubs Five\nSpades Jack\nClubs Jack\nClubs Three\nHearts Nine\nHearts Three\nDiamonds Nine\nSpades Four\nClubs Seven\nClubs Eight\nSpades King\nSpades Eight\nHearts Eight\nHearts Queen\nHearts Six\nSpades Queen\nSpades Ace\nSpades Five\nSpades Nine\nDiamonds Two\nHearts Ten\nClubs Two\nClubs King\nHearts King\nClubs Ten\nHearts Five\nDiamonds Five\nDiamonds Seven\nSpades Ten\nHearts Ace\nDiamonds King\nHearts Seven\nClubs Nine\nDiamonds Three\nClubs Queen\nDiamonds Ten\nDiamonds Eight\nSpades Six\nDiamonds Six\nSpades Seven\nClubs Six\nDiamonds Jack\nDiamonds Queen\nHearts Four\nClubs Four\nDiamonds Four\nClubs Ace\nHearts Two\nHearts Jack\nSpades Two\n"
        ).unwrap();
        let game = KPlusSolitaire::with_deck(d);
        let game = game.state;
        let moves = generate_moves(&game);
        let set: HashSet<Action> = HashSet::from_iter(moves.into_iter());
        let required = [
            a!(ta!(17), tb!(1, 2)),
            a!(ta!(20), fd!(0, 0)),
            a!(tb!(3, 3), tb!(6, 7)),
            a!(tb!(4, 4), tb!(5, 6)),
        ]
        .into_iter()
        .collect();

        assert_eq!(set, required);
    }

    #[test]
    fn two() {
        let d = Deck::from_str(
            "Clubs Ten\nSpades Six\nSpades Jack\nSpades Seven\nDiamonds Three\nDiamonds Two\nDiamonds Four\nDiamonds Six\nHearts Two\nHearts Six\nDiamonds Queen\nDiamonds Ten\nClubs Queen\nSpades Queen\nSpades Ace\nSpades King\nSpades Two\nHearts Five\nClubs Two\nSpades Five\nHearts Ten\nHearts Seven\nSpades Three\nSpades Ten\nClubs Three\nClubs Jack\nClubs Four\nHearts Four\nClubs Five\nClubs King\nDiamonds Nine\nDiamonds Seven\nHearts Nine\nClubs Nine\nHearts Eight\nHearts Queen\nHearts Ace\nDiamonds Jack\nClubs Six\nHearts Jack\nHearts King\nClubs Seven\nSpades Eight\nHearts Three\nSpades Four\nDiamonds Ace\nDiamonds Five\nSpades Nine\nDiamonds Eight\nClubs Ace\nDiamonds King\nClubs Eight\n"
        ).unwrap();

        let game = KPlusSolitaire::with_deck(d);
        let game = game.state;
        let moves = generate_moves(&game);
        let set: HashSet<Action> = HashSet::from_iter(moves.into_iter());
        let required = [
            a!(ta!(2), tb!(0, 1)),
            a!(ta!(5), tb!(5, 6)),
            a!(ta!(8), fd!(0, 0)),
            a!(ta!(17), fd!(0, 0)),
            a!(tb!(4, 4), fd!(0, 0)),
            a!(tb!(5, 5), tb!(1, 2)),
            // this is silly
            a!(tb!(4, 4), tb!(2, 3)),
        ]
        .into_iter()
        .collect();

        assert_eq!(set, required);
    }

    #[test]
    fn five() {
        let d = Deck::from_str(
            "Hearts Six\nHearts Three\nDiamonds Four\nDiamonds Six\nHearts Five\nHearts Queen\nSpades Four\nClubs Eight\nSpades Ten\nClubs Nine\nDiamonds Two\nSpades Ace\nSpades Queen\nClubs Queen\nSpades Six\nSpades Seven\nClubs Three\nHearts Eight\nDiamonds Three\nDiamonds Ace\nDiamonds Eight\nHearts Seven\nDiamonds Jack\nHearts King\nDiamonds King\nClubs Ten\nHearts Two\nDiamonds Seven\nClubs Ace\nClubs Four\nSpades Eight\nHearts Four\nHearts Ten\nDiamonds Ten\nSpades Three\nDiamonds Nine\nSpades Two\nClubs Six\nClubs Two\nDiamonds Five\nClubs Five\nSpades Five\nClubs King\nDiamonds Queen\nClubs Seven\nHearts Nine\nClubs Jack\nSpades King\nSpades Jack\nSpades Nine\nHearts Jack\nHearts Ace\n"
        ).unwrap();

        let game = KPlusSolitaire::with_deck(d);
        let game = game.state;
        let moves = generate_moves(&game);
        let set: HashSet<Action> = HashSet::from_iter(moves.into_iter());
        let required = [
            a!(ta!(11), tb!(4, 5)),
            a!(ta!(20), tb!(2, 3)),
            a!(ta!(23), fd!(0, 0)),
            a!(tb!(4, 4), tb!(6, 7)),
            a!(tb!(5, 5), tb!(3, 4)),
        ]
        .into_iter()
        .collect();

        assert_eq!(set, required);
    }
}
