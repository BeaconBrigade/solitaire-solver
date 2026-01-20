mod image;

use std::{collections::HashMap, env, fs::File, io::Read, str::FromStr};

use macroquad::prelude::*;
use solitaire_game::{
    action::{Action, Coord, Location},
    deck::{Card, Deck},
    state::find_last_idx,
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
        //   if mouse down
        //     if drag_root is None
        //         find drag_root using clickable_zone
        //     set dragged_root's dragged_zone to cursor location
        //     set each child of dragged_root's dragged_zone relative to dragged_root
        //   else if mouse up but dragged card
        //     perform move action
        //     for each card in dragged_list
        //         set dragged_zone to None
        //         update clickable_zone
        //     set dragged_root to None
        //   clear dragged_list
        //

        // handle user input
        if is_mouse_button_down(MouseButton::Left) {
            match dragged_root {
                Some(r) => {
                    // update dragged_pos for each pulled card
                    println!("holding currently: {r:?}");
                    let (x, y) = mouse_position();
                    let mut m = Vec2 { x, y };
                    card_data.get_mut(&r).unwrap().dragged_pos = Some(m);
                    for child in dragged_list[1..].iter().flatten() {
                        m.y += OVERLAP_OFFSET;
                        card_data.get_mut(child).unwrap().dragged_pos = Some(m);
                    }
                }
                None => {
                    // set up dragged_pos for each dragged card
                    let r = find_cursor_hover(&game, &card_data);
                    dragged_root = r;
                    let coord = r.and_then(|c| game.state.get_coord(c));
                    println!("hovering over: {r:?}, coord: {coord:?}");
                    if let (Some(coord), Some(r)) = (coord, r) {
                        let (x, y) = mouse_position();
                        let mut m = Vec2 { x, y };
                        card_data.get_mut(&r).unwrap().dragged_pos = Some(m);
                        match coord.location {
                            Location::Foundation(_) => {
                                // no cards can be pulled along
                            }
                            Location::Talon => {
                                // no cards can be pulled along
                            }
                            Location::Tableau(i) => {
                                println!("adding to tableau {i}, idx={}", coord.idx);
                                let pile = game.state.tableau[i as usize];
                                // move all cards starting from after dragged_root
                                for child in pile.0[coord.idx as usize + 1..].iter().flatten() {
                                    println!("adding {child:?}");
                                    m.y += OVERLAP_OFFSET;
                                    card_data.get_mut(child).unwrap().dragged_pos = Some(m);
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(root) = dragged_root {
            // mouse is up, but we have a dragged root, so we should make a move
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            for (rect, coord) in DROP_MAP {
                if rect.contains(m) {
                    let mut to_coord = coord;
                    match coord.location {
                        Location::Foundation(i) => {
                            to_coord.idx =
                                find_last_idx(game.state.foundation[i as usize].iter(), |c| {
                                    c.is_some()
                                })
                                .map(|i| i + 1)
                                .unwrap_or(0) as u8;
                        }
                        Location::Tableau(i) => {
                            to_coord.idx =
                                find_last_idx(game.state.tableau[i as usize].0.into_iter(), |c| {
                                    c.is_some()
                                })
                                .map(|i| (i + 1) as u8)
                                .unwrap_or(0) as u8;
                        }
                        _ => unreachable!(),
                    }
                    let from_coord = game.state.get_coord(root).unwrap();
                    game.do_move(Action::Move(from_coord, to_coord));

                    for card in dragged_list.iter().flatten() {
                        let d = card_data.get_mut(card).unwrap();
                        d.dragged_pos = None;
                        d.clickable_zone = coord_to_clickable(&game, coord);
                    }
                    break;
                }
            }

            dragged_root = None;
        } else {
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            for (rect, _) in DROP_MAP {
                if rect.contains(m) {
                    // want to make grabby cursor
                    break;
                }
            }
        }
        clear_list(&mut dragged_list);

        // drawing
        //   for each card
        //       if is being dragged then
        //           put in dragged_list (to draw last on top of everything)
        //       else
        //           draw at normal zone
        //   for each dragged card
        //       draw at dragged_pos
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
        if let Some(top) = talon.0.get(talon.1 as usize).copied().flatten() {
            let mut offset = 0.0;
            match talon.1 {
                -1 | 0 => {
                    // no cards underneath
                }
                1 => {
                    // one card underneath
                    draw_texture_ex(
                        &card_textures[&talon.0[talon.1 as usize - 1].unwrap()],
                        TALON_START.x,
                        TALON_START.y,
                        WHITE,
                        params.clone(),
                    );
                    offset = OVERLAP_OFFSET;
                }
                _ => {
                    // two cards underneath
                    draw_texture_ex(
                        &card_textures[&talon.0[talon.1 as usize - 2].unwrap()],
                        TALON_START.x,
                        TALON_START.y,
                        WHITE,
                        params.clone(),
                    );
                    draw_texture_ex(
                        &card_textures[&talon.0[talon.1 as usize - 2].unwrap()],
                        TALON_START.x + OVERLAP_OFFSET,
                        TALON_START.y,
                        WHITE,
                        params.clone(),
                    );
                    offset = OVERLAP_OFFSET * 2.0;
                }
            }
            let data = card_data[&top];
            if data.dragged_pos.is_some() {
                push_first(&mut dragged_list, top);
            } else {
                draw_texture_ex(
                    &card_textures[&top],
                    TALON_START.x + offset,
                    TALON_START.y,
                    WHITE,
                    params.clone(),
                );
            }
        } else {
            // no cards
            draw_texture_ex(
                &blank_texture,
                TALON_START.x,
                TALON_START.y,
                WHITE,
                params.clone(),
            );
        }

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

        // draw foundation
        let mut offset = 0.0;
        for pile in game.state.foundation.into_iter() {
            if let Some(i) = find_last_idx(pile.into_iter(), |c| c.is_some()) {
                let card = pile[i].unwrap();
                if card_data[&card].dragged_pos.is_some() {
                    // draw later
                    push_first(&mut dragged_list, card);
                    let under_tex = if i > 0 {
                        &card_textures[&pile[i - 1].unwrap()]
                    } else {
                        &blank_texture
                    };
                    draw_texture_ex(
                        under_tex,
                        FOUNDATION_START.x + offset,
                        FOUNDATION_START.y,
                        WHITE,
                        params.clone(),
                    );
                } else {
                    // card should be drawn normally
                    draw_texture_ex(
                        &card_textures[&card],
                        FOUNDATION_START.x + offset,
                        FOUNDATION_START.y,
                        WHITE,
                        params.clone(),
                    );
                }
            } else {
                draw_texture_ex(
                    &blank_texture,
                    FOUNDATION_START.x + offset,
                    FOUNDATION_START.y,
                    WHITE,
                    params.clone(),
                );
            }
            offset += HORIZONTAL_OFFSET;
        }

        // draw tableau
        let mut x_offset = 0.0;
        for pile in game.state.tableau {
            let max_idx = find_last_idx(pile.0.into_iter(), |c| c.is_some());
            let Some(max_idx) = max_idx else {
                // no cards in pile
                draw_texture_ex(
                    &blank_texture,
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y,
                    WHITE,
                    params.clone(),
                );
                continue;
            };
            let mut y_offset = 0.0;
            // draw turned down cards
            for _ in pile.0[0..pile.1 as usize].iter().flatten() {
                draw_texture_ex(
                    &back_texture,
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y + y_offset,
                    WHITE,
                    params.clone(),
                );
                y_offset += OVERLAP_OFFSET;
            }
            // draw face up cards
            for (i, c) in pile.0[pile.1 as usize..=max_idx]
                .iter()
                .flatten()
                .enumerate()
            {
                // every card below the dragged one must also be dragged
                if card_data[c].dragged_pos.is_some() {
                    // println!("moving into dragged_list start={:?}, end={:?}: {:?}", pile.1 as usize + i, max_idx, &pile.0[pile.1 as usize + i..=max_idx]);
                    dragged_list[0..=max_idx - i - pile.1 as usize]
                        .copy_from_slice(&pile.0[pile.1 as usize + i..=max_idx]);
                    break;
                }
                draw_texture_ex(
                    &card_textures[c],
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y + y_offset,
                    WHITE,
                    params.clone(),
                );
            }

            x_offset += HORIZONTAL_OFFSET;
        }

        // draw dragged cards
        for c in dragged_list.iter().flatten() {
            println!("drawing dragged cards: {dragged_list:?}");
            let d = card_data[c];
            draw_texture_ex(
                &card_textures[c],
                d.dragged_pos.unwrap().x,
                d.dragged_pos.unwrap().y,
                WHITE,
                params.clone(),
            );
        }

        next_frame().await;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CardData {
    pub dragged_pos: Option<Vec2>,
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
const HORIZONTAL_OFFSET: f32 = CARD_SIZE.x * 1.29;
const FOUNDATION_START: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: TOP_OFFSET,
};
const TABLEAU_START: Vec2 = Vec2 {
    x: CARD_SIZE.x,
    y: CARD_SIZE.y * 2.5,
};
// two card lengths from the right
const STOCK_START: Vec2 = Vec2 {
    x: SCREEN_WIDTH as f32 - CARD_SIZE.x * 2.0,
    y: TOP_OFFSET,
};
const TALON_START: Vec2 = Vec2 {
    x: STOCK_START.x - OVERLAP_OFFSET * 3.0 - HORIZONTAL_OFFSET,
    y: TOP_OFFSET,
};

const DROP_MAP: [(Rect, Coord); 11] = [
    // foundations
    (
        Rect {
            x: FOUNDATION_START.x,
            y: FOUNDATION_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y,
        },
        Coord {
            location: Location::Foundation(0),
            idx: 0,
        },
    ),
    (
        Rect {
            x: FOUNDATION_START.x + HORIZONTAL_OFFSET,
            y: FOUNDATION_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y,
        },
        Coord {
            location: Location::Foundation(1),
            idx: 0,
        },
    ),
    (
        Rect {
            x: FOUNDATION_START.x + HORIZONTAL_OFFSET * 2.0,
            y: FOUNDATION_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y,
        },
        Coord {
            location: Location::Foundation(2),
            idx: 0,
        },
    ),
    (
        Rect {
            x: FOUNDATION_START.x + HORIZONTAL_OFFSET * 3.0,
            y: FOUNDATION_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y,
        },
        Coord {
            location: Location::Foundation(3),
            idx: 0,
        },
    ),
    // tableau
    (
        Rect {
            x: TABLEAU_START.x,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(0),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(1),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET * 2.0,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(2),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET * 3.0,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(3),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET * 4.0,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(4),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET * 5.0,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(5),
            idx: 0,
        },
    ),
    (
        Rect {
            x: TABLEAU_START.x + HORIZONTAL_OFFSET * 6.0,
            y: TABLEAU_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y + OVERLAP_OFFSET * 18.0,
        },
        Coord {
            location: Location::Tableau(6),
            idx: 0,
        },
    ),
];

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
                dragged_pos: None,
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
            map.get_mut(card).unwrap().clickable_zone.unwrap().h = CARD_SIZE.y;
        }

        current_zone.x += HORIZONTAL_OFFSET;
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

fn push_first(arr: &mut [Option<Card>; 13], item: Card) {
    for (i, c) in arr.iter().enumerate() {
        if c.is_none() {
            arr[i] = Some(item);
            return;
        }
    }
}

fn clear_list(dragged_list: &mut [Option<Card>; 13]) {
    for c in dragged_list.iter_mut() {
        *c = None;
    }
}

fn find_cursor_hover(game: &Solitaire, card_data: &HashMap<Card, CardData>) -> Option<Card> {
    let state = game.state;
    // search talon
    for c in state.talon.0.iter().flatten() {
        let (x, y) = mouse_position();
        let m = Vec2 { x, y };
        if card_data[c].clickable_zone.is_some_and(|z| z.contains(m)) {
            return Some(*c);
        }
    }
    // search foundation
    for pile in state.foundation.iter() {
        for c in pile.iter().flatten() {
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            if card_data[c].clickable_zone.is_some_and(|z| z.contains(m)) {
                return Some(*c);
            }
        }
    }
    // search tableau
    for pile in state.tableau.iter() {
        for c in pile.0.iter().flatten() {
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            if card_data[c].clickable_zone.is_some_and(|z| z.contains(m)) {
                return Some(*c);
            }
        }
    }

    None
}

/// converts a coordinate to a card's clickable zone (if it has one)
fn coord_to_clickable(game: &Solitaire, coord: Coord) -> Option<Rect> {
    todo!()
}
