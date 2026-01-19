use std::collections::HashMap;

use macroquad::prelude::*;
use solitaire_game::deck::{Card, Deck, Suit, Value};

pub const BLANK: &str = "images/deck/blank.png";
#[allow(unused)]
pub const POKEMON: &str = "images/deck/pokemon_back.png";
pub const BACK: &str = "images/deck/regular_back.png";

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

    format!("images/deck/{val}_of_{suit}.png")
}

pub async fn initialize_card_textures() -> HashMap<Card, Texture2D> {
    let mut cache = HashMap::with_capacity(52);

    for card in Deck::new_ordered().0 {
        let im = load_image(&card_to_image(card)).await.unwrap();
        let tex = Texture2D::from_image(&im);
        cache.insert(card, tex);
    }

    cache
}
