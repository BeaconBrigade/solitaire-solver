/// # `solitaire-game`
///
/// Game logic for Solitaire
pub mod action;
pub mod state;

use crate::{common::iter_to_arr, deck::Deck};
use action::Action;
use state::State;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Solitaire {
    pub state: State,
}

impl Solitaire {
    pub fn with_deck(deck: Deck) -> Self {
        Self {
            state: State::new(deck),
        }
    }

    pub fn do_move(&mut self, action: Action) {
        self.state = self.state.apply(action);
    }

    pub fn new_almost_completed() -> Self {
        let ordered = Deck::new_ordered();
        let last = ordered.0[51];
        let mut iter = ordered.0.into_iter();
        let mut game = Self {
            state: State {
                tableau: [([None; 19], 0); 7],
                foundation: [
                    iter_to_arr(&mut iter),
                    iter_to_arr(&mut iter),
                    iter_to_arr(&mut iter),
                    iter_to_arr(&mut iter),
                ],
                talon: ([None; 24], -1, 1),
            },
        };
        game.state.talon.0[0] = Some(last);
        game.state.foundation[3][12] = None;
        game
    }
}
