#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

use application::AppState;

mod application;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let nouns_bytes = include_bytes!("../files/russian_nouns.txt");

    let ui = AppWindow::new();
    let app = AppState::new(5, ui.as_weak(), nouns_bytes);
    app.run();
}
