use crate::window;
use gtk::prelude::*;
use std::path::PathBuf;

fn icon_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(appimage) = std::env::var("APPIMAGE") {
        if let Some(parent) = PathBuf::from(appimage).parent() {
            paths.push(parent.join("usr/share/icons"));
        }
    }

    paths.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/icons"),
    );

    paths
}

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
            if let Some(display) = gtk::gdk::Display::default() {
                let theme = gtk::IconTheme::for_display(&display);
                for icons_dir in icon_search_paths() {
                    if icons_dir.exists() {
                        theme.add_search_path(icons_dir);
                    }
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
