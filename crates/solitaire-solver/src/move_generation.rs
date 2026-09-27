//! # move-generation
//!
//! This is where the solver generates the moves to evaluate

use solitaire_game::{
    common::{find_last_idx, Coord, Location},
    deck::{Card, Suit, Value},
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

    // find all possible destination cards
    let (foundation_available_cards, foundation_targets) = foundation_targets(&state.foundation);
    let tableau_targets = tableau_targets(&state.tableau);

    // go through all talon moves
    let mut from = Coord::new(Location::Talon, 0);
    for (i, card) in state.talon.0.iter().enumerate() {
        let Some(card) = card else {
            continue;
        };
        // avoid looking for moves that are already invalid
        if !state.is_reachable_talon(i as u8) {
            continue;
        }

        from.idx = i as u8;
        find_moves(
            &mut moves,
            from,
            card,
            Some(&foundation_targets),
            &tableau_targets,
        );
    }

    // go through all tableau moves
    let mut from = Coord::new(Location::Tableau(0), 0);
    for (p, pile) in state.tableau.iter().enumerate() {
        from.location = Location::Tableau(p as u8);
        for (i, card) in pile.0[pile.1 as usize..].iter().flatten().enumerate() {
            from.idx = pile.1 + i as u8;
            find_moves(
                &mut moves,
                from,
                card,
                Some(&foundation_targets),
                &tableau_targets,
            );
        }
    }

    // go through all foundation moves
    for (p, i) in foundation_available_cards.into_iter().enumerate() {
        let Some(i) = i else {
            continue;
        };
        let from = Coord::new(Location::Foundation(p as u8), i as u8);
        let card = state.foundation[p][i].unwrap();
        find_moves(&mut moves, from, &card, None, &tableau_targets);
    }

    moves
}

fn find_moves(
    moves: &mut Vec<Action>,
    from: Coord,
    card: &Card,
    foundation_targets: Option<&[Option<(Card, Coord)>; 4]>,
    tableau_targets: &[Option<(Card, Coord)>; 7],
) {
    // if the card is an ace, there's only one spot in the foundation it can go
    let mut skip_foundation = false;
    if card.value == Value::Ace {
        if let Some(foundation_targets) = foundation_targets {
            if let Some((_, to)) = foundation_targets
                .iter()
                .flatten()
                .find(|(tgt, _)| tgt.value == Value::Ace)
            {
                moves.push(Action::new(from, *to));
                skip_foundation = true;
            }
        }
    }

    // if the card is a king, we say it has only one move into the tableau:
    // to the first open spot in the pile
    let mut skip_tableau = false;
    if card.value == Value::King {
        if let Some((_, to)) = tableau_targets
            .iter()
            .flatten()
            .find(|(tgt, _)| tgt.value == Value::King)
        {
            moves.push(Action::new(from, *to));
            skip_tableau = true;
        }
    }

    // check the tableau for moves
    if !skip_tableau {
        for (target_card, to) in tableau_targets.iter().flatten() {
            // can't move into your own pile
            if to.location == from.location {
                continue;
            }
            if card == target_card || *card == target_card.colour_pair() {
                moves.push(Action::new(from, *to));
                if from.location == Location::Talon {
                    // we only consider one move from talon to tableau
                    // since they're essentially equivalent (don't
                    // affect existence of solution but could add
                    // at most one extra move to solution length)
                    break;
                }
            }
        }
    }

    if skip_foundation {
        return;
    }
    // foundation targets is passed as an option, so when searching for
    // moves of cards in the foundation, we don't do a search over the
    // foundation.
    let Some(foundation_targets) = foundation_targets else {
        return;
    };
    // check the foundation for moves
    for (target_card, to) in foundation_targets.iter().flatten() {
        if card == target_card {
            moves.push(Action::new(from, *to));
            // can only move to one spot in the foundation - no use
            // checking more piles
            break;
        }
    }
}

