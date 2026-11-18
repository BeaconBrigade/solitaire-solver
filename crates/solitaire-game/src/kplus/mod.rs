use crate::{
    common::iter_to_arr,
    deck::Deck,
    kplus::{action::Action, state::State},
};

pub mod action;
pub mod state;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct KPlusSolitaire {
    pub state: State,
}

impl KPlusSolitaire {
    pub fn with_deck(deck: Deck) -> Self {
        Self {
            state: State::new(deck),
        }
    }

    /// play move and sort the tableau
    pub fn do_move_and_sort(&mut self, action: Action) {
        self.state = self.state.apply_and_sort(action);
    }

    /// apply a move generated for a sorted version of this state
    pub fn do_sorted_move(&mut self, action: Action) {
        let perm = self.state.sort_piles_perm();
        // apply the action normally because we need to fix the tableau after
        // if you did apply_sorted, the action is applied and the piles
        // would be sorted again which would mean the perm matrix wouldn't
        // necessarily unsort the piles
        self.state = self.state.apply(action);
        self.state.unsort_with_perm(&perm);
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
                talon: ([None; 24], -1, 1, 0),
            },
        };
        game.state.talon.0[0] = Some(last);
        game.state.foundation[3][12] = None;
        game
    }
}
