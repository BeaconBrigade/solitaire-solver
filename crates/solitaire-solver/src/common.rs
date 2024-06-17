//! Common functions useful for all solvers

use solitaire_game::{
    action::{Action, Coord, Location},
    deck::Card,
    state::{find_last_idx, State},
};

/// Get all available moves in a specific game state
// TODO: create a iterator version that doesn't need so many bytes of the stack
pub fn available_moves(_state: State) -> [Option<Action>; 3000] {
    let mut moves = [None; 3000];

    moves[0] = Some(Action::TurnStock);

    moves
}

/// Find all moves in a [`State`]
pub struct MoveIterator<'a> {
    state: &'a State,
    last_from: Coord,
    last_to: Coord,
}

impl<'a> Iterator for MoveIterator<'a> {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        let tableau = self.state.tableau;
        let talon = self.state.talon;
        let foundation = self.state.foundation;

        let tableau_iter = match self.last_from {
            Coord {
                location: Location::Tableau(start_pile),
                idx: start_idx,
            } => Some(
                tableau
                    .iter()
                    .enumerate()
                    .skip(start_pile as usize - 1)
                    .flat_map(|(i, p)| {
                        // use same closure for type purposes
                        let c = move |(j, _)| Coord::new(Location::Tableau(i as u8), j as u8 + p.1);
                        // use start_idx to skip the correct amount of cards
                        if i == 0 {
                            p.0[start_idx as usize..].iter().enumerate().map(c)
                        } else {
                            p.0[p.1 as usize..].iter().enumerate().map(c)
                        }
                    }),
            ),
            _ => None,
        };
        let talon_iter = match self.last_from {
            Coord {
                location: Location::Talon,
                idx: talon_card,
            } if talon_card >= 0 => Some(self.last_from),
            Coord {
                location: Location::Foundation(_),
                idx: _,
            } if talon.1 >= 0 => Some(Coord::new(Location::Talon, talon.1 as u8)),
            _ => None,
        };
        let foundation_iter = {
            let c = |(i, p): (usize, &[Option<Card>; 13])| {
                Some(Coord::new(
                    Location::Foundation(i as u8),
                    find_last_idx(p.iter(), |c| c.is_some())? as u8,
                ))
            };
            match self.last_from {
                Coord {
                    location: Location::Foundation(start_pile),
                    idx: _,
                } => foundation
                    .iter()
                    .enumerate()
                    .skip(start_pile as usize - 1)
                    .flat_map(c),
                _ => {
                    let start_pile = 1;
                    foundation
                        .iter()
                        .enumerate()
                        .skip(start_pile as usize - 1)
                        .flat_map(c)
                }
            }
        };

        // iterate through all possible "from" positions
        for from in tableau_iter
            .clone()
            .chain(talon_iter)
            .chain(foundation_iter)
        {
            self.last_from = from;
            // iterate through all "to" positions
            for to in None {
                self.last_to = to;
                let m = Action::Move(from, to);
                if self.state.is_valid_move(m) {
                    return Some(m);
                }
            }
        }

        None
    }
}

/// Orders actions by likelihood to solving the game,
/// helps the algorithm choose the best move. Based
/// off 4.4 - Action Ordering: [Searching Solitaire in Real Time](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
pub fn action_value(state: &State, action: Action) -> u8 {
    // turn stocks are worth zero
    let Action::Move(from, to) = action else {
        return 0;
    };
    // moves to foundation are most valuable
    if let Location::Foundation(_) = to.location {
        // moves from tableau and uncover cards are most valuable
        if let Location::Foundation(i) = from.location {
            if state.tableau[i as usize].1 == from.idx {
                return 5;
            }
        }
        return 4;
    }
    // move from tableau to tableau uncovering a card
    if let (Location::Tableau(i), Location::Tableau(_)) = (from.location, to.location) {
        if state.tableau[i as usize].1 == from.idx {
            return 3;
        }
        // not revealing anything
        return 0;
    }
    // move from talon down to the tableau
    if let (Location::Talon, Location::Tableau(_)) = (from.location, to.location) {
        return 2;
    }
    // move a card down from the tableau
    if let (Location::Foundation(_), Location::Tableau(_)) = (from.location, to.location) {
        return 1;
    }
    // this should be unreachable
    unreachable!("all move cases covered")
}
