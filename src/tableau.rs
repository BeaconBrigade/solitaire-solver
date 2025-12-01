use egui::{DragAndDrop, Frame, Id, LayerId, Order, Rect, Ui, UiBuilder};
use solitaire_game::{
    action::{Coord, Location},
    state::find_last_idx,
};

use crate::{
    image::{card_to_image, BACK, BLANK},
    image_button, App,
};

/// Functions for drawing the tableau
impl App {
    pub fn draw_tableau(&mut self, ui: &mut Ui) -> (Option<Coord>, Option<Coord>) {
        let id_source = "tableau_source";
        let mut source: Option<Coord> = None;
        let mut dest: Option<usize> = None;
        ui.horizontal(|ui| {
            for (col_idx, pile) in self.game.state.tableau.into_iter().enumerate() {
                ui.vertical(|ui| {
                    // TODO: tweak
                    let frame = Frame::default();
                    let max_idx = find_last_idx(pile.0.into_iter(), |c| c.is_some());
                    // make sure rect is initialized as nothing
                    let mut prev: Rect = Rect::ZERO;

                    let (_response, dropped) = ui.dnd_drop_zone(frame, |ui| {
                        // drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                        // draw each card now
                        match max_idx {
                            Some(0) => {
                                prev = image_button!(ui, BLANK.to_string()).rect;
                                let item_id = Id::new(id_source).with(col_idx).with(0);
                                let coord = Coord::new(Location::Tableau(col_idx as _), 0);
                                ui.dnd_drag_source(item_id, coord, |ui| {
                                    image_button!(ui, prev, card_to_image(pile.0[0].unwrap()));
                                });
                            }
                            Some(last) => {
                                // draw card backs up
                                prev = image_button!(ui, BLANK.to_string()).rect;
                                *prev.top_mut() = prev.top() - 20.0;
                                let face_down_len = (0..pile.1).len();
                                for _ in pile.0[0..pile.1 as usize].iter().flatten() {
                                    let mut new = prev;
                                    *new.top_mut() = 20.0 + new.top();
                                    prev = image_button!(ui, new, BACK).rect;
                                }
                                // draw the up cards

                                // find card which is being dragged
                                let dragged_idx: usize = DragAndDrop::payload::<Coord>(ui.ctx())
                                    .filter(|c| c.location == Location::Tableau(col_idx as _))
                                    .map(|c| c.idx as usize)
                                    .unwrap_or(100);

                                // draw cards normally until dragged card
                                for (i, cur_card) in pile.0
                                    [pile.1 as usize..dragged_idx.min(last + 1)]
                                    .iter()
                                    .flatten()
                                    .enumerate()
                                {
                                    let idx = (i + face_down_len) as u8;
                                    let item_id = Id::new(id_source).with(col_idx).with(idx);
                                    let coord = Coord::new(Location::Tableau(col_idx as _), idx);
                                    let mut new = prev;
                                    *new.top_mut() = 20.0 + new.top();
                                    ui.dnd_drag_source(item_id, coord, |ui| {
                                        prev =
                                            image_button!(ui, new, card_to_image(*cur_card)).rect;
                                    });
                                }

                                // we do have more things to draw
                                println!("last={last}, dragged={dragged_idx}");
                                if last >= dragged_idx {
                                    // now draw the dragged item (let egui handle it)
                                    let item_id =
                                        Id::new(id_source).with(col_idx).with(dragged_idx);
                                    let coord = Coord::new(
                                        Location::Tableau(col_idx as _),
                                        dragged_idx as u8,
                                    );
                                    println!(
                                        "dragged is {}",
                                        card_to_image(pile.0[dragged_idx].unwrap())
                                    );
                                    ui.scope_builder(
                                        UiBuilder::new().layer_id(LayerId::new(
                                            Order::Tooltip,
                                            item_id.with("chicken-nugget"),
                                        )),
                                        |ui| {
                                            let base_pos = ui
                                                .dnd_drag_source(item_id, coord, |ui| {
                                                    image_button!(
                                                        ui,
                                                        card_to_image(pile.0[dragged_idx].unwrap())
                                                    );
                                                })
                                                .response
                                                .rect;

                                            for (i, cur_card) in pile.0[dragged_idx + 1..=last]
                                                .iter()
                                                .flatten()
                                                .enumerate()
                                            {
                                                let idx = (i + face_down_len) as u8;
                                                let item_id =
                                                    Id::new(id_source).with(col_idx).with(idx);
                                                // let coord =
                                                //     Coord::new(Location::Tableau(col_idx as _), idx);
                                                let mut pos = base_pos;
                                                *pos.top_mut() = 20.0 * i as f32 + pos.top();
                                                println!("drawing card at offset: {}", 20 * i);
                                                ui.scope_builder(
                                                    UiBuilder::new().layer_id(LayerId::new(
                                                        Order::Foreground,
                                                        item_id,
                                                    )),
                                                    |ui| {
                                                        image_button!(
                                                            ui,
                                                            pos,
                                                            card_to_image(*cur_card)
                                                        );
                                                    },
                                                );
                                            }
                                        },
                                    );
                                }
                            }
                            None => {
                                image_button!(ui, BLANK.to_string());
                            }
                        };
                    });

                    if let Some(dropped) = dropped {
                        // let _ = response.response.dnd_release_payload::<Coord>();
                        source = Some(*dropped);
                        dest = Some(col_idx);
                    }
                });
            }
        });
        let dest = dest.map(|d| {
            Coord::new(
                Location::Tableau(d as _),
                find_last_idx(self.game.state.tableau[d].0.into_iter(), |c| c.is_some())
                    .map(|i| (i + 1) as u8)
                    .unwrap_or(0),
            )
        });
        (source, dest)
    }
}