/// Returns what card is needed to go on the top of each pile and the
/// associated destination [`Coord`].
///
/// If a pile is empty it has an Ace in that slot. If there's
/// a King in the pile, it returns None (no more cards can go
/// in that pile).
///
/// Each empty pile will have a different required Ace, so
/// this changes the output of generate_moves from the old version.
///
/// Assumes the piles are filled from left to right.
// TODO: turn this to only put the first ace, since the rest are implied.
//       this would match what the old algorithm did
fn foundation_targets(
    foundation: &[[Option<Card>; 13]; 4],
) -> ([Option<usize>; 4], [Option<(Card, Coord)>; 4]) {
    let mut foundation_targets = [None, None, None, None];
    let mut indices = [0, 1, 2, 3];
    let mut available_cards = [None; 4];
    let mut remaining_suits = [Suit::Hearts, Suit::Spades, Suit::Diamonds, Suit::Clubs];
    for (i, pile) in foundation.iter().enumerate() {
        let last_idx = find_last_idx(pile.iter(), |c| c.is_some());
        let target = if let Some(j) = last_idx {
            available_cards[i] = Some(j);
            let last_card = pile[j].unwrap();
            last_card
                .next_card()
                .map(|c| (c, Coord::new(Location::Foundation(i as u8), (j + 1) as u8)))
        } else {
            Some((
                Card::new(remaining_suits[i], Value::Ace),
                Coord::new(Location::Foundation(i as u8), 0),
            ))
        };

        if let Some((t, _)) = target {
            // found suit is in position indices[t.suit] in remaining_suits array
            // found suit is in position t.suit in the indices array
            //
            // swap suit is in position i in remaining_suits array
            // swap suit is in position remaining_suits[i] as usize in indices array
            // we have to swap
            let found_suit_idx = indices[t.suit as usize];
            let swap_suit_idx = remaining_suits[i] as usize;
            if found_suit_idx > i {
                indices.swap(t.suit as usize, swap_suit_idx);
                remaining_suits.swap(found_suit_idx, i);
            }
        }
        foundation_targets[i] = target;
    }

    (available_cards, foundation_targets)
}

