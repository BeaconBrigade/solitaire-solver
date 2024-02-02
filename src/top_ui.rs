use egui::{CursorIcon, Id, RichText, Ui};
use epaint::Color32;
use solitaire_game::{
    action::{Action, Coord, Location},
    state::find_last_idx,
};

use crate::{
    image::{card_to_image, BACK, BLANK},
    image_button,
    tableau::{drag_source, drop_target},
    App,
};

impl App {
    pub fn draw_heading(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(
                RichText::new("Cool Solitaire Game")
                    .strong()
                    .color(Color32::LIGHT_GRAY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui
                    .button(RichText::new("Restart").color(Color32::BLACK))
                    .clicked()
                {
                    self.game = self.original;
                }
            });
        });
    }

    pub fn draw_foundation(&mut self, ui: &mut Ui) -> (Option<Coord>, Option<Coord>) {
        let mut source: Option<Coord> = None;
        let mut dest: Option<usize> = None;

        for (col_idx, pile) in self.game.state.foundation.into_iter().enumerate() {
            let can_accept_what_is_being_dragged = true;
            let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                let idx = find_last_idx(pile.into_iter(), |c| c.is_some());
                let item_id = Id::new("foundation").with(col_idx);
                let (card, under_card) = match idx {
                    Some(0) => (card_to_image(pile[0].unwrap()), BLANK.to_string()),
                    Some(n) => (
                        card_to_image(pile[n].unwrap()),
                        card_to_image(pile[n - 1].unwrap()),
                    ),
                    None => (BLANK.to_string(), BLANK.to_string()),
                };
                let res = image_button!(ui, under_card);
                drag_source(ui, item_id, |ui| {
                    image_button!(ui, res.rect, card);
                });

                if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                    source = Some(Coord::new(
                        Location::Foundation(col_idx as _),
                        idx.unwrap() as u8,
                    ));
                }
            })
            .response;

            let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
            if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
                dest = Some(col_idx);
            }
        }

        let dest = dest.map(|d| {
            Coord::new(
                Location::Foundation(d as _),
                find_last_idx(self.game.state.foundation[d].into_iter(), |c| c.is_some())
                    .map(|i| (i + 1) as u8)
                    .unwrap_or(0),
            )
        });
        (source, dest)
    }

    pub fn draw_talon(&mut self, ui: &mut Ui) -> (Option<Coord>, Option<Coord>) {
        let mut source: Option<Coord> = None;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            let top_of_deck = if self.game.state.talon.1 as usize
                == (self.game.state.talon.2 as usize).wrapping_sub(1)
            {
                BLANK
            } else {
                BACK
            };
            let top_res = image_button!(ui, top_of_deck);
            if top_res.clicked() {
                self.game.do_move(Action::TurnStock);
            }
            if top_res.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }

            ui.add_space(50.0);

            let item_id = Id::new("talon");
            if self.game.state.talon.1 < 0 {
                image_button!(ui, BLANK);
            } else {
                let top = self.game.state.talon.0[self.game.state.talon.1 as usize]
                    .map(card_to_image)
                    .unwrap_or_else(|| BLANK.to_string());

                ui.horizontal(|ui| {
                    let res = match self.game.state.talon.1 {
                        // no cards underneath
                        -1 | 0 => image_button!(ui, BLANK),
                        // one card underneath
                        1 => {
                            let mut res = image_button!(
                                ui,
                                self.game.state.talon.0[self.game.state.talon.1 as usize - 1]
                                    .map(card_to_image)
                                    .unwrap()
                            );
                            let mut new = res.rect;
                            *new.right_mut() += 30.0;
                            res.rect = new;
                            res
                        }
                        // two cards underneath
                        _ => {
                            let res = image_button!(
                                ui,
                                self.game.state.talon.0[self.game.state.talon.1 as usize - 2]
                                    .map(card_to_image)
                                    .unwrap()
                            );
                            let mut new = res.rect;
                            *new.right_mut() += 30.0;
                            let mut res = image_button!(
                                ui,
                                new,
                                self.game.state.talon.0[self.game.state.talon.1 as usize - 1]
                                    .map(card_to_image)
                                    .unwrap()
                            );

                            new = res.rect;
                            *new.right_mut() += 30.0;
                            res.rect = new;
                            res
                        }
                    };

                    if top != BLANK {
                        drag_source(ui, item_id, |ui| {
                            image_button!(ui, res.rect, top);
                        });
                    }
                });
            };

            if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                source = Some(Coord::new(Location::Talon, self.game.state.talon.1 as _));
            }
        });
        (source, None)
    }
}
