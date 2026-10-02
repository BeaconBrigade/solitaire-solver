use macroquad::prelude::*;
use macroquad::ui::root_ui;
use solitaire_solver::{move_generation::generate_moves, RootPath, Solver};

use std::{
    cmp::min,
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use solitaire_game::{
    common::{find_last_idx, Location},
    deck::{Card, Deck},
    kplus::{action::Action, KPlusSolitaire},
};

use crate::{
    clear_list,
    image::{self, initialize_card_textures},
    push_first, CARD_SIZE, COVERED_CARD_SIZE, DROP_MAP, FOUNDATION_START, HORIZONTAL_OFFSET,
    OVERLAP_OFFSET, SCREEN_WIDTH, TABLEAU_START,
};

#[derive(Clone)]
pub struct KPlusGame {
    pub game: KPlusSolitaire,

    dragged_root: Option<Card>,
    dragged_list: [Option<Card>; 13],
    card_data: HashMap<Card, CardData>,
    cursor_offset: Vec2,

    params: DrawTextureParams,
    card_textures: HashMap<Card, Texture2D>,
    blank_texture: Texture2D,

    root_path: RootPath,

    /// contains the thread searching for a move and a boolean to indicate if the engine
    /// should continue solving after a move is found.
    solver_move: Option<(bool, Arc<JoinHandle<Option<Action>>>)>,
    solver: Arc<Mutex<Box<dyn Solver + Send>>>,
}

impl KPlusGame {
    pub async fn new(deck: Deck, solver: Box<dyn Solver + Send>) -> Self {
        let card_textures = initialize_card_textures().await;
        let blank_texture = load_texture(image::BLANK)
            .await
            .expect("couldn't find blank texture");

        // scaler for card size (convenience for resizing cards and maintaining ratio)
        let params = DrawTextureParams {
            dest_size: Some(CARD_SIZE),
            ..Default::default()
        };

        let game = KPlusSolitaire::with_deck(deck);
        let card_data = initialize_card_data(&game);

        let mut root_path = RootPath::new();
        root_path.insert(game.state);

        Self {
            game,
            card_textures,
            blank_texture,
            params,
            dragged_root: None,
            dragged_list: [None; 13],
            cursor_offset: Vec2::ZERO,
            card_data,
            root_path,
            solver_move: None,
            solver: Arc::new(Mutex::new(solver)),
        }
    }

    pub fn draw_frame_and_keep_playing(&mut self) -> SolveUserAction {
        if is_key_pressed(KeyCode::Escape) {
            return SolveUserAction::Menu;
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 40.0,
                y: 10.0,
            },
            "Menu",
        ) {
            return SolveUserAction::Menu;
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 100.0,
                y: 10.0,
            },
            "Undo",
        ) {
            if let Some(game) = self.root_path.pop() {
                self.game.state = game;
            }
            update_all_clickable(&self.game, &mut self.card_data);
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 200.0,
                y: 10.0,
            },
            "Restart",
        ) {
            if let Some(game) = self.root_path.first().copied() {
                self.game.state = game;
                self.root_path.clear();
                self.root_path.insert(game);
                self.card_data = initialize_card_data(&self.game);
            }
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 300.0,
                y: 10.0,
            },
            "Solver",
        ) {
            return SolveUserAction::Solve;
        }

        // draw how many moves this has taken (not including undos)
        draw_text(
            &format!("Move {}", self.root_path.len()),
            SCREEN_WIDTH as f32 - 750.0,
            25.0,
            20.0,
            WHITE,
        );

        // update stuff:
        // we don't have a turn stock action anymore so we don't have to
        // worry about a button or turn stock
        if is_mouse_button_down(MouseButton::Left) {
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
                    let prev = self.game;
                    self.game.do_move(Action::new(from_coord, to_coord));
                    if self.game != prev {
                        self.root_path.insert(prev.state);
                    }
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
        self.draw_game();

        SolveUserAction::None
    }

    fn draw_game(&mut self) {
        // draw the talon
        let talon = &self.game.state.talon;
        let mut x_offset = 0.0;
        for (i, card) in talon.0.iter().enumerate() {
            if let Some(c) = card {
                if self.card_data[c].dragged_pos.is_some() {
                    push_first(&mut self.dragged_list, *c);
                } else {
                    // gray out cards which aren't reachable
                    let tint = if self.game.state.is_reachable_talon(i as u8) {
                        WHITE
                    } else {
                        GRAY
                    };
                    draw_texture_ex(
                        &self.card_textures[c],
                        TALON_START.x + x_offset,
                        TALON_START.y,
                        tint,
                        self.params.clone(),
                    );
                }
            }
            x_offset += OVERLAP_OFFSET;
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
            // draw turned down cards as grayed out
            for c in pile.0[0..pile.1 as usize].iter().flatten() {
                draw_texture_ex(
                    &self.card_textures[c],
                    TABLEAU_START.x + x_offset,
                    TABLEAU_START.y + y_offset,
                    GRAY,
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
    }

    /// Mode where the game is viewed and the user watches the solver play
    /// or can take it over if they want.
    pub fn solve_game_and_keep_playing(&mut self) -> SolveUserAction {
        if is_key_pressed(KeyCode::Escape) {
            // fix the zones after the computer messed them all up
            update_all_clickable(&self.game, &mut self.card_data);
            self.solver_move = None;
            return SolveUserAction::Menu;
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 40.0,
                y: 10.0,
            },
            "Menu",
        ) {
            update_all_clickable(&self.game, &mut self.card_data);
            self.solver_move = None;
            return SolveUserAction::Menu;
        }
        // don't undo while the solver is making a move
        // could maybe cause an invalid action to be attempted
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 100.0,
                y: 10.0,
            },
            "Undo",
        ) && self.solver_move.is_none() {
            if let Some(game) = self.root_path.pop() {
                self.game.state = game;
                self.solver_move = None;
            }
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 200.0,
                y: 10.0,
            },
            "Restart",
        ) {
            if let Some(game) = self.root_path.first().copied() {
                self.game.state = game;
                self.root_path.clear();
                self.root_path.insert(game);
                self.card_data = initialize_card_data(&self.game);
                self.solver_move = None;
            }
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 300.0,
                y: 10.0,
            },
            "Start playing",
        ) {
            // make sure the clickable is up to date
            update_all_clickable(&self.game, &mut self.card_data);
            // terminate search
            self.solver_move = None;
            return SolveUserAction::Play;
        }

        let one_move_text = if self.solver_move.is_none() {
            "Generate move"
        } else {
            "Move generating"
        };
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 450.0,
                y: 10.0,
            },
            one_move_text,
        ) && self.solver_move.is_none()
        {
            // trust me I won't double race
            self.game.state.sort_piles();
            // we need to have a way to unsort the piles for display
            let s = self.game.state.clone();
            let mut rp = self.root_path.clone();
            let solver = self.solver.clone();
            // start move generation
            self.solver_move = Some((
                false,
                Arc::new(thread::spawn(move || {
                    solver
                        .lock()
                        .unwrap()
                        .next_move(&mut rp, &s, &generate_moves(&s))
                })),
            ));
        }
        let mut all_move_text = "Play out game";
        // check if the move has finished computing
        if let Some((keep_playing, handle)) = &self.solver_move {
            if handle.is_finished() {
                // we should be the only one holding our handle after the Option::take
                let keep_playing = *keep_playing; // load bearing copy
                let handle = Arc::try_unwrap(self.solver_move.take().unwrap().1).unwrap();
                let res = handle.join();
                if let Ok(Some(a)) = res {
                    self.game.do_move_sorted(a);
                    // TODO: need to handle sorting
                    // a = unsort(a)
                    // self.game.do_move(a);
                    self.root_path.insert(self.game.state);
                }

                // start looking for a new move also, no need to sort the piles because our player
                // sorts the piles.
                if keep_playing && !self.game.state.is_win() {
                    let s = self.game.state.clone();
                    let mut rp = self.root_path.clone();
                    let solver = self.solver.clone();
                    self.solver_move = Some((
                        true,
                        Arc::new(thread::spawn(move || {
                            solver
                                .lock()
                                .unwrap()
                                .next_move(&mut rp, &s, &generate_moves(&s))
                        })),
                    ));
                }
            } else {
                // we're not done
                if *keep_playing {
                    all_move_text = "Stop playing";
                } else {
                    all_move_text = "Continue playing";
                }
            }
        }
        if root_ui().button(
            Vec2 {
                x: SCREEN_WIDTH as f32 - 600.0,
                y: 10.0,
            },
            all_move_text,
        ) {
            // switch between continuing playing or stopping after this move
            if let Some((keep_playing, _)) = &mut self.solver_move {
                *keep_playing = !*keep_playing;
            } else {
                // this is a pure copy paste
                // trust me I won't double race
                self.game.state.sort_piles();
                // we need to have a way to unsort the piles for display
                let s = self.game.state.clone();
                let mut rp = self.root_path.clone();
                let solver = self.solver.clone();
                // start move generation
                self.solver_move = Some((
                    true,
                    Arc::new(thread::spawn(move || {
                        solver
                            .lock()
                            .unwrap()
                            .next_move(&mut rp, &s, &generate_moves(&s))
                    })),
                ));
            }
        }

        // draw how many moves this has taken (not including undos)
        draw_text(
            &format!("Move {}", self.root_path.len()),
            SCREEN_WIDTH as f32 - 750.0,
            25.0,
            20.0,
            WHITE,
        );


        self.draw_game();

        SolveUserAction::None
    }
}

