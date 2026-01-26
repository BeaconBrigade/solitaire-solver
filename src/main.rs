mod game;
mod image;

use std::{cell::RefCell, fs::{self, File}, io::Read, rc::Rc, str::FromStr};

use futures::executor;
use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets::Window},
};
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
    let mut mode = Mode::Menu;

    // abuse of typesystem
    let mut already_playing = Rc::new(RefCell::new(Mode::Menu));
    // ui stuff
    let mut selected_source = 0;
    const SOURCE_OPTIONS: &[&str; 2] = &["Random", "File"];
    let mut deck_path = "decks/".to_string();
    let mut error_message: Option<String> = None;
    let mut next_mode = None;
    let mut save_path = "decks/".to_string();

    loop {
        let background_colour = color_u8!(5, 133, 3, 255);
        clear_background(background_colour);
        if let Some(next) = next_mode.take() {
            mode = next;
        }

        match &mut mode {
            Mode::Menu => {
                if is_key_pressed(KeyCode::Escape) {
                    // mode = Mode::Standard;
                }
                if error_message.is_some() {
                    root_ui().window(hash!(), WINDOW_START, WINDOW_SIZE, |ui| {
                        ui.label(None, "Error:");
                        ui.separator();
                        ui.label(None, &error_message.clone().unwrap());

                        if ui.button(None, "Continue") {
                            error_message = None;
                        }
                    });
                } else {
                    Window::new(hash!(), WINDOW_START, WINDOW_SIZE)
                        .label("Solitaire")
                        .titlebar(true)
                        .movable(false)
                        .ui(&mut root_ui(), |ui| {
                            ui.label(None, "Choose Solitaire game mode");
                            ui.separator();
                            ui.combo_box(
                                hash!(),
                                "Deck source",
                                SOURCE_OPTIONS,
                                &mut selected_source,
                            );
                            ui.separator();
                            ui.input_text(hash!(), "Deck file path", &mut deck_path);
                            if ui.button(None, "Standard Solitaire") {
                                let already = already_playing.borrow_mut();
                                if matches!(*already, Mode::Standard(_, _)) {
                                    // load bearing drop
                                    drop(already);
                                    next_mode = Some(already_playing.replace(Mode::Menu));
                                } else if selected_source == 0 {
                                    let deck = Deck::new_shuffled();
                                    // oh yeah, async baby
                                    next_mode = Some(Mode::Standard(Box::new(executor::block_on(
                                        StandardGame::new(deck),
                                    )), deck));
                                } else {
                                    // find a file
                                    match read_deck(&deck_path) {
                                        Ok(d) => {
                                            next_mode = Some(Mode::Standard(Box::new(
                                                executor::block_on(StandardGame::new(d)),
                                            ), d));
                                        }
                                        Err(e) => {
                                            error_message = Some(e);
                                        }
                                    }
                                }
                            }
                            let already = already_playing.borrow();
                            if let Mode::Standard(_, deck) = *already {
                                ui.input_text(hash!(), "Save deck path", &mut save_path);
                                if ui.button(None, "Clear previous game") {
                                    // also a load bearing drop
                                    drop(already);
                                    already_playing.replace(Mode::Menu);
                                } else if ui.button(None, "Save deck") {
                                    error_message = save_deck(&save_path, deck).err();
                                }
                            }
                        });
                }
            }
            Mode::Standard(game, _) => {
                if !game.draw_frame_and_keep_playing() {
                    already_playing = Rc::new(RefCell::new(mode));
                    mode = Mode::Menu;
                }
            }
        }

        next_frame().await;
    }
}

fn read_deck(path: &str) -> Result<Deck, String> {
    let mut deck_file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    deck_file.read_to_string(&mut contents).unwrap();

    Deck::from_str(&contents).map_err(|_| "Could not parse deck".to_string())
}

fn save_deck(path: &str, deck: Deck) -> Result<(), String> {
    fs::write(path, deck.to_string()).map_err(|e| e.to_string())?;

    Ok(())
}

enum Mode {
    Menu,
    Standard(Box<StandardGame>, Deck),
}

const WINDOW_START: Vec2 = Vec2 {
    x: HORIZONTAL_OFFSET * 2.0,
    y: TOP_OFFSET,
};
const WINDOW_SIZE: Vec2 = Vec2 {
    x: SCREEN_WIDTH as f32 - WINDOW_START.x * 2.0,
    y: SCREEN_HEIGHT as f32 - WINDOW_START.y * 2.0,
};

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
