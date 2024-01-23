/// # `solitaire-game`
///
/// Game logic for Solitaire
pub mod action;
pub mod deck;
pub mod state;

use action::Action;
use state::State;

#[derive(Debug, Default)]
pub struct Solitaire {
    pub state: State,
}

impl Solitaire {
    pub fn do_move(&mut self, action: Action) {
        self.state = self.state.apply(action);
    }
}
