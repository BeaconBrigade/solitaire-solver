mod image;
mod tableau;
mod top_ui;

use std::env;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use eframe::{egui, CreationContext, NativeOptions};
use egui::{Color32, Frame, Margin, ViewportBuilder};
use epaint::Vec2;
use solitaire_game::action::Action;
use solitaire_game::deck::Deck;
use solitaire_game::Solitaire;

pub const IMAGE_SIZE: Vec2 = Vec2::new(75.0, 108.9);

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
    original: Solitaire,
}

impl App {
    pub fn new(cc: &CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let deck = if let Some(path) = env::args().nth(1) {
            let mut deck_file = File::open(path).unwrap();
            let mut contents = String::new();
            deck_file.read_to_string(&mut contents).unwrap();

            Deck::from_str(&contents).unwrap()
        } else {
            Deck::new_shuffled()
        };
        println!("{}", deck.to_string());
        let game = Solitaire::with_deck(deck);

        let original = game;
        Self { game, original }
    }
}

/// Use less code to build out image buttons
#[macro_export]
macro_rules! image_button {
    ($ui:expr, $path:expr) => {
        $ui.add(
            egui::Image::new($path)
                .fit_to_exact_size($crate::IMAGE_SIZE)
                .sense(egui::Sense::drag()),
        )
    };
    ($ui:expr, $pos:expr, $path:expr) => {
        $ui.put(
            $pos,
            egui::Image::new($path)
                .fit_to_exact_size($crate::IMAGE_SIZE)
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
            self.draw_heading(ui);
            ui.add_space(10.0);

            let (mut f_source, mut f_dest, mut p_source, mut p_dest) = (None, None, None, None);
            ui.horizontal(|ui| {
                (f_source, f_dest) = self.draw_foundation(ui);
                (p_source, p_dest) = self.draw_talon(ui);
            });

            // draw the tableau
            ui.add_space(50.0);
            let (t_source, t_dest) = self.draw_tableau(ui);

            // combine all sources and dests; there should only be one of each
            let source = f_source.or(t_source).or(p_source);
            let dest = f_dest.or(t_dest).or(p_dest);

            if let (Some(coord), Some(dest)) = (source, dest) {
                if ui.input(|i| i.pointer.any_released()) {
                    // do the drop:
                    let from = coord;
                    let to = dest;
                    println!("doing move: f({from:?}) - t({to:?})");
                    self.game.do_move(Action::Move(from, to));
                }
            }
        });
    }
}
