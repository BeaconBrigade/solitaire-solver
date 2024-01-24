use egui::{CursorIcon, Id, InnerResponse, LayerId, Order, Rect, Response, Sense, Shape, Ui, Vec2};
use solitaire_game::{
    action::{Action, Coord, Location},
    state::find_last_idx,
};

use crate::{
    image::{card_to_image, BACK, BLANK},
    image_button, App,
};

/// Functions for drawing the tableau
impl App {
    pub fn draw_tableau(&mut self, ui: &mut Ui) {
        let id_source = "tableau_source";
        let mut source: Option<Coord> = None;
        let mut dest: Option<usize> = None;
        ui.horizontal(|ui| {
            for (col_idx, pile) in self.game.state.tableau.into_iter().enumerate() {
                ui.vertical(|ui| {
                    let can_accept_what_is_being_dragged = true;
                    let max_idx = find_last_idx(pile.0.into_iter(), |c| c.is_some());
                    let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                        // draw each card now
                        match max_idx {
                            Some(0) => {
                                let item_id = Id::new(id_source).with(col_idx).with(0);
                                drag_source(ui, item_id, |ui| {
                                    image_button!(ui, card_to_image(pile.0[0].unwrap()));
                                });

                                if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                    source = Some(Coord::new(Location::Tableau(col_idx as _), 0));
                                }
                            }
                            Some(last) => {
                                // draw card backs up
                                let mut prev: Option<Response> = None;
                                let face_down_len = (0..pile.1).len();
                                for _ in pile.0[0..pile.1 as usize].iter().flatten() {
                                    match prev.take() {
                                        Some(r) => {
                                            let mut new = r.rect;
                                            *new.top_mut() = 20.0 + new.top();
                                            prev = image_button!(ui, new, BACK).into();
                                        }
                                        None => {
                                            prev = image_button!(ui, BACK).into();
                                        }
                                    }
                                }
                                // draw the up cards
                                for (i, _) in
                                    pile.0[pile.1 as usize..=last].iter().flatten().enumerate()
                                {
                                    match prev.take() {
                                        Some(r) => {
                                            let mut new = r.rect;
                                            *new.top_mut() = 20.0 + new.top();
                                            let item_id =
                                                Id::new(id_source).with(col_idx).with(last);
                                            drag_source(ui, item_id, |ui| {
                                                prev = image_button!(
                                                    ui,
                                                    new,
                                                    // add face_down_len to index starting from face up cards
                                                    card_to_image(
                                                        pile.0[i + face_down_len].unwrap()
                                                    )
                                                )
                                                .into();
                                            });
                                            if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                                source = Some(Coord::new(
                                                    Location::Tableau(col_idx as _),
                                                    i as u8 + face_down_len as u8,
                                                ));
                                            }
                                        }
                                        None => {
                                            let item_id =
                                                Id::new(id_source).with(col_idx).with(last);
                                            drag_source(ui, item_id, |ui| {
                                                prev = image_button!(
                                                    ui,
                                                    // add face_down_len to index starting from face up cards
                                                    card_to_image(
                                                        pile.0[i + face_down_len].unwrap()
                                                    )
                                                )
                                                .into();
                                            });
                                            if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                                source = Some(Coord::new(
                                                    Location::Tableau(col_idx as _),
                                                    i as u8 + face_down_len as u8,
                                                ));
                                            }
                                        }
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

        if let (Some(coord), Some(dest)) = (source, dest) {
            if ui.input(|i| i.pointer.any_released()) {
                // do the drop:
                let from = coord;
                let to = Coord::new(
                    Location::Tableau(dest as _),
                    find_last_idx(self.game.state.tableau[dest].0.into_iter(), |c| c.is_some())
                        .map(|i| (i + 1) as u8)
                        .unwrap_or(0),
                );
                println!("doing move: f({from:?}) - t({to:?})");
                self.game.do_move(Action::Move(from, to));
            }
        }
    }
}

/// drag and drop code stolen from [here](https://github.com/emilk/egui/blob/0.25.0/crates/egui_demo_lib/src/demo/drag_and_drop.rs)

pub fn drag_source(ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, Sense::drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
    } else {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);

        // Paint the body to a new layer:
        let layer_id = LayerId::new(Order::Tooltip, id);
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
        }
    }
}

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        fill = ui.visuals().gray_out(fill);
        stroke.color = ui.visuals().gray_out(stroke.color);
    }

    ui.painter().set(
        where_to_put_background,
        epaint::RectShape::new(rect, style.rounding, fill, stroke),
    );

    InnerResponse::new(ret, response)
}
