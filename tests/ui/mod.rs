mod helpers;

use formatpen::ui::{DriveList, FormatForm};
use formatpen::models::Drive;
use gtk::prelude::*;
use helpers::{find_widget_by_name, pump_gtk_events};

fn init_ui() {
    let _ = libadwaita::init();
}

#[gtk::test]
fn format_form_fs_change_atualiza_hint_e_trunca() {
    init_ui();
    let form = FormatForm::new();

    let dropdown: gtk::DropDown =
        find_widget_by_name(&form.widget, "formatpen-fs-dropdown").expect("dropdown FS");
    dropdown.set_selected(1); // exFAT
    pump_gtk_events();

    let hint: gtk::Label =
        find_widget_by_name(&form.widget, "formatpen-label-hint").expect("hint label");
    let hint_text = hint.text().to_string();
    assert!(hint_text.contains("exFAT"));
    assert!(hint_text.contains("11"));

    let entry: gtk::Entry =
        find_widget_by_name(&form.widget, "formatpen-label-entry").expect("label entry");
    entry.set_text("123456789012");
    pump_gtk_events();
    assert_eq!(entry.text().chars().count(), 11);

    dropdown.set_selected(0); // FAT32
    pump_gtk_events();
    let hint_fat = hint.text().to_string();
    assert!(hint_fat.contains("FAT32"));
}

#[gtk::test]
fn format_form_validate_rejeita_caracteres_proibidos() {
    init_ui();
    let form = FormatForm::new();
    form.label_entry.set_text("abc/def");
    pump_gtk_events();

    let err = form.validate_label().unwrap_err();
    assert!(err.contains("caractere"));
}

#[gtk::test]
fn format_form_validate_aceita_label_valido() {
    init_ui();
    let form = FormatForm::new();
    form.label_entry.set_text("MeuPen");
    pump_gtk_events();

    assert_eq!(
        form.validate_label().unwrap(),
        Some("MeuPen".to_string())
    );
}

#[gtk::test]
fn drive_list_empty_mostra_status() {
    init_ui();
    let list = DriveList::new();
    list.set_drives(vec![]);
    list.set_status_message(Some("Nenhum dispositivo removível encontrado."));
    pump_gtk_events();

    let status: gtk::Label =
        find_widget_by_name(&list.widget, "formatpen-drive-status").expect("status");
    assert!(status.is_visible());
    assert!(status.text().contains("Nenhum dispositivo"));

    let dropdown: gtk::DropDown =
        find_widget_by_name(&list.widget, "formatpen-drive-dropdown").expect("dropdown");
    assert_eq!(dropdown.selected(), gtk::INVALID_LIST_POSITION);
}

#[gtk::test]
fn drive_list_one_drive_seleciona_automaticamente() {
    init_ui();
    let list = DriveList::new();
    let drive = Drive {
        path: "sde".into(),
        device_path: "/dev/sde".into(),
        size: 16_000_000_000,
        label: Some("MEUPEN".into()),
        model: Some("Kingston DataTraveler".into()),
        id_type: None,
        is_removable: true,
        mount_points: vec![],
        object_path: "/org/freedesktop/UDisks2/block_devices/sde".into(),
        drive_object_path: "/org/freedesktop/UDisks2/drives/Kingston".into(),
    };

    list.set_drives(vec![drive.clone()]);
    list.set_status_message(None);
    pump_gtk_events();

    let selected = list.selected_drive().expect("drive selecionado");
    assert_eq!(selected.path, "sde");
    assert_eq!(selected.label.as_deref(), Some("MEUPEN"));
}
