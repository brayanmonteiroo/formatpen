mod app;
mod models;
mod services;
mod ui;
mod window;

use app::App;

/**
 * Função principal.
 */
fn main() {
    let app = App::new();
    app.run();
}
