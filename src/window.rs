use crate::environment::{self, DriveRefreshOutcome};
use crate::format_errors;
use crate::models::Drive;
use crate::services::FormatService;
use crate::ui::{DriveList, FormatForm};
use crate::updater::{self, AutoUpdatePref, UpdateOutcome};
use gtk::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/**
 * Cria um toast com uma mensagem longa.
 */
fn toast_long_message(msg: &str) -> libadwaita::Toast {
    let label = gtk::Label::new(Some(msg));
    label.set_wrap(true);
    label.set_max_width_chars(55);
    label.set_ellipsize(gtk::pango::EllipsizeMode::None);
    label.add_css_class("heading");
    libadwaita::Toast::builder()
        .custom_title(&label)
        .timeout(8)
        .build()
}

fn show_restart_dialog(
    window: &libadwaita::ApplicationWindow,
    app: &libadwaita::Application,
    appimage_path: &PathBuf,
) {
    let dialog = libadwaita::MessageDialog::new(
        Some(window),
        Some("Atualização instalada"),
        Some("Reinicie o FormatPen para usar a nova versão."),
    );
    dialog.add_response("later", "Depois");
    dialog.add_response("restart", "Reiniciar agora");
    dialog.set_default_response(Some("restart"));
    dialog.set_close_response("later");

    let app = app.clone();
    let path = appimage_path.clone();
    dialog.connect_response(None, move |dialog, response| {
        dialog.close();
        if response == "restart" {
            if let Err(e) = updater::restart_appimage(&path) {
                eprintln!("Erro ao reiniciar: {e}");
            } else {
                app.quit();
            }
        }
    });
    dialog.present();
}

fn run_update_flow(
    app: &libadwaita::Application,
    window: &libadwaita::ApplicationWindow,
    toast_overlay: &libadwaita::ToastOverlay,
    manual: bool,
) {
    let Some(appimage) = updater::appimage_path() else {
        return;
    };

    if manual {
        toast_overlay.add_toast(libadwaita::Toast::new("Verificando atualizações..."));
    }

    let app = app.clone();
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();

    glib::spawn_future_local(async move {
        let result = gio::spawn_blocking(move || updater::check_and_apply_update(&appimage)).await;

        match result {
            Ok(UpdateOutcome::UpToDate) => {
                if manual {
                    toast_overlay.add_toast(libadwaita::Toast::new(
                        "Você já está na versão mais recente.",
                    ));
                }
            }
            Ok(UpdateOutcome::Updated { path }) => {
                show_restart_dialog(&window, &app, &path);
            }
            Ok(UpdateOutcome::Failed(e)) => {
                eprintln!("Falha ao atualizar FormatPen: {e}");
                if manual {
                    let toast = toast_long_message(&format!(
                        "Não foi possível verificar ou baixar a atualização.\n{e}"
                    ));
                    toast_overlay.add_toast(toast);
                }
            }
            Ok(UpdateOutcome::NotApplicable) => {}
            Err(e) => {
                eprintln!("Erro ao verificar atualizações: {e:?}");
                if manual {
                    toast_overlay
                        .add_toast(libadwaita::Toast::new("Erro ao verificar atualizações."));
                }
            }
        }
    });
}

fn show_update_consent_dialog(
    app: &libadwaita::Application,
    window: &libadwaita::ApplicationWindow,
    toast_overlay: &libadwaita::ToastOverlay,
) {
    let dialog = libadwaita::MessageDialog::new(
        Some(window),
        Some("Verificar atualizações?"),
        Some(
            "O FormatPen pode verificar novas versões no GitHub e baixar apenas \
             o que mudou (economiza banda). Você pode desativar depois com \
             FORMATPEN_NO_UPDATE=1.",
        ),
    );
    dialog.add_response("deny", "Não perguntar de novo");
    dialog.add_response("allow", "Permitir");
    dialog.set_default_response(Some("allow"));
    dialog.set_close_response("deny");

    let app = app.clone();
    let window = window.clone();
    let toast_overlay = toast_overlay.clone();
    dialog.connect_response(None, move |dialog, response| {
        dialog.close();
        let enabled = response == "allow";
        if let Err(e) = updater::save_auto_update_pref(enabled) {
            eprintln!("Erro ao salvar preferência de update: {e}");
        }
        if enabled {
            let app2 = app.clone();
            let window2 = window.clone();
            let toast2 = toast_overlay.clone();
            glib::timeout_add_seconds_local(2, move || {
                run_update_flow(&app2, &window2, &toast2, false);
                glib::ControlFlow::Break
            });
        }
    });
    dialog.present();
}