/// Returns what card is needed for each pile of the tableau. Since two zero or
/// two cards could go on each pile, if the pile has `Some(card)`, then the
/// other card will be found using [`Card::colour_pair`]. If the pile is empty,
/// a King of Hearts will be in that position, but any king can go there.
fn tableau_targets(tableau: &[([Option<Card>; 19], u8); 7]) -> [Option<(Card, Coord)>; 7] {
    let mut tableau_targets = [None; 7];
    for (i, pile) in tableau.iter().enumerate() {
        let target = if let Some(j) = find_last_idx(pile.0.iter(), |c| c.is_some()) {
            let last_card = pile.0[j].unwrap();
            last_card.prev_card().map(|c| {
                (
                    c.colour_opposite(),
                    Coord::new(Location::Tableau(i as u8), (j + 1) as u8),
                )
            })
        } else {
            Some((
                Card::new(Suit::Hearts, Value::King),
                Coord::new(Location::Tableau(i as u8), 0),
            ))
        };

        tableau_targets[i] = target;
    }

    tableau_targets
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;

    use solitaire_game::common::*;
    use solitaire_game::deck::*;
    use solitaire_game::kplus::{action::*, KPlusSolitaire};

    use crate::move_generation::foundation_targets;

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

    macro_rules! cd {
        ($s:expr) => {
            Some(Card::from_str($s).unwrap())
        };
    }

    macro_rules! ccd {
        ($s:expr, $c:expr) => {
            Some((Card::from_str($s).unwrap(), $c))
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

    #[test]
    fn simple_foundation_targets() {
        let foundation = [[None; 13], [None; 13], [None; 13], [None; 13]];
        let exp_targets = [
            ccd!("Hearts Ace", fd!(0, 0)),
            ccd!("Spades Ace", fd!(1, 0)),
            ccd!("Diamonds Ace", fd!(2, 0)),
            ccd!("Clubs Ace", fd!(3, 0)),
        ];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn no_king_foundation_targets() {
        let foundation = [
            [
                cd!("Hearts Ace"),
                cd!("Hearts Two"),
                cd!("Hearts Three"),
                cd!("Hearts Four"),
                cd!("Hearts Five"),
                cd!("Hearts Six"),
                cd!("Hearts Seven"),
                cd!("Hearts Eight"),
                cd!("Hearts Nine"),
                cd!("Hearts Ten"),
                cd!("Hearts Jack"),
                cd!("Hearts Queen"),
                None,
            ],
            [None; 13],
            [None; 13],
            [None; 13],
        ];
        let exp_targets = [
            ccd!("Hearts King", fd!(0, 12)),
            ccd!("Spades Ace", fd!(1, 0)),
            ccd!("Diamonds Ace", fd!(2, 0)),
            ccd!("Clubs Ace", fd!(3, 0)),
        ];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn no_target() {
        let foundation = [
            [
                cd!("Hearts Ace"),
                cd!("Hearts Two"),
                cd!("Hearts Three"),
                cd!("Hearts Four"),
                cd!("Hearts Five"),
                cd!("Hearts Six"),
                cd!("Hearts Seven"),
                cd!("Hearts Eight"),
                cd!("Hearts Nine"),
                cd!("Hearts Ten"),
                cd!("Hearts Jack"),
                cd!("Hearts Queen"),
                cd!("Hearts King"),
            ],
            [None; 13],
            [None; 13],
            [None; 13],
        ];
        let exp_targets = [
            None,
            ccd!("Spades Ace", fd!(1, 0)),
            ccd!("Diamonds Ace", fd!(2, 0)),
            ccd!("Clubs Ace", fd!(3, 0)),
        ];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn no_king_foundation_targets_last() {
        let foundation = [
            [None; 13],
            [None; 13],
            [None; 13],
            [
                cd!("Clubs Ace"),
                cd!("Clubs Two"),
                cd!("Clubs Three"),
                cd!("Clubs Four"),
                cd!("Clubs Five"),
                cd!("Clubs Six"),
                cd!("Clubs Seven"),
                cd!("Clubs Eight"),
                cd!("Clubs Nine"),
                cd!("Clubs Ten"),
                cd!("Clubs Jack"),
                cd!("Clubs Queen"),
                None,
            ],
        ];
        let exp_targets = [
            ccd!("Hearts Ace", fd!(0, 0)),
            ccd!("Spades Ace", fd!(1, 0)),
            ccd!("Diamonds Ace", fd!(2, 0)),
            ccd!("Clubs King", fd!(3, 12)),
        ];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn no_king_foundation_targets_out_of_order() {
        let foundation = [
            [None; 13],
            [None; 13],
            [
                cd!("Clubs Ace"),
                cd!("Clubs Two"),
                cd!("Clubs Three"),
                cd!("Clubs Four"),
                cd!("Clubs Five"),
                cd!("Clubs Six"),
                cd!("Clubs Seven"),
                cd!("Clubs Eight"),
                cd!("Clubs Nine"),
                cd!("Clubs Ten"),
                cd!("Clubs Jack"),
                cd!("Clubs Queen"),
                None,
            ],
            [None; 13],
        ];
        let exp_targets = [
            ccd!("Hearts Ace", fd!(0, 0)),
            ccd!("Spades Ace", fd!(1, 0)),
            ccd!("Clubs King", fd!(2, 12)),
            ccd!("Diamonds Ace", fd!(3, 0)),
        ];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn targets_all_full() {
        let foundation = [
            [
                cd!("Spades Ace"),
                cd!("Spades Two"),
                cd!("Spades Three"),
                cd!("Spades Four"),
                cd!("Spades Five"),
                cd!("Spades Six"),
                cd!("Spades Seven"),
                cd!("Spades Eight"),
                cd!("Spades Nine"),
                cd!("Spades Ten"),
                cd!("Spades Jack"),
                cd!("Spades Queen"),
                cd!("Spades King"),
            ],
            [
                cd!("Hearts Ace"),
                cd!("Hearts Two"),
                cd!("Hearts Three"),
                cd!("Hearts Four"),
                cd!("Hearts Five"),
                cd!("Hearts Six"),
                cd!("Hearts Seven"),
                cd!("Hearts Eight"),
                cd!("Hearts Nine"),
                cd!("Hearts Ten"),
                cd!("Hearts Jack"),
                cd!("Hearts Queen"),
                cd!("Hearts King"),
            ],
            [
                cd!("Clubs Ace"),
                cd!("Clubs Two"),
                cd!("Clubs Three"),
                cd!("Clubs Four"),
                cd!("Clubs Five"),
                cd!("Clubs Six"),
                cd!("Clubs Seven"),
                cd!("Clubs Eight"),
                cd!("Clubs Nine"),
                cd!("Clubs Ten"),
                cd!("Clubs Jack"),
                cd!("Clubs Queen"),
                cd!("Clubs King"),
            ],
            [
                cd!("Diamonds Ace"),
                cd!("Diamonds Two"),
                cd!("Diamonds Three"),
                cd!("Diamonds Four"),
                cd!("Diamonds Five"),
                cd!("Diamonds Six"),
                cd!("Diamonds Seven"),
                cd!("Diamonds Eight"),
                cd!("Diamonds Nine"),
                cd!("Diamonds Ten"),
                cd!("Diamonds Jack"),
                cd!("Diamonds Queen"),
                cd!("Diamonds King"),
            ],
        ];
        let exp_targets = [None, None, None, None];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets,)
    }

    #[test]
    fn targets_almost_full_out_of_order() {
        let foundation = [
            [
                cd!("Spades Ace"),
                cd!("Spades Two"),
                cd!("Spades Three"),
                cd!("Spades Four"),
                cd!("Spades Five"),
                cd!("Spades Six"),
                cd!("Spades Seven"),
                cd!("Spades Eight"),
                cd!("Spades Nine"),
                cd!("Spades Ten"),
                cd!("Spades Jack"),
                cd!("Spades Queen"),
                cd!("Spades King"),
            ],
            [
                cd!("Hearts Ace"),
                cd!("Hearts Two"),
                cd!("Hearts Three"),
                cd!("Hearts Four"),
                cd!("Hearts Five"),
                cd!("Hearts Six"),
                cd!("Hearts Seven"),
                cd!("Hearts Eight"),
                cd!("Hearts Nine"),
                cd!("Hearts Ten"),
                cd!("Hearts Jack"),
                cd!("Hearts Queen"),
                None,
            ],
            [
                cd!("Clubs Ace"),
                cd!("Clubs Two"),
                cd!("Clubs Three"),
                cd!("Clubs Four"),
                cd!("Clubs Five"),
                cd!("Clubs Six"),
                cd!("Clubs Seven"),
                cd!("Clubs Eight"),
                cd!("Clubs Nine"),
                cd!("Clubs Ten"),
                cd!("Clubs Jack"),
                cd!("Clubs Queen"),
                cd!("Clubs King"),
            ],
            [
                cd!("Diamonds Ace"),
                cd!("Diamonds Two"),
                cd!("Diamonds Three"),
                cd!("Diamonds Four"),
                cd!("Diamonds Five"),
                cd!("Diamonds Six"),
                cd!("Diamonds Seven"),
                cd!("Diamonds Eight"),
                cd!("Diamonds Nine"),
                cd!("Diamonds Ten"),
                cd!("Diamonds Jack"),
                cd!("Diamonds Queen"),
                cd!("Diamonds King"),
            ],
        ];
        let exp_targets = [None, ccd!("Hearts King", fd!(1, 12)), None, None];

        let (_, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets)
    }

    #[test]
    fn targets_middle_numbers_out_of_order() {
        let foundation = [
            [
                cd!("Spades Ace"),
                cd!("Spades Two"),
                cd!("Spades Three"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                cd!("Hearts Ace"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                cd!("Clubs Ace"),
                cd!("Clubs Two"),
                cd!("Clubs Three"),
                cd!("Clubs Four"),
                cd!("Clubs Five"),
                cd!("Clubs Six"),
                cd!("Clubs Seven"),
                cd!("Clubs Eight"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                cd!("Diamonds Ace"),
                cd!("Diamonds Two"),
                cd!("Diamonds Three"),
                cd!("Diamonds Four"),
                cd!("Diamonds Five"),
                cd!("Diamonds Six"),
                cd!("Diamonds Seven"),
                cd!("Diamonds Eight"),
                cd!("Diamonds Nine"),
                cd!("Diamonds Ten"),
                cd!("Diamonds Jack"),
                None,
                None,
            ],
        ];
        let exp_targets = [
            ccd!("Spades Four", fd!(0, 3)),
            ccd!("Hearts Two", fd!(1, 1)),
            ccd!("Clubs Nine", fd!(2, 8)),
            ccd!("Diamonds Queen", fd!(3, 11)),
        ];
        let exp_available = [Some(2), Some(0), Some(7), Some(10)];

        let (available, targets) = foundation_targets(&foundation);
        assert_eq!(targets, exp_targets);
        assert_eq!(available, exp_available)
    }
}
