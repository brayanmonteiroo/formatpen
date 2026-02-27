use crate::window;
use gtk::prelude::*;

/**
 * Implementação da aplicação.
 */
pub struct App {
    app: libadwaita::Application,
}

/**
 * Implementação da aplicação.
 */
impl App {
    /**
     * Cria uma nova instância da aplicação. Carrega os ícones da pasta data/icons.
     */
    pub fn new() -> Self {
        let app = libadwaita::Application::new(
            Some("com.formatpen.FormatPen"),
            gtk::gio::ApplicationFlags::empty(),
        );

        app.connect_activate(|app| {
            let icons_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/icons");
            if icons_dir.exists() {
                if let Some(display) = gtk::gdk::Display::default() {
                    let theme = gtk::IconTheme::for_display(&display);
                    theme.add_search_path(icons_dir);
                }
            }
            let window = window::Window::new(app);
            window.present();
        });

        Self { app }
    }

    /**
     * Executa a aplicação.
     */
    pub fn run(self) {
        self.app.run();
    }
}
