use egui::{CursorIcon, Id, InnerResponse, LayerId, Order, Rect, Sense, Ui, Vec2};
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
                    let can_accept_what_is_being_dragged = true;
                    let max_idx = find_last_idx(pile.0.into_iter(), |c| c.is_some());
                    // make sure rect is initialized as nothing
                    let mut prev: Rect = Rect::ZERO;
                    let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                        // draw each card now
                        match max_idx {
                            Some(0) => {
                                prev = image_button!(ui, BLANK.to_string()).rect;
                                let item_id = Id::new(id_source).with(col_idx).with(0);
                                drag_source(ui, item_id, |ui| {
                                    image_button!(ui, prev, card_to_image(pile.0[0].unwrap()));
                                });

                                if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                    source = Some(Coord::new(Location::Tableau(col_idx as _), 0));
                                }
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
                                for (i, _) in
                                    pile.0[pile.1 as usize..=last].iter().flatten().enumerate()
                                {
                                    let mut new = prev;
                                    *new.top_mut() = 20.0 + new.top();
                                    let item_id = Id::new(id_source).with(col_idx).with(i);
                                    // only draggable if no cards are being dragged
                                    let delta = if !dragged {
                                        drag_source(ui, item_id, |ui| {
                                            prev = image_button!(
                                                ui,
                                                new,
                                                // add face_down_len to index starting from face up cards
                                                card_to_image(pile.0[i + face_down_len].unwrap())
                                            )
                                            .rect
                                        })
                                    } else {
                                        let layer_id = LayerId::new(Order::Middle, item_id);
                                        let response = ui
                                            .with_layer_id(layer_id, |ui| {
                                                prev = image_button!(
                                                    ui,
                                                    // add face_down_len to index starting from face up cards
                                                    card_to_image(
                                                        pile.0[i + face_down_len].unwrap()
                                                    )
                                                )
                                                .rect
                                            })
                                            .response;
                                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                            let delta = pointer_pos - response.rect.center();
                                            let delta =
                                                Vec2::new(0.0, 20.0 * dragged_idx as f32) + delta;
                                            ui.ctx().translate_layer(layer_id, delta);
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
                                    if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                        source = Some(Coord::new(
                                            Location::Tableau(col_idx as _),
                                            i as u8 + face_down_len as u8,
                                        ));
                                    }
                                }
                            }
                            None => {
                                image_button!(ui, BLANK.to_string());
                            }
                        };
                    })
                    .response;

                    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
                    if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
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

/// drag and drop code stolen from [here](https://github.com/emilk/egui/blob/0.25.0/crates/egui_demo_lib/src/demo/drag_and_drop.rs)

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) -> Vec2 {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        Vec2::ZERO
    } else {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Background, id);
        let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty [`Response`])
        // So this is fine!

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);

            delta
        } else {
            Vec2::ZERO
        }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    _can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let _is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    // let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (_rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    InnerResponse::new(ret, response)
}