fn setup_appimage_updates(
    app: &libadwaita::Application,
    window: &libadwaita::ApplicationWindow,
    toast_overlay: &libadwaita::ToastOverlay,
    update_btn: &gtk::Button,
) {
    if !updater::is_appimage() || updater::updates_disabled() {
        update_btn.set_visible(false);
        return;
    }

    {
        let app = app.clone();
        let window = window.clone();
        let toast_overlay = toast_overlay.clone();
        update_btn.connect_clicked(move |_| {
            run_update_flow(&app, &window, &toast_overlay, true);
        });
    }

    match updater::load_auto_update_pref() {
        AutoUpdatePref::NotAsked => {
            let app = app.clone();
            let window = window.clone();
            let toast_overlay = toast_overlay.clone();
            glib::idle_add_local_once(move || {
                show_update_consent_dialog(&app, &window, &toast_overlay);
            });
        }
        AutoUpdatePref::Enabled => {
            let app = app.clone();
            let window = window.clone();
            let toast_overlay = toast_overlay.clone();
            glib::timeout_add_seconds_local(2, move || {
                run_update_flow(&app, &window, &toast_overlay, false);
                glib::ControlFlow::Break
            });
        }
        AutoUpdatePref::Disabled => {}
    }
}

/**
 * Atualiza banner, lista e formulário conforme o estado do ambiente e dos discos.
 */
fn apply_refresh_outcome(
    outcome: &DriveRefreshOutcome,
    banner: &libadwaita::Banner,
    drive_list: &DriveList,
    format_form: &FormatForm,
    selected_drive: &Rc<RefCell<Option<Drive>>>,
) {
    *selected_drive.borrow_mut() = None;
    format_form.set_sensitive(false);

    if let Some(issue) = &outcome.runtime_issue {
        banner.set_title(&issue.title);
        banner.remove_css_class("warning");
        banner.set_revealed(true);
        drive_list.set_status_message(Some(&issue.detail));
        drive_list.set_drives(Vec::new());
        return;
    }

    if let Some(warning) = &outcome.format_tools_warning {
        banner.set_title("Ferramentas de formatação incompletas no sistema");
        banner.add_css_class("warning");
        banner.set_revealed(true);
        drive_list.set_status_message(Some(warning));
    } else {
        banner.set_revealed(false);
        banner.remove_css_class("warning");
    }

    drive_list.set_drives(outcome.drives.clone());

    if outcome.drives.is_empty() {
        if outcome.format_tools_warning.is_none() {
            drive_list.set_status_message(Some(
                "Nenhum dispositivo removível encontrado. Conecte um pendrive e clique em Atualizar.",
            ));
        }
        return;
    }

    if outcome.format_tools_warning.is_none() {
        drive_list.set_status_message(None);
    }

    if let Some(drive) = drive_list.selected_drive() {
        *selected_drive.borrow_mut() = Some(drive);
        format_form.set_sensitive(true);
    }
}

/**
 * Implementação da janela principal.
 */
pub struct Window {
    inner: libadwaita::ApplicationWindow,
}

/**
 * Implementação da janela principal.
 */
impl Window {
    /**
     * Apresenta a janela.
     */
    pub fn present(&self) {
        self.inner.present();
    }

