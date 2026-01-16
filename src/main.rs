mod image;

use std::{collections::HashMap, env, fs::File, io::Read, str::FromStr};

use macroquad::prelude::*;
use solitaire_game::{
    action::Action,
    deck::{Card, Deck, Suit, Value},
    Solitaire,
};

use crate::image::initialize_card_textures;

fn window_conf() -> Conf {
    Conf {
        window_title: "Solitaire".to_string(),
        window_resizable: false,
        window_width: 1000,
        window_height: 667,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // find textures
    let card_textures = initialize_card_textures().await;
    let blank_texture = load_texture(image::BLANK).await;
    let back_texture = load_texture(image::BACK).await;

    // scaler for card size (convenience for resizing cards and maintaining ratio)
    let params = DrawTextureParams {
        dest_size: Some(CARD_SIZE),
        ..Default::default()
    };
    let background_colour = color_u8!(5, 133, 3, 255);

    // set up deck
    let deck = if let Some(path) = env::args().nth(1) {
        let mut deck_file = File::open(path).unwrap();
        let mut contents = String::new();
        deck_file.read_to_string(&mut contents).unwrap();

        Deck::from_str(&contents).unwrap()
    } else {
        Deck::new_shuffled()
    };
    println!("{}", deck);
    let mut game = Solitaire::with_deck(deck);
    game.do_move(Action::TurnStock);

    // stores dragged card
    let mut dragged: Option<Card> = None;

    // keeps track of information needed for drag and drop
    let mut card_data = initialize_card_data(&game);

    loop {
        clear_background(background_colour);

        let t = card_textures
            .get(&Card::new(Suit::Spades, Value::Ace))
            .unwrap();
        draw_texture_ex(
            t,
            screen_width() / 2.0,
            screen_height() / 2.0,
            WHITE,
            params.clone(),
        );
        let t = card_textures
            .get(&Card::new(Suit::Hearts, Value::King))
            .unwrap();
        draw_texture_ex(
            t,
            screen_width() / 2.0,
            screen_height() / 2.0 + OVERLAP_OFFSET,
            WHITE,
            params.clone(),
        );

        // We'll need to store what is being dragged and the position of everything which isn't
        // dragged. Drop zones will need to be known (and bigger than the clickable areas of each
        // card). Each card needs an associated click zone, and whether it is being dragged (for
        // tableau stack dragging). Need a function for clickable zones when overlapped.
        //
        // Additional data for each card
        //   - is being dragged
        //   - clickable zone (optional value)
        //   - (?) is face up (could be convenient to have but we can already find this using Deck)
        //
        // handle user input (we'll have to keep track of what is being dragged)
        //   - turn stocks
        //   - drag cards
        //     - from talon
        //     - from tableau
        //     - from foundation (which has stacks)
        //   - drop cards
        //
        // drawing
        //   - talon: draw top three cards (if there are any) and upside down card (if any)
        //   - tableau: draw either blank or top card of tableau
        //   - foundation: draw blanks or face down cards and upside cards for each column
        //
        //

        next_frame().await;
    }
}

#[derive(Debug)]
struct CardData {
    dragged: bool,
    clickable_zone: Option<Rect>,
}

const K: f32 = 1.25;
const CARD_SIZE: Vec2 = Vec2 {
    x: K * 50.0,
    y: K * 72.6,
};
const OVERLAP_OFFSET: f32 = 20.0 * K;
const COVERED_CARD_SIZE: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: CARD_SIZE.y - OVERLAP_OFFSET,
};

fn initialize_card_data(game: &Solitaire) -> HashMap<Card, CardData> {
    let mut map = HashMap::with_capacity(52);

    for card in game.state.talon.0.iter().flatten() {}

    map
}
