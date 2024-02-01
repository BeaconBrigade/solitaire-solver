use std::cmp;

use crate::{
    action::{Action, Coord, Location},
    deck::{Card, Deck, Value},
};

/// Representation of solitary using [K+ solitaire](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
#[derive(Debug, Clone, Copy)]
pub struct State {
    /// There are seven slots in the tableau. A specific slot
    /// can hold up to 19 cards because on the far right slot
    /// six cards are face down (6), and King can be on top(1), and then
    /// the cards Queen through Ace (12). This totals to 19.
    /// The second integer for each pile indicates where face up cards start.
    /// Every card with a higher or equal index will be face up.
    pub tableau: [([Option<Card>; 19], u8); 7],
    /// There are four foundation slots each with up to 13 cards
    pub foundation: [[Option<Card>; 13]; 4],
    /// The deck and talon have at most 24 cards. The second
    /// integer represents where face up cards start.
    pub talon: ([Option<Card>; 24], i8),
}

impl Default for State {
    fn default() -> Self {
        Self::new(Deck::new_shuffled())
    }
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
        let talon = (talon, -1);

        Self {
            tableau,
            foundation,
            talon,
        }
    }

    pub fn apply(&self, action: Action) -> Self {
        let mut new = *self;
        match action {
            Action::TurnStock => {
                println!("turning stock");
                // make sure the addition doesn't go past 23
                let next = self.talon.1 + 1;
                new.talon.1 = if next > 23 { -1 } else { next }
            }
            Action::Move(from, to) => {
                println!("making a move");
                // do nothing
                if from == to {
                    println!("from is to");
                    return new;
                }
                // card can't move to the talon
                if to.location == Location::Talon {
                    println!("trying to move through talon");
                    return new;
                }
                // can't move within the same column in both the talon or the tableau
                if from.location == to.location {
                    println!("trying to move to same column");
                    return new;
                }

                // make sure from card exists but the to location doesn't
                let Some(from_item) = self.get(from) else {
                    println!("no item found at location");
                    return new;
                };
                let None = self.get(to) else {
                    println!("card already at new");
                    return new;
                };
                // get the card from will have to move to
                let placement_item = if to.idx > 0 {
                    let above = Coord::new(to.location, to.idx - 1);
                    self.get(above)
                } else {
                    None
                };

                // ensure move is valid
                match to.location {
                    Location::Foundation(_) => match placement_item {
                        Some(up) => {
                            if up.suit != from_item.suit {
                                println!("tried to move to the foundation with unmatched suits");
                                return new;
                            }
                            if up.value as u8 != from_item.value as u8 - 1 {
                                println!("tried to move to foundation with unordered numbers");
                                return new;
                            }
                        }
                        None => {
                            if from_item.value != Value::Ace {
                                println!("tried to move non-ace to base of foundation");
                                return new;
                            }
                        }
                    },
                    Location::Tableau(_) => {
                        match placement_item {
                            Some(up) => {
                                if up.has_same_colour(&from_item) {
                                    println!("wtf: {up:?} - {from_item:?}");
                                    println!("tried to move card in tableau to same colour");
                                    return new;
                                }
                                if up.value as u8 != from_item.value as u8 + 1 {
                                    println!("wth: up({up:?}) from({from_item:?})",);
                                    println!("tried to move card in tableau to card not one level higher");
                                    return new;
                                }
                            }
                            None => {
                                if from_item.value != Value::King {
                                    println!("tried to move non-king to empty tableau stack");
                                    return new;
                                }
                            }
                        }
                    }
                    Location::Talon => unreachable!(),
                }

                // remove item from source
                match from.location {
                    Location::Foundation(i) => {
                        println!("removing source from foundation");
                        let idx =
                            find_last_idx(new.foundation[i as usize].into_iter(), |c| c.is_some())
                                .unwrap();
                        new.foundation[i as usize][idx] = None;
                    }
                    Location::Tableau(i) => {
                        // ensure from card is face up
                        if self.tableau[i as usize].1 > from.idx {
                            println!("trying to move card that's face down");
                            return new;
                        }
                        println!("removing source from tableau");
                        let last_idx =
                            find_last_idx(new.tableau[i as usize].0.into_iter(), |c| c.is_some())
                                .unwrap();
                        // there are multiple cards to move, this will copy None to the correct places
                        new.tableau[i as usize].0[from.idx as usize..=last_idx].fill(None);

                        // only reveal face down cards when we're moving the card on top of the foundation stack
                        if from.idx == self.tableau[i as usize].1 {
                            // convert to i8 to make sure no overflow errors occur
                            new.tableau[i as usize].1 =
                                cmp::max(0, new.tableau[i as usize].1 as i8 - 1) as u8;
                        }
                    }
                    Location::Talon => {
                        println!("taking from talon");
                        // clear the slot
                        new.talon.0[from.idx as usize] = None;
                        // rotate to move the None value to the right end of the array
                        new.talon.0[from.idx as usize..].rotate_left(1);
                        new.talon.1 -= 1;
                    }
                }

                // add item to dest
                match to.location {
                    Location::Foundation(i) => {
                        println!("moving source to foundation");
                        new.foundation[i as usize][to.idx as usize] = Some(from_item);
                    }
                    Location::Tableau(i) => {
                        println!("moving source to tableau ({i})");
                        // moving from tableau to tableau, we might have to move multiple cards
                        if let Location::Tableau(from_col) = from.location {
                            let last_idx =
                                find_last_idx(self.tableau[from_col as usize].0.into_iter(), |c| {
                                    c.is_some()
                                })
                                .unwrap();
                            let len = last_idx - from.idx as usize;
                            let dst = &mut new.tableau[i as usize].0
                                [to.idx as usize..=to.idx as usize + len];
                            let src = &self.tableau[from_col as usize].0
                                [from.idx as usize..=last_idx as _];
                            dst.copy_from_slice(src);
                        } else {
                            new.tableau[i as usize].0[to.idx as usize] = Some(from_item);
                        }
                    }
                    Location::Talon => unreachable!(),
                }
            }
        }
        new
    }

    pub fn get(&self, pos: Coord) -> Option<Card> {
        match pos.location {
            Location::Foundation(i) => self.foundation[i as usize][pos.idx as usize],
            Location::Tableau(i) => self.tableau[i as usize].0[pos.idx as usize],
            Location::Talon => {
                // ensure card is reachable
                // if the position idx is greater than the talon cut off, the card should be hidden
                // at the current time.
                if pos.idx as i8 != self.talon.1 {
                    return None;
                }
                self.talon.0[pos.idx as usize]
            }
        }
    }

    pub fn set(mut self, pos: Coord, val: Option<Card>) -> Self {
        match pos.location {
            Location::Foundation(i) => self.foundation[i as usize][pos.idx as usize] = val,
            Location::Tableau(i) => self.tableau[i as usize].0[pos.idx as usize] = val,
            Location::Talon => {
                if pos.idx as i8 != self.talon.1 {
                    return self;
                }
                self.talon.0[pos.idx as usize] = val
            }
        };
        self
    }
}

