use crate::{common::iter_to_arr, deck::Deck, kplus::{action::Action, state::State}};

pub mod state;
pub mod action;

#[derive(Debug, Default, Clone, Copy)]
pub struct KPlusSolitaire {
    pub state: State,
}

impl KPlusSolitaire {
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
                talon: ([None; 24], 0, 1),
            },
        };
        game.state.talon.0[0] = Some(last);
        game.state.foundation[3][12] = None;
        game
    }
}
