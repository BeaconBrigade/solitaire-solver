use macroquad::prelude::*;

#[macroquad::main("Solitaire")]
async fn main() {
    loop {
        clear_background(GREEN);

        next_frame().await;
    }
}
