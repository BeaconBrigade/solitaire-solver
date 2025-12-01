use egui::{emath::TSTransform, DragAndDrop, Frame, Id, LayerId, Order, Rect, Ui, UiBuilder, Vec2};
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
                                let mut dragged = false;
                                let mut dragged_idx = 0;
                                for (i, cur_card) in
                                    pile.0[pile.1 as usize..=last].iter().flatten().enumerate()
                                {
                                    let mut new = prev;
                                    *new.top_mut() = 20.0 + new.top();
                                    let item_id = Id::new(id_source).with(col_idx).with(i);
                                    if DragAndDrop::payload::<Coord>(ui.ctx()).is_some() {
                                        dragged = true;
                                    }
                                    // only draggable if no cards are being dragged
                                    let delta = if !dragged {
                                        let coord = Coord::new(
                                            Location::Tableau(col_idx as _),
                                            i as u8 + face_down_len as u8,
                                        );
                                        let response = ui
                                            .dnd_drag_source(item_id, coord, |ui| {
                                                prev = image_button!(
                                                    ui,
                                                    new,
                                                    // add face_down_len to index starting from face up cards
                                                    card_to_image(*cur_card)
                                                )
                                                .rect
                                            })
                                            .response;
                                        if DragAndDrop::payload(ui.ctx()).map(|p| *p) == Some(coord)
                                        {
                                            if let Some(pointer_pos) =
                                                ui.ctx().pointer_interact_pos()
                                            {
                                                pointer_pos - response.rect.center()
                                            } else {
                                                Vec2::ZERO
                                            }
                                        } else {
                                            Vec2::ZERO
                                        }
                                    } else {
                                        let layer_id = LayerId::new(Order::Middle, item_id);
                                        let response = ui
                                            .scope_builder(
                                                UiBuilder::new().layer_id(layer_id),
                                                |ui| {
                                                    prev = image_button!(
                                                        ui,
                                                        // add face_down_len to index starting from face up cards
                                                        card_to_image(*cur_card)
                                                    )
                                                    .rect
                                                },
                                            )
                                            .response;
                                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                            let delta = pointer_pos - response.rect.center();
                                            let delta =
                                                Vec2::new(0.0, 20.0 * dragged_idx as f32) + delta;
                                            let transform = TSTransform {
                                                scaling: 1.0,
                                                translation: delta,
                                            };
                                            ui.ctx().transform_layer_shapes(layer_id, transform);
                                        }
                                        dragged_idx += 1;
                                        Vec2::ZERO
                                    };
                                    // make sure our last Rect we use to translate the cards underneath, is
                                    // the properly translated hovered card.
                                    if delta != Vec2::ZERO {
                                        dragged = true;
                                        dragged_idx = 1;
                                        prev = prev.translate(delta);
                                    }
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
