use std::cmp;

use crate::{
    common::{combine, find_last_idx, iter_to_arr, Coord, Location},
    deck::{Card, Deck},
    kplus::action::Action,
};

/// Representation of Solitaire using [K+ solitaire](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
#[derive(Debug, Clone, Copy)]
pub struct State {
    // pile.1 is the index where face up cards start
    pub tableau: [([Option<Card>; 19], u8); 7],
    pub foundation: [[Option<Card>; 13]; 4],
    /// talon.1 is the special index which is a potential extra available card
    /// talon.2 is the amount of cards in the talon
    /// talon.3 is how many shifts are needed to fix the array
    pub talon: ([Option<Card>; 24], i8, u8, u8),
}

impl State {
    pub fn new(deck: Deck) -> Self {
        let mut iter = deck.0.into_iter();

        let tableau = [
            (combine(iter_to_arr::<1, _>(&mut iter), [None; 18]), 0),
            (combine(iter_to_arr::<2, _>(&mut iter), [None; 17]), 1),
            (combine(iter_to_arr::<3, _>(&mut iter), [None; 16]), 2),
            (combine(iter_to_arr::<4, _>(&mut iter), [None; 15]), 3),
            (combine(iter_to_arr::<5, _>(&mut iter), [None; 14]), 4),
            (combine(iter_to_arr::<6, _>(&mut iter), [None; 13]), 5),
            (combine(iter_to_arr::<7, _>(&mut iter), [None; 12]), 6),
        ];

        let foundation = [[None; 13], [None; 13], [None; 13], [None; 13]];

        let mut talon = [None; 24];
        for (i, c) in iter.take(24).enumerate() {
            talon[i] = Some(c);
        }
        // start at -1 since no cards start available
        let talon = (talon, -1, 24, 0);

        Self {
            tableau,
            foundation,
            talon,
        }
    }

    pub fn apply(&self, action: Action) -> Self {
        let mut new = *self;

        if !self.is_valid_move(action) {
            return new;
        }
        let from_item = self.get(action.from).unwrap();

        // take item from source
        match action.from.location {
            Location::Foundation(pile) => {
                let pile = pile as usize;
                let idx = find_last_idx(new.foundation[pile].into_iter(), |c| c.is_some()).unwrap();
                new.foundation[pile][idx] = None;
            }
            Location::Tableau(pile) => {
                let pile = pile as usize;
                let last_idx =
                    find_last_idx(new.tableau[pile].0.into_iter(), |c| c.is_some()).unwrap();
                // there are multiple cards to move, this will copy None to the correct places
                new.tableau[pile].0[action.from.idx as usize..=last_idx].fill(None);

                // only reveal face down cards when we're moving the card on top of the foundation stack
                if action.from.idx == self.tableau[pile].1 {
                    // convert to i8 to make sure no overflow errors occur
                    new.tableau[pile].1 = cmp::max(0, new.tableau[pile].1 as i8 - 1) as u8;
                }
            }
            Location::Talon => {
                // the card below the moving one has to become the special index.
                // if the card moved was not the special index, we should rotate
                // left however many times are required.

                new.talon.0[action.from.idx as usize] = None;
                new.talon.3 += 1;

                if self.talon.1 != action.from.idx as i8 {
                    new.talon.0[action.from.idx as usize..].rotate_left(new.talon.3 as usize);
                    new.talon.3 = 0;
                }
                // can be negative if there's no special index
                new.talon.1 = action.from.idx as i8 - 1;

                new.talon.2 -= 1;
            }
        }

        // add item to destination
        match action.to.location {
            Location::Foundation(pile) => {
                new.foundation[pile as usize][action.to.idx as usize] = Some(from_item);
            }
            Location::Tableau(pile) => {
                let pile = pile as usize;
                // moving from tableau to tableau, we might have to move multiple cards
                if let Location::Tableau(from_col) = action.from.location {
                    let last_idx =
                        find_last_idx(self.tableau[from_col as usize].0.into_iter(), |c| {
                            c.is_some()
                        })
                        .unwrap();
                    let len = last_idx - action.from.idx as usize;
                    let dst = &mut new.tableau[pile].0
                        [action.to.idx as usize..=action.to.idx as usize + len];
                    let src = &self.tableau[from_col as usize].0
                        [action.from.idx as usize..=last_idx as _];
                    dst.copy_from_slice(src);
                } else {
                    new.tableau[pile].0[action.to.idx as usize] = Some(from_item);
                }
            }
            // still can't move into the talon
            Location::Talon => unreachable!(),
        }

        new
    }

    pub fn get(&self, pos: Coord) -> Option<Card> {
        match pos.location {
            Location::Foundation(i) => self.foundation[i as usize][pos.idx as usize],
            Location::Tableau(i) => self.tableau[i as usize].0[pos.idx as usize],
            Location::Talon => self.talon.0[pos.idx as usize],
        }
    }

    /// returns whether a card is reachable in the talon
    pub fn is_reachable_talon(&self, idx: u8) -> bool {
        // every third card is available. also, the last card will be available
        // finally the card at the special index is available.
        if (idx + 1) % 3 == 0 {
            true
        } else if idx == self.talon.2 + self.talon.3 {
            true
        } else if idx as i8 == self.talon.1 {
            true
        } else {
            false
        }
    }

    pub fn is_valid_move(&self, action: Action) -> bool {
        todo!()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Deck::new_shuffled())
    }
}
