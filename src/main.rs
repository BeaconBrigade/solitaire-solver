mod game;
mod image;

use std::{env, fs::File, io::Read, str::FromStr};

use macroquad::prelude::*;
use solitaire_game::deck::Deck;

use crate::game::standard::StandardGame;

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
    let mut game = StandardGame::new(deck).await;

    loop {
        game.draw_frame();
        next_frame().await;
    }
}

pub const SCREEN_WIDTH: i32 = 1200;
pub const SCREEN_HEIGHT: i32 = 850;
pub const K: f32 = 1.50;
pub const CARD_SIZE: Vec2 = Vec2 {
    x: K * 50.0,
    y: K * 72.6,
};
pub const OVERLAP_OFFSET: f32 = 20.0 * K;
pub const COVERED_CARD_SIZE: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: OVERLAP_OFFSET,
};

pub const TOP_OFFSET: f32 = CARD_SIZE.y * 0.29;
pub const HORIZONTAL_OFFSET: f32 = CARD_SIZE.x * 1.29;
