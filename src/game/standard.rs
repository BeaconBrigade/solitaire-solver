//! This is just normal solitaire with the face down cards

use crate::COVERED_CARD_SIZE;
use crate::HORIZONTAL_OFFSET;
use crate::OVERLAP_OFFSET;
use crate::SCREEN_WIDTH;
use crate::TOP_OFFSET;
use std::collections::HashMap;

use macroquad::prelude::*;

use macroquad::ui::root_ui;
use solitaire_game::common::find_last_idx;
use solitaire_game::common::{Coord, Location};
use solitaire_game::standard::action::Action;
use solitaire_game::{
    deck::{Card, Deck},
    standard::Solitaire,
};

use crate::{
    image::{self, initialize_card_textures},
    CARD_SIZE,
};

pub struct StandardGame {
    pub game: Solitaire,

    dragged_root: Option<Card>,
    dragged_list: [Option<Card>; 13],
    card_data: HashMap<Card, CardData>,
    // draw card relative to where the mouse clicked it
    cursor_offset: Vec2,

    params: DrawTextureParams,
    card_textures: HashMap<Card, Texture2D>,
    blank_texture: Texture2D,
    back_texture: Texture2D,
}

impl StandardGame {
    pub async fn new(deck: Deck) -> Self {
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

        let game = Solitaire::with_deck(deck);
        let card_data = initialize_card_data(&game);

        Self {
            game,
            card_textures,
            blank_texture,
            back_texture,
            params,
            dragged_root: None,
            dragged_list: [None; 13],
            cursor_offset: Vec2::ZERO,
            card_data,
        }
    }

