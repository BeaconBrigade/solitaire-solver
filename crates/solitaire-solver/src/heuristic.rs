use solitaire_game::{common::Location, deck::Card, kplus::state::State};

/// h0 from Bjarnason 2007 table 1
pub fn h0(state: &State) -> isize {
    let mut h = 0;
    // number 1
    for pile in state.foundation {
        for card in pile.iter().flatten() {
            h += 5 - card.value as isize;
        }
    }

    for (p, (pile, first_up)) in state.tableau.iter().enumerate() {
        for (idx, card) in pile[0..*first_up as usize].iter().enumerate() {
            // up to the first face up card, they all have to exist
            let card = card.unwrap();
            // number 2
            h += card.value as isize - 13;
            // number 4
            let pair = state.get_coord(card.colour_pair()).unwrap();
            if let Location::Tableau(p) = pair.location {
                if pair.idx < state.tableau[p as usize].1 {
                    h -= 5;
                }
            }
            // number 5 and 6
            h += h0_block_score(state, card, p, idx);
        }
    }
    // number 3 is irrelevant here

    h
}

fn h0_block_score(state: &State, card: Card, pile: usize, idx: usize) -> isize {
    let mut h = 0;
    let build_cards = card.build_cards();
    for under in state.tableau[pile].0[..idx].iter().flatten() {
        if under.suit == card.suit && under.value < card.value {
            // number 5
            h -= 5;
        }
        if let Some((first, second)) = build_cards {
            if *under == first || *under == second {
                // number 6
                h -= 10;
            }
        }
    }
    h
}