pub enum SolveUserAction {
    Menu,
    Play,
    Solve,
    None,
}

#[derive(Debug, Default, Clone, Copy)]
struct CardData {
    pub dragged_pos: Option<Vec2>,
    pub clickable_zone: Option<Rect>,
}

const TALON_START: Vec2 = Vec2 {
    x: TABLEAU_START.x,
    y: TABLEAU_START.y - 1.5 * CARD_SIZE.y,
};

fn initialize_card_data(game: &KPlusSolitaire) -> HashMap<Card, CardData> {
    let mut map = HashMap::with_capacity(52);

    let mut current = Rect {
        x: TALON_START.x,
        y: TALON_START.y,
        w: OVERLAP_OFFSET,
        h: CARD_SIZE.y,
    };
    let mut prev = None;
    for (i, card) in game.state.talon.0.iter().flatten().enumerate() {
        let d = if (i + 1).is_multiple_of(3) {
            prev = Some(card);
            CardData {
                dragged_pos: None,
                clickable_zone: Some(current),
            }
        } else {
            prev = None;
            CardData::default()
        };
        current.x += OVERLAP_OFFSET;
        map.insert(*card, d);
    }
    // make sure the last card, if it is available, has the full clickable zone
    if let Some(card) = prev {
        let d = map.get_mut(card).unwrap();
        let mut z = d.clickable_zone.unwrap();
        z.w = CARD_SIZE.x;
        d.clickable_zone = Some(z);
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

fn find_cursor_hover(
    game: &KPlusSolitaire,
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

fn update_all_clickable(game: &KPlusSolitaire, card_data: &mut HashMap<Card, CardData>) {
    // update talon
    let mut current_zone = Rect {
        x: TALON_START.x,
        y: TALON_START.y,
        w: OVERLAP_OFFSET,
        h: CARD_SIZE.y,
    };
    let mut prev = None;
    for (i, card) in game.state.talon.0.iter().enumerate() {
        if let Some(card) = card {
            let z = if game.state.is_reachable_talon(i as u8) {
                prev = Some(card);
                // the special index will be more visible
                if game.state.talon.1 == i as i8 {
                    // for each moved card, we have one more overlap available, but no bigger than
                    // the card width
                    let expanded = current_zone.w + OVERLAP_OFFSET * game.state.talon.3 as f32;
                    // no comparing of floats, but losing fractions of a pixel shouldn't affect
                    // much, so just cast to ints
                    current_zone.w = min(CARD_SIZE.x as u32, expanded as u32) as f32;
                }
                Some(current_zone)
            } else {
                prev = None;
                None
            };
            // more hacky resetting of mutable variables in confusing ways
            // I'm glad to have made the least maintanable code of all time
            current_zone.w = OVERLAP_OFFSET;

            let d = card_data.get_mut(card).unwrap();
            d.clickable_zone = z;
        }
        current_zone.x += OVERLAP_OFFSET;
    }
    // make sure the last card, if it is available, has the full clickable zone
    if let Some(card) = prev {
        let d = card_data.get_mut(card).unwrap();
        let mut z = d.clickable_zone.unwrap();
        z.w = CARD_SIZE.x;
        d.clickable_zone = Some(z);
    }

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