    pub fn draw_frame_and_keep_playing(&mut self) -> bool {
        if is_key_pressed(KeyCode::Escape) {
            return false;
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 40.0,
                y: 10.0,
            },
            "Menu",
        ) {
            return false;
        }
        //
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
        let (x, y) = mouse_position();
        let m = Vec2 { x, y };
        // check for talon button clicks
        if self.dragged_root.is_none()
            && is_mouse_button_pressed(MouseButton::Left)
            && TALON_BUTTON.contains(m)
        {
            self.game.do_move(Action::TurnStock);
            update_clickable_of_talon(&self.game, &mut self.card_data);
        } else if is_mouse_button_down(MouseButton::Left) {
            match self.dragged_root {
                Some(r) => {
                    // update dragged_pos for each pulled card
                    let (x, y) = mouse_position();
                    let mut m = Vec2 { x, y };
                    m += self.cursor_offset;
                    self.card_data.get_mut(&r).unwrap().dragged_pos = Some(m);
                    for child in self.dragged_list[1..].iter().flatten() {
                        m.y += OVERLAP_OFFSET;
                        self.card_data.get_mut(child).unwrap().dragged_pos = Some(m);
                    }
                }
                None => {
                    // set up dragged_pos for each dragged card
                    let (offset, r) = find_cursor_hover(&self.game, &self.card_data);
                    self.dragged_root = r;
                    self.cursor_offset = offset;
                    let coord = r.and_then(|c| self.game.state.get_coord(c));
                    if let (Some(coord), Some(r)) = (coord, r) {
                        let (x, y) = mouse_position();
                        let mut m = Vec2 { x, y };
                        m += self.cursor_offset;
                        self.card_data.get_mut(&r).unwrap().dragged_pos = Some(m);
                        // card_data.get_mut(&r).unwrap().clickable_zone = None;
                        match coord.location {
                            Location::Foundation(_) => {
                                // no cards can be pulled along
                            }
                            Location::Talon => {
                                // no cards can be pulled along
                            }
                            Location::Tableau(i) => {
                                let pile = self.game.state.tableau[i as usize];
                                // move all cards starting from after dragged_root
                                for child in pile.0[coord.idx as usize + 1..].iter().flatten() {
                                    m.y += OVERLAP_OFFSET;
                                    self.card_data.get_mut(child).unwrap().dragged_pos = Some(m);
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(root) = self.dragged_root {
            // mouse is up, but we have a dragged root, so we should make a move
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            // top left
            let point = m + self.cursor_offset;
            // drop card from top left corner
            let z = Rect {
                x: point.x,
                y: point.y,
                w: CARD_SIZE.x,
                h: CARD_SIZE.y,
            };
            for (rect, coord) in DROP_MAP {
                if rect.overlaps(&z) {
                    let mut to_coord = coord;
                    to_coord.idx = match coord.location {
                        Location::Foundation(i) => {
                            find_last_idx(self.game.state.foundation[i as usize].iter(), |c| {
                                c.is_some()
                            })
                            .map(|i| i + 1)
                            .unwrap_or(0) as u8
                        }
                        Location::Tableau(i) => {
                            find_last_idx(self.game.state.tableau[i as usize].0.into_iter(), |c| {
                                c.is_some()
                            })
                            .map(|i| i + 1)
                            .unwrap_or(0) as u8
                        }
                        _ => unreachable!(),
                    };
                    let from_coord = self.game.state.get_coord(root).unwrap();
                    self.game.do_move(Action::Move(from_coord, to_coord));
                    break;
                }
            }

            // stop dragging all the cards regardless of if the move
            // was correct
            for card in self.dragged_list.iter().flatten() {
                let d = self.card_data.get_mut(card).unwrap();
                d.dragged_pos = None;
            }

            // update the clickable zones for each card
            update_all_clickable(&self.game, &mut self.card_data);

            self.dragged_root = None;
        }
        clear_list(&mut self.dragged_list);

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
        let talon = &self.game.state.talon;
        if let Some(top) = talon.0.get(talon.1 as usize).copied().flatten() {
            let mut offset = 0.0;
            // draw underneath texture
            draw_texture_ex(
                &self.blank_texture,
                TALON_START.x,
                TALON_START.y,
                WHITE,
                self.params.clone(),
            );
            match talon.1 {
                -1 | 0 => {
                    // no cards underneath
                }
                1 => {
                    // one card underneath
                    draw_texture_ex(
                        &self.card_textures[&talon.0[talon.1 as usize - 1].unwrap()],
                        TALON_START.x,
                        TALON_START.y,
                        WHITE,
                        self.params.clone(),
                    );
                    offset = OVERLAP_OFFSET;
                }
                _ => {
                    // two cards underneath
                    draw_texture_ex(
                        &self.card_textures[&talon.0[talon.1 as usize - 2].unwrap()],
                        TALON_START.x,
                        TALON_START.y,
                        WHITE,
                        self.params.clone(),
                    );
                    draw_texture_ex(
                        &self.card_textures[&talon.0[talon.1 as usize - 1].unwrap()],
                        TALON_START.x + OVERLAP_OFFSET,
                        TALON_START.y,
                        WHITE,
                        self.params.clone(),
                    );
                    offset = OVERLAP_OFFSET * 2.0;
                }
            }
            let data = self.card_data[&top];
            if data.dragged_pos.is_some() {
                push_first(&mut self.dragged_list, top);
            } else {
                draw_texture_ex(
                    &self.card_textures[&top],
                    TALON_START.x + offset,
                    TALON_START.y,
                    WHITE,
                    self.params.clone(),
                );
            }
        } else {
            // no cards
            draw_texture_ex(
                &self.blank_texture,
                TALON_START.x,
                TALON_START.y,
                WHITE,
                self.params.clone(),
            );
        }

        // draw card stock
        if talon.1 as usize == (talon.2 as usize).wrapping_sub(1) {
            draw_texture_ex(
                &self.blank_texture,
                STOCK_START.x,
                STOCK_START.y,
                WHITE,
                self.params.clone(),
            );
        } else {
            draw_texture_ex(
                &self.back_texture,
                STOCK_START.x,
                STOCK_START.y,
                WHITE,
                self.params.clone(),
            );
        }

        // draw foundation
        let mut offset = 0.0;
        for pile in self.game.state.foundation.into_iter() {
            if let Some(i) = find_last_idx(pile.into_iter(), |c| c.is_some()) {
                let card = pile[i].unwrap();
                if self.card_data[&card].dragged_pos.is_some() {
                    // draw later
                    push_first(&mut self.dragged_list, card);
                    let under_tex = if i > 0 {
                        &self.card_textures[&pile[i - 1].unwrap()]
                    } else {
                        &self.blank_texture
                    };
                    draw_texture_ex(
                        under_tex,
                        FOUNDATION_START.x + offset,
                        FOUNDATION_START.y,
                        WHITE,
                        self.params.clone(),
                    );
                } else {
                    // card should be drawn normally
                    draw_texture_ex(
                        &self.card_textures[&card],
                        FOUNDATION_START.x + offset,
                        FOUNDATION_START.y,
                        WHITE,
                        self.params.clone(),
                    );
                }
            } else {
                draw_texture_ex(
                    &self.blank_texture,
                    FOUNDATION_START.x + offset,
                    FOUNDATION_START.y,
                    WHITE,
                    self.params.clone(),
                );
            }
            offset += HORIZONTAL_OFFSET;
        }

        // draw tableau
        let mut x_offset = 0.0;
        for pile in self.game.state.tableau {
            let max_idx = find_last_idx(pile.0.into_iter(), |c| c.is_some());
            let Some(max_idx) = max_idx else {
                // no cards in pile
                draw_texture_ex(
                    &self.blank_texture,
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y,
                    WHITE,
                    self.params.clone(),
                );
                x_offset += HORIZONTAL_OFFSET;
                continue;
            };
            if max_idx == 0 || self.dragged_root == pile.0[0] {
                // draw underneath texture if we're dragging
                draw_texture_ex(
                    &self.blank_texture,
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y,
                    WHITE,
                    self.params.clone(),
                );
            }
            let mut y_offset = 0.0;
            // draw turned down cards
            for _ in pile.0[0..pile.1 as usize].iter().flatten() {
                draw_texture_ex(
                    &self.back_texture,
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y + y_offset,
                    WHITE,
                    self.params.clone(),
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
                if self.card_data[c].dragged_pos.is_some() {
                    self.dragged_list[0..=max_idx - i - pile.1 as usize]
                        .copy_from_slice(&pile.0[pile.1 as usize + i..=max_idx]);
                    break;
                }
                draw_texture_ex(
                    &self.card_textures[c],
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y + y_offset,
                    WHITE,
                    self.params.clone(),
                );
                y_offset += OVERLAP_OFFSET;
            }

            x_offset += HORIZONTAL_OFFSET;
        }

        // draw dragged cards
        for c in self.dragged_list.iter().flatten() {
            let d = self.card_data[c];
            draw_texture_ex(
                &self.card_textures[c],
                d.dragged_pos.unwrap().x,
                d.dragged_pos.unwrap().y,
                WHITE,
                self.params.clone(),
            );
        }

        true
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CardData {
    pub dragged_pos: Option<Vec2>,
    pub clickable_zone: Option<Rect>,
}

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
            let d = map.get_mut(card).unwrap();
            let mut z = d.clickable_zone.unwrap();
            z.h = CARD_SIZE.y;
            d.clickable_zone = Some(z);
        }

        current_zone.x += HORIZONTAL_OFFSET;
        current_zone.y = TABLEAU_START.y;
    }

    // in case of cooked variations which have cards in the foundation to start
    for (p, pile) in game.state.foundation.iter().enumerate() {
        let mut j = None;
        for (i, card) in pile.iter().flatten().enumerate() {
            map.insert(*card, CardData::default());
            j = Some(i);
        }
        if let Some(j) = j {
            map.get_mut(&game.state.foundation[p][j].unwrap())
                .unwrap()
                .clickable_zone = Some(Rect {
                x: FOUNDATION_START.x + HORIZONTAL_OFFSET * p as f32,
                y: FOUNDATION_START.y,
                w: CARD_SIZE.x,
                h: CARD_SIZE.y,
            });
        }
    }

    // foundation will have no cards
    // therefore we have set each card already
    debug_assert!(
        map.len() == 52,
        "initialize card data did not have correct size"
    );

    map
}

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

const TALON_BUTTON: Rect = Rect {
    x: STOCK_START.x,
    y: STOCK_START.y,
    w: CARD_SIZE.x,
    h: CARD_SIZE.y,
};

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

fn find_cursor_hover(
    game: &Solitaire,
    card_data: &HashMap<Card, CardData>,
) -> (Vec2, Option<Card>) {
    let state = game.state;
    // search talon
    for c in state.talon.0.iter().flatten() {
        let (x, y) = mouse_position();
        let m = Vec2 { x, y };
        if let Some(z) = card_data[c].clickable_zone {
            if z.contains(m) {
                return (z.point() - m, Some(*c));
            }
        }
    }
    // search foundation
    for pile in state.foundation.iter() {
        for c in pile.iter().flatten() {
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            if let Some(z) = card_data[c].clickable_zone {
                if z.contains(m) {
                    return (z.point() - m, Some(*c));
                }
            }
        }
    }
    // search tableau
    for pile in state.tableau.iter() {
        for c in pile.0.iter().flatten() {
            let (x, y) = mouse_position();
            let m = Vec2 { x, y };
            if let Some(z) = card_data[c].clickable_zone {
                if z.contains(m) {
                    return (z.point() - m, Some(*c));
                }
            }
        }
    }

    (Vec2::ZERO, None)
}

/// definitive clickable updater
fn update_all_clickable(game: &Solitaire, card_data: &mut HashMap<Card, CardData>) {
    update_clickable_of_talon(game, card_data);

    // update tableau
    let mut current_zone = TABLEAU_START; // where each card is located
    for column in game.state.tableau {
        // face down cards (column.1 returns the first face up index so no +1)
        for card in column.0.iter().take(column.1 as usize) {
            // we know anything before column.1 must be a card
            let d = card_data.get_mut(&card.unwrap()).unwrap();
            d.clickable_zone = None;

            current_zone.y += OVERLAP_OFFSET;
        }

        // face up cards
        let mut prev = None;
        for card in column.0.iter().skip(column.1 as usize).flatten() {
            prev = Some(card);
            let z = Rect {
                x: current_zone.x,
                y: current_zone.y,
                w: COVERED_CARD_SIZE.x,
                h: COVERED_CARD_SIZE.y,
            };
            let d = card_data.get_mut(card).unwrap();
            d.clickable_zone = Some(z);

            current_zone.y += OVERLAP_OFFSET;
        }

        // update last card to have full clickable zone
        if let Some(card) = prev {
            let d = card_data.get_mut(card).unwrap();
            let mut z = d.clickable_zone.unwrap();
            z.h = CARD_SIZE.y;
            d.clickable_zone = Some(z);
        }

        current_zone.x += HORIZONTAL_OFFSET;
        current_zone.y = TABLEAU_START.y;
    }

    // foundation
    for (p, pile) in game.state.foundation.iter().enumerate() {
        let mut j = None;
        for (i, card) in pile.iter().flatten().enumerate() {
            let d = card_data.get_mut(card).unwrap();
            d.clickable_zone = None;
            j = Some(i);
        }
        if let Some(j) = j {
            card_data
                .get_mut(&game.state.foundation[p][j].unwrap())
                .unwrap()
                .clickable_zone = Some(Rect {
                x: FOUNDATION_START.x + HORIZONTAL_OFFSET * p as f32,
                y: FOUNDATION_START.y,
                w: CARD_SIZE.x,
                h: CARD_SIZE.y,
            });
        }
    }
}

fn update_clickable_of_talon(game: &Solitaire, card_data: &mut HashMap<Card, CardData>) {
    let offset = match game.state.talon.1 {
        // nothing clickable in the talon
        -1 => return,
        // will be all the way to the left so no offset
        0 => 0.0,
        // will be one card to the right
        1 => OVERLAP_OFFSET,
        // will be two cards to the right
        _ => OVERLAP_OFFSET * 2.0,
    };
    let talon = game.state.talon;
    for card in talon.0[..talon.1 as usize].iter().flatten() {
        let d = card_data.get_mut(card).unwrap();
        d.clickable_zone = None;
    }
    let card = talon.0[talon.1 as usize];
    if let Some(card) = card {
        let clickable_zone = Rect {
            x: TALON_START.x + offset,
            y: TALON_START.y,
            w: CARD_SIZE.x,
            h: CARD_SIZE.y,
        };
        card_data.get_mut(&card).unwrap().clickable_zone = Some(clickable_zone);
    }
}
