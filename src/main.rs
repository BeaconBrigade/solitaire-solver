mod image;

use eframe::{egui, CreationContext, NativeOptions};
use egui::{Color32, Frame, Margin, Response, RichText, Vec2, ViewportBuilder};
use image::{card_to_image, BACK, BLANK};
use solitaire_game::action::{Action, Coord, Location};
use solitaire_game::state::{find_last, find_last_idx};
use solitaire_game::Solitaire;

const IMAGE_SIZE: Vec2 = Vec2::new(75.0, 108.9);

fn main() -> eframe::Result<()> {
    let opts = NativeOptions {
        window_builder: Some(Box::new(|mut v: ViewportBuilder| {
            v.min_inner_size = Some(Vec2::new(800.0, 600.0));
            v
        })),
        default_theme: eframe::Theme::Light,
        follow_system_theme: false,
        ..Default::default()
    };
    eframe::run_native("Solitaire", opts, Box::new(|cc| Box::new(App::new(cc))))?;

    Ok(())
}

#[derive(Debug)]
struct App {
    game: Solitaire,
}

impl App {
    pub fn new(cc: &CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let game = Solitaire::default();
        Self { game }
    }
}

/// Use less code to build out image buttons
macro_rules! image_button {
    ($ui:expr, $path:expr) => {
        $ui.add(
            egui::ImageButton::new(egui::Image::new($path).fit_to_exact_size(IMAGE_SIZE))
                .frame(false)
                .sense(egui::Sense::drag()),
        )
    };
    ($ui:expr, $pos:expr, $path:expr) => {
        $ui.put(
            $pos,
            egui::ImageButton::new(egui::Image::new($path).fit_to_exact_size(IMAGE_SIZE))
                .frame(false)
                .sense(egui::Sense::drag()),
        )
    };
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            style.visuals.override_text_color = Color32::LIGHT_GRAY.into();
        });

        // set background colour
        let frame = Frame {
            // nice green colour
            fill: Color32::from_rgb(0x01, 0x7e, 0x04),
            inner_margin: Margin::same(20.0),
            ..Default::default()
        };
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(
                    RichText::new("Cool Solitaire Game")
                        .strong()
                        .color(Color32::LIGHT_GRAY),
                )
            });
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                // draw foundation
                for pile in self.game.state.foundation {
                    let card = find_last(pile.into_iter(), |c| c.is_some()).flatten();
                    image_button!(
                        ui,
                        card.map(card_to_image).unwrap_or_else(|| BLANK.to_string())
                    );
                }

                // draw talon and deck
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let top_of_deck =
                        if self.game.state.talon.1 as usize == self.game.state.talon.0.len() {
                            BLANK
                        } else {
                            BACK
                        };
                    image_button!(ui, top_of_deck);
                    let top_of_talon = if self.game.state.talon.1 < 0 {
                        BLANK.to_string()
                    } else {
                        self.game.state.talon.0[self.game.state.talon.1 as usize]
                            .map(card_to_image)
                            .unwrap_or_else(|| BLANK.to_string())
                    };
                    image_button!(ui, top_of_talon);
                });
            });

            // draw the tableau
            ui.add_space(50.0);
            ui.horizontal(|ui| {
                for pile in self.game.state.tableau {
                    ui.vertical(|ui| {
                        let max_idx = find_last_idx(pile.into_iter(), |c| c.is_some());
                        let mut iter = pile.into_iter();

                        match max_idx {
                            Some(0) => {
                                image_button!(ui, card_to_image(iter.next().flatten().unwrap()));
                            }
                            Some(i) => {
                                // draw card backs up to the top card
                                let mut last: Option<Response> = None;
                                for _ in iter.take(i).flatten() {
                                    match last.take() {
                                        Some(r) => {
                                            let mut new = r.rect;
                                            *new.top_mut() = 20.0 + new.top();
                                            last = image_button!(ui, new, BACK).into();
                                        }
                                        None => {
                                            last = image_button!(ui, BACK).into();
                                        }
                                    }
                                }

                                let mut new = last.unwrap().rect;
                                *new.top_mut() = 20.0 + new.top();
                                // draw top card
                                image_button!(ui, new, card_to_image(pile[i].unwrap()));
                            }
                            None => {
                                image_button!(ui, BLANK.to_string());
                            }
                        };
                    });
                }
            });
        });
    }
}