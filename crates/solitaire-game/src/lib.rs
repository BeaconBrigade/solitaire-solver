/// # `solitaire-game`
///
/// Game logic for Solitaire
pub mod action;
pub mod deck;
pub mod state;

use action::Action;
use deck::Deck;
use state::State;

#[derive(Debug, Default, Clone, Copy)]
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
}