    /**
     * Cria uma nova instância da janela principal.
     */
    pub fn new(app: &libadwaita::Application) -> Self {
        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("FormatPen - Formatador de Pendrive")
            .icon_name("com.formatpen.FormatPen")
            .default_width(480)
            .default_height(-1)
            .resizable(false)
            .build();

        let header = libadwaita::HeaderBar::new();
        let title = libadwaita::WindowTitle::new("FormatPen", "Formatador de Pendrive");
        header.set_title_widget(Some(&title));

        let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Atualizar lista de dispositivos"));

        let update_btn = gtk::Button::from_icon_name("system-software-update-symbolic");
        update_btn.set_tooltip_text(Some("Verificar atualizações"));
        header.pack_end(&refresh_btn);
        header.pack_end(&update_btn);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.append(&header);

        let banner = libadwaita::Banner::new("");
        banner.set_revealed(false);
        main_box.append(&banner);

        let clamp = libadwaita::Clamp::new();
        clamp.set_maximum_size(480);
        clamp.set_margin_top(12);
        clamp.set_margin_bottom(12);
        clamp.set_margin_start(12);
        clamp.set_margin_end(12);

        let toast_overlay = libadwaita::ToastOverlay::new();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_margin_top(8);
        content.set_margin_bottom(8);

        let drive_list = DriveList::new();
        content.append(&drive_list.widget);

        let format_form = FormatForm::new();
        format_form.set_sensitive(false);
        content.append(&format_form.widget);

        let selected_drive: Rc<RefCell<Option<Drive>>> = Rc::new(RefCell::new(None));

        {
            let selected = selected_drive.clone();
            let form = format_form.clone();
            drive_list.connect_selection_changed(move |drive| {
                *selected.borrow_mut() = drive.clone();
                form.set_sensitive(drive.is_some());
            });
        }

        apply_refresh_outcome(
            &environment::refresh_drives(),
            &banner,
            &drive_list,
            &format_form,
            &selected_drive,
        );

        {
            let dl = drive_list.clone();
            let bn = banner.clone();
            let ff = format_form.clone();
            let sel = selected_drive.clone();
            refresh_btn.connect_clicked(move |_| {
                apply_refresh_outcome(
                    &environment::refresh_drives(),
                    &bn,
                    &dl,
                    &ff,
                    &sel,
                );
            });
        }

        {
            let selected = selected_drive.clone();
            let dl = drive_list.clone();
            let ff = format_form.clone();
            let wc = window.clone();
            let toc = toast_overlay.clone();
            let bn = banner.clone();
            let sel = selected_drive.clone();

            format_form.format_button.connect_clicked(move |_| {
                let drive = selected.borrow().clone();
                let Some(drive) = drive else {
                    return;
                };

                let label = match ff.validate_label() {
                    Ok(l) => l,
                    Err(msg) => {
                        let toast = toast_long_message(&msg);
                        toc.add_toast(toast);
                        return;
                    }
                };

                let dialog = libadwaita::MessageDialog::new(
                    Some(&wc),
                    Some("Confirmar formatação"),
                    Some(&format!(
                        "Todos os dados em {} ({}) serão apagados permanentemente. \
                         O dispositivo será reparticionado com uma única partição. Deseja continuar?",
                        drive.display_name(),
                        drive.device_path.display()
                    )),
                );
                dialog.add_response("cancel", "Cancelar");
                dialog.add_response("format", "Formatar");
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let object_path = drive.object_path.clone();
                let drive_object_path = drive.drive_object_path.clone();
                let fs_type = ff.get_fs_type();

                let dl2 = dl.clone();
                let ff2 = ff.clone();
                let toc2 = toc.clone();
                let bn2 = bn.clone();
                let sel2 = sel.clone();

                dialog.connect_response(None, move |dialog: &libadwaita::MessageDialog, response| {
                    if response != "format" {
                        return;
                    }

                    dialog.close();
                    ff2.set_sensitive(false);

                    let obj_path = object_path.clone();
                    let drive_obj_path = drive_object_path.clone();
                    let label_clone = label.clone();
                    let dl3 = dl2.clone();
                    let ff3 = ff2.clone();
                    let toc3 = toc2.clone();
                    let bn3 = bn2.clone();
                    let sel3 = sel2.clone();

                    glib::spawn_future_local(async move {
                        let result = gio::spawn_blocking(move || {
                            FormatService::format(
                                &obj_path,
                                &drive_obj_path,
                                fs_type,
                                label_clone.as_deref(),
                            )
                        })
                        .await;

                        ff3.set_sensitive(true);

                        match result {
                            Ok(Ok(())) => {
                                apply_refresh_outcome(
                                    &environment::refresh_drives(),
                                    &bn3,
                                    &dl3,
                                    &ff3,
                                    &sel3,
                                );
                                let toast =
                                    libadwaita::Toast::new("Formatação concluída com sucesso");
                                toc3.add_toast(toast);
                            }
                            Ok(Err(e)) => {
                                eprintln!("Erro ao formatar: {e:#}");
                                let user_msg = format_errors::user_message_for_format_error(&e);
                                let mut msg = user_msg.message;
                                if user_msg.show_install_hint {
                                    let hint = environment::install_hint_for_host();
                                    msg.push_str(&format!(
                                        "\n\n{}:\n{}",
                                        hint.label, hint.command
                                    ));
                                }
                                let toast = toast_long_message(&msg);
                                toc3.add_toast(toast);
                            }
                            Err(_) => {
                                let toast = toast_long_message(
                                    "Ocorreu um erro inesperado. Tente novamente.",
                                );
                                toc3.add_toast(toast);
                            }
                        }
                    });
                });
                dialog.present();
            });
        }

        clamp.set_child(Some(&content));
        toast_overlay.set_child(Some(&clamp));
        main_box.append(&toast_overlay);

        let version_label = gtk::Label::new(None);
        version_label.set_text(&format!("v{}", env!("CARGO_PKG_VERSION")));
        version_label.add_css_class("dim-label");
        version_label.set_widget_name("formatpen-version-label");
        version_label.set_margin_top(4);
        version_label.set_margin_bottom(8);
        version_label.set_halign(gtk::Align::Center);
        main_box.append(&version_label);

        setup_appimage_updates(app, &window, &toast_overlay, &update_btn);

        window.set_content(Some(&main_box));

        Self { inner: window }
    }
}
