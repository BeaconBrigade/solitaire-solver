use crate::{
    action::{Action, Coord, Location},
    deck::{Card, Deck},
};

/// Representation of solitary using [K+ solitaire](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
#[derive(Debug, Clone, Copy)]
pub struct State {
    /// There are seven slots in the tableau. A specific slot
    /// can hold up to 19 cards because on the far right slot
    /// six cards are face down (6), and King can be on top(1), and then
    /// the cards Queen through Ace (12). This totals to 19
    pub tableau: [[Option<Card>; 19]; 7],
    /// There are four foundation slots each with up to 13 cards
    pub foundation: [[Option<Card>; 13]; 4],
    /// The deck and talon have at most 24 cards. The second
    /// integer represents where face up cards start.
    pub talon: ([Option<Card>; 24], i8),
}

impl Default for State {
    fn default() -> Self {
        let deck = Deck::new_shuffled();
        let mut iter = deck.0.into_iter();

        let tableau = [
            combine(iter_to_arr::<1, _>(&mut iter), [None; 18]),
            combine(iter_to_arr::<2, _>(&mut iter), [None; 17]),
            combine(iter_to_arr::<3, _>(&mut iter), [None; 16]),
            combine(iter_to_arr::<4, _>(&mut iter), [None; 15]),
            combine(iter_to_arr::<5, _>(&mut iter), [None; 14]),
            combine(iter_to_arr::<6, _>(&mut iter), [None; 13]),
            combine(iter_to_arr::<7, _>(&mut iter), [None; 12]),
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
}

impl State {
    pub fn apply(&self, action: Action) -> Self {
        let mut new = *self;
        match action {
            Action::TurnStock => new.talon.1 += 1,
            Action::ClearTalon => new.talon.1 = 0,
            Action::Move(from, to) => {
                // do nothing
                if from == to {
                    return new;
                }
                // card can't move through the talon
                if from.location == Location::Talon && to.location == Location::Talon {
                    return new;
                }

                // make sure from card exists but the to location doesn't
                let Some(from_item) = self.get(from) else {
                    return new;
                };
                let None = self.get(to) else { return new };
            }
        }

        new
    }

    pub fn get(&self, pos: Coord) -> Option<Card> {
        match pos.location {
            Location::Foundation(i) => self.foundation[i as usize][pos.idx as usize],
            Location::Tableau(i) => self.tableau[i as usize][pos.idx as usize],
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
            Location::Tableau(i) => self.tableau[i as usize][pos.idx as usize] = val,
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
