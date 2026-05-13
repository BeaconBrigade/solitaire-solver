use std::cmp;

use crate::{
    common::{combine, find_last_idx, iter_to_arr, Coord, Location},
    deck::{Card, Deck, Value},
    kplus::action::Action,
};

/// Representation of Solitaire using [K+ solitaire](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

                let mut shifted = 0;
                // check if we have a special index (or if we had one and used up all the cards to
                // the left)
                if (self.talon.1 >= 0 || self.talon.3 > 0) && self.talon.1 != action.from.idx as i8
                {
                    // if the special index is after where we shift, we need to
                    // adjust the special index to account for the shift
                    if action.from.idx as i8 > self.talon.1 {
                        shifted = new.talon.3;
                    }
                    // rotate from the old special index to remove blanks
                    new.talon.0[new.talon.1 as usize + 1..].rotate_left(new.talon.3 as usize);
                    new.talon.3 = 0;
                }
                new.talon.3 += 1;

                // can be negative if there's no special index
                new.talon.1 = action.from.idx as i8 - 1 - shifted as i8;

                // one less card in the talon
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
            // do flatten last so we actually count blank spaces in the talon
            Location::Talon => self.talon.0.get(pos.idx as usize).copied().flatten(),
        }
    }

    /// returns whether a card is reachable in the talon
    pub fn is_reachable_talon(&self, idx: u8) -> bool {
        // if after the special then you're available if
        // - multiple of three starting from after special (ignoring blanks)
        // - multiple of three on the flattened list (equivalent to turning stock
        //   all the way around)
        // - or last in the talon
        if self.talon.1 >= 0 && idx as i8 > self.talon.1 {
            return (idx - self.talon.1 as u8 - self.talon.3).is_multiple_of(3)
                || (idx - self.talon.3 + 1).is_multiple_of(3)
                || idx == self.talon.2 + self.talon.3 - 1;
        }
        // every third card is available, the last card
        // will be available and the card at the special index is available
        (idx + 1).is_multiple_of(3)
            || idx as i8 == self.talon.1
            || idx == self.talon.2 + self.talon.3 - 1
    }

    pub fn is_valid_move(&self, action: Action) -> bool {
        // this is going to have to be more rigorous than standard
        // as it is becoming clear that the move verification allowed
        // illegal moves on standard which makes move generation kinda difficult.
        // do nothing
        let from = action.from;
        let to = action.to;
        if from == to {
            return false;
        }
        // card can't move to the talon
        if to.location == Location::Talon {
            return false;
        }
        // can't move within the same column in both the talon or the tableau
        if from.location == to.location {
            return false;
        }

        // make sure from card exists but the to location doesn't
        let Some(from_item) = self.get(from) else {
            return false;
        };
        let None = self.get(to) else {
            return false;
        };
        // make sure we can reach talon card
        if from.location == Location::Talon && !self.is_reachable_talon(from.idx) {
            return false;
        }
        // get the card from will have to move to
        let placement_item = if to.idx > 0 {
            let above = Coord::new(to.location, to.idx - 1);
            self.get(above)
        } else {
            None
        };

        // ensure move is valid
        match to.location {
            Location::Foundation(_) => {
                // ensure we aren't moving multiple cards to the foundation
                if let Location::Tableau(from_col) = from.location {
                    let last_idx =
                        find_last_idx(self.tableau[from_col as usize].0.into_iter(), |c| {
                            c.is_some()
                        })
                        .unwrap();
                    if from.idx < last_idx as u8 {
                        return false;
                    }
                }
                match placement_item {
                    Some(up) => {
                        if up.suit != from_item.suit {
                            return false;
                        }
                        if up.value as u8 != from_item.value as u8 - 1 {
                            return false;
                        }
                    }
                    None => {
                        if from_item.value != Value::Ace {
                            return false;
                        }
                    }
                }
            }
            Location::Tableau(_) => match placement_item {
                Some(up) => {
                    if up.has_same_colour(&from_item) {
                        return false;
                    }
                    if up.value as u8 != from_item.value as u8 + 1 {
                        return false;
                    }
                }
                None => {
                    if from_item.value != Value::King {
                        return false;
                    }
                }
            },
            Location::Talon => unreachable!(),
        }
        if let Location::Tableau(i) = from.location {
            if self.tableau[i as usize].1 > from.idx {
                return false;
            }
        }

        true
    }

    pub fn set(mut self, pos: Coord, val: Option<Card>) -> Self {
        match pos.location {
            Location::Foundation(i) => self.foundation[i as usize][pos.idx as usize] = val,
            Location::Tableau(i) => self.tableau[i as usize].0[pos.idx as usize] = val,
            Location::Talon => self.talon.0[pos.idx as usize] = val,
        };
        self
    }

    pub fn get_coord(&self, card: Card) -> Option<Coord> {
        // search talon
        for (i, c) in self.talon.0.iter().enumerate() {
            if *c == Some(card) {
                return Some(Coord {
                    location: Location::Talon,
                    idx: i as u8,
                });
            }
        }
        // search foundation
        for (p, pile) in self.foundation.iter().enumerate() {
            for (i, c) in pile.iter().flatten().enumerate() {
                if *c == card {
                    return Some(Coord {
                        location: Location::Foundation(p as u8),
                        idx: i as u8,
                    });
                }
            }
        }
        // search tableau
        for (p, pile) in self.tableau.iter().enumerate() {
            for (i, c) in pile.0.iter().flatten().enumerate() {
                if *c == card {
                    return Some(Coord {
                        location: Location::Tableau(p as u8),
                        idx: i as u8,
                    });
                }
            }
        }

        None
    }

    /// Checks if the game is won
    ///
    /// Is pretty lazy and just checks the last card in the foundation is full
    pub fn is_win(&self) -> bool {
        self.foundation[0][12].is_some()
            && self.foundation[1][12].is_some()
            && self.foundation[2][12].is_some()
            && self.foundation[3][12].is_some()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Deck::new_shuffled())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        common::{Coord, Location},
        deck::Deck,
        kplus::{Action, KPlusSolitaire},
    };

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
    fn clear_front() {
        let d = Deck::from_str(
            "Spades Six\nClubs Five\nHearts Four\nHearts Ten\nSpades Four\nDiamonds Eight\nSpades Eight\nClubs Nine\nHearts Three\nClubs King\nSpades Three\nDiamonds Jack\nClubs Six\nClubs Two\nClubs Ace\nHearts Six\nDiamonds King\nHearts Queen\nHearts Eight\nDiamonds Three\nClubs Four\nDiamonds Ten\nHearts Five\nClubs Jack\nSpades Jack\nDiamonds Ace\nSpades King\nHearts Nine\nSpades Two\nSpades Ace\nHearts Ace\nHearts King\nDiamonds Four\nDiamonds Five\nSpades Ten\nHearts Seven\nClubs Three\nClubs Eight\nSpades Queen\nDiamonds Seven\nClubs Ten\nDiamonds Nine\nSpades Seven\nDiamonds Six\nClubs Seven\nDiamonds Queen\nDiamonds Two\nSpades Five\nClubs Queen\nHearts Jack\nSpades Nine\nHearts Two\n"
        ).unwrap();
        let mut game = KPlusSolitaire::with_deck(d);
        game.do_move(a!(ta!(2), fd!(0, 0)));
        game.do_move(a!(tb!(4, 4), fd!(1, 0)));
        game.do_move(a!(tb!(4, 3), fd!(1, 1)));
        game.do_move(a!(ta!(1), fd!(2, 0)));
        game.do_move(a!(ta!(0), fd!(2, 1)));
        game.do_move(a!(ta!(23), fd!(0, 1)));
        // this shouldn't crash things
        game.do_move(a!(ta!(5), tb!(0, 1)));
        // just for no reason
        assert_eq!(game.state.is_win(), false);
    }
}
