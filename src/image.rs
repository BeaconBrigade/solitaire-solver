use solitaire_game::deck::{Card, Suit, Value};

pub const BLANK: &str = "file://images/deck/blank.png";
#[allow(unused)]
pub const POKEMON: &str = "file://images/deck/pokemon_back.png";
pub const BACK: &str = "file://images/deck/regular_back.png";

/// Converts card to the path to the corresponding image
pub fn card_to_image(card: Card) -> String {
    let suit = match card.suit {
        Suit::Clubs => "clubs",
        Suit::Spades => "spades",
        Suit::Diamonds => "diamonds",
        Suit::Hearts => "hearts",
    };
    let val = match card.value {
        Value::Ace => "ace",
        Value::Two => "2",
        Value::Three => "3",
        Value::Four => "4",
        Value::Five => "5",
        Value::Six => "6",
        Value::Seven => "7",
        Value::Eight => "8",
        Value::Nine => "9",
        Value::Ten => "10",
        Value::Jack => "jack",
        Value::Queen => "queen",
        Value::King => "king",
    };

    format!("file://images/deck/{val}_of_{suit}.png")
}
