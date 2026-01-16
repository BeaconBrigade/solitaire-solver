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
        window_width: SCREEN_WIDTH,
        window_height: SCREEN_HEIGHT,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // find textures
    let card_textures = initialize_card_textures().await;
    let blank_texture = load_texture(image::BLANK)
        .await
        .expect("couldn't find blank texture");
    let back_texture = load_texture(image::BACK)
        .await
        .expect("couldn't find back texture");

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
    let mut dragged_root: Option<Card> = None;
    let mut dragged_list: [Option<Card>; 13] = [None; 13];

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
        //   - draw dragged cards last so they're on top
        //      - use fixed size list with length 13 of Option<Card>
        //

        // handling user input
        //   if mouse down but no dragged card
        //     add card to dragged_root
        //   else if mouse up but dragged card
        //     perform move action
        //     for each card in dragged_list
        //         set dragged_zone to None
        //         update clickable_zone
        //     set dragged_root to None
        //   clear dragged_list
        //
        //   for each card
        //     if card is dragged_root then
        //         update dragged_zone to cursor location
        //     else if card is child of dragged_root then
        //         update dragged_zone cursor + offset (based of order of being dragged) (guaranteed cards will come up in order)
        //

        // drawing
        //   for each card
        //       if is being dragged then
        //           put in dragged_list (to draw last on top of everything)
        //       else
        //           draw at normal zone
        //   for each dragged card
        //       draw at dragged_zone
        //
        // normal zone is based from whether we're in:
        //   talon, foundation, tableau
        // talon:
        //   draw only top three cards if there are any starting from TALON_START.
        // foundation:
        //   draw only top card or blank if there are none.
        // tableau:
        //   draw each card column by column (similar to how drag zones are done
        //   in initialize_card_data) with some face down and some up.

        // draw talon
        let talon = &game.state.talon;
        let _ = todo!();

        // draw card stock
        if talon.1 as usize == (talon.2 as usize).wrapping_sub(1) {
            draw_texture_ex(
                &blank_texture,
                STOCK_START.x,
                STOCK_START.y,
                WHITE,
                params.clone(),
            );
        } else {
            draw_texture_ex(
                &back_texture,
                STOCK_START.x,
                STOCK_START.y,
                WHITE,
                params.clone(),
            );
        }

        next_frame().await;
    }
}

#[derive(Debug, Default)]
struct CardData {
    pub dragged_zone: Option<Vec2>,
    pub clickable_zone: Option<Rect>,
}

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 667;
const K: f32 = 1.25;
const CARD_SIZE: Vec2 = Vec2 {
    x: K * 50.0,
    y: K * 72.6,
};
const OVERLAP_OFFSET: f32 = 20.0 * K;
const COVERED_CARD_SIZE: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: OVERLAP_OFFSET,
};

const TOP_OFFSET: f32 = CARD_SIZE.y * 0.29;
const FOUNDATION_START: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: TOP_OFFSET,
};
const TABLEAU_START: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: CARD_SIZE.y * 5.0,
};
// two card lengths from the right
const STOCK_START: Vec2 = Vec2 {
    x: SCREEN_WIDTH as f32 - CARD_SIZE.x * 2.0,
    y: TOP_OFFSET,
};
const TALON_START: Vec2 = Vec2 {
    x: STOCK_START.x - CARD_SIZE.x * (1.0 + OVERLAP_OFFSET * 2.0),
    y: TOP_OFFSET,
};

fn initialize_card_data(game: &Solitaire) -> HashMap<Card, CardData> {
    let mut map = HashMap::with_capacity(52);

    // none of the stock cards are visible
    for card in game.state.talon.0.iter().flatten() {
        map.insert(*card, CardData::default());
    }

    // tableau
    let mut current_zone = TABLEAU_START; // where each card is located
    for column in game.state.tableau {
        // face down cards (column.1 returns the first face up index so no +1)
        for card in column.0.iter().take(column.1 as usize) {
            // we know anything before column.1 must be a card
            map.insert(card.unwrap(), CardData::default());
            current_zone.y += OVERLAP_OFFSET;
        }

        // face up cards
        let mut prev = None;
        for card in column.0.iter().skip(column.1 as usize).flatten() {
            prev = Some(card);
            let d = CardData {
                dragged_zone: None,
                clickable_zone: Some(Rect {
                    x: current_zone.x,
                    y: current_zone.y,
                    w: COVERED_CARD_SIZE.x,
                    h: COVERED_CARD_SIZE.y,
                }),
            };

            map.insert(*card, d);
            current_zone.y += OVERLAP_OFFSET;
        }

        // update last card to have full clickable zone
        if let Some(card) = prev {
            map.get_mut(card).unwrap().clickable_zone.unwrap().y = CARD_SIZE.y;
        }

        current_zone.x += CARD_SIZE.x * 1.29;
        current_zone.y = TABLEAU_START.y;
    }

    // foundation will have no cards
    // therefore we have set each card already
    debug_assert!(
        map.len() == 52,
        "initialize card data did not have correct size"
    );

    map
}
