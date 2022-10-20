#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::game::Game;

slint::include_modules!();

mod game;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let ui = AppWindow::new();

    let game = Game::new_empty();

    ui.set_words(game.words);

    ui.on_key_pressed(|key| {
        println!("{key}");
    });

    // let ui_handle = ui.as_weak();
    // ui.on_request_increase_value(move || {
    //     let ui = ui_handle.unwrap();
    //     ui.set_counter(ui.get_counter() + 1);
    // });

    ui.run();
}
