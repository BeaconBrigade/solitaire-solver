use egui::{RichText, Ui};
use epaint::Color32;
use solitaire_game::{action::Action, state::find_last};

use crate::{
    image::{card_to_image, BACK, BLANK},
    image_button, App,
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

    pub fn draw_foundation(&mut self, ui: &mut Ui) {
        // draw foundation
        for pile in self.game.state.foundation {
            let card = find_last(pile.into_iter(), |c| c.is_some()).flatten();
            image_button!(
                ui,
                card.map(card_to_image).unwrap_or_else(|| BLANK.to_string())
            );
        }
    }

    pub fn draw_talon(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            let top_of_deck = if self.game.state.talon.1 as usize == self.game.state.talon.0.len() {
                BLANK
            } else {
                BACK
            };
            if image_button!(ui, top_of_deck).clicked() {
                self.game.do_move(Action::TurnStock);
            }
            let top_of_talon = if self.game.state.talon.1 < 0 {
                BLANK.to_string()
            } else {
                self.game.state.talon.0[self.game.state.talon.1 as usize]
                    .map(card_to_image)
                    .unwrap_or_else(|| BLANK.to_string())
            };
            image_button!(ui, top_of_talon);
        });
    }
}