fn iter_to_arr<const N: usize, T: Copy>(iter: &mut impl Iterator<Item = T>) -> [Option<T>; N] {
    let mut a = [None; N];
    a.iter_mut()
        .take(N)
        .zip(iter)
        .for_each(|(a, i)| *a = Some(i));

    a
}

fn combine<const N: usize, const M: usize, const L: usize, T: Default + Copy>(
    n: [T; N],
    m: [T; M],
) -> [T; L] {
    debug_assert_eq!(N + M, L);
    let mut l = [T::default(); L];

    l[..N].copy_from_slice(&n[..N]);
    l[N..(M + N)].copy_from_slice(&m[..M]);

    l
}

/// Find last occurence of item in an iterator, returning its index; breaks on first false
pub fn find_last_idx<T>(
    iter: impl Iterator<Item = T>,
    mut pred: impl FnMut(&T) -> bool,
) -> Option<usize> {
    let mut idx = None;

    for (i, item) in iter.enumerate() {
        if pred(&item) {
            idx = Some(i);
        } else {
            break;
        }
    }

    idx
}

/// Find last occurence of item in an iterator returning said item, breaks on first false
pub fn find_last<T>(iter: impl Iterator<Item = T>, mut pred: impl FnMut(&T) -> bool) -> Option<T> {
    let mut res = None;

    for item in iter {
        if pred(&item) {
            res = Some(item);
        } else {
            break;
        }
    }

    res
}
