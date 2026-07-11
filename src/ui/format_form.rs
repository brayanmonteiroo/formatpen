use crate::label_validation;
use crate::services::FilesystemType;
use gtk::prelude::*;
use libadwaita::prelude::*;

/**
 * O formulário de formatação é composto por um dropdown para selecionar o tipo de sistema de arquivos e um entry para o nome do volume.
 */
#[derive(Clone)]
pub struct FormatForm {
    pub widget: gtk::Box,
    fs_type_dropdown: gtk::DropDown,
    pub label_entry: gtk::Entry,
    #[allow(dead_code)]
    label_hint: gtk::Label,
    pub format_button: gtk::Button,
}

/**
 * Implementação do formulário de formatação.
 */
impl FormatForm {
    /**
     * Cria uma nova instância do formulário de formatação.
     */
    pub fn new() -> Self {
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 12);

        let fs_group = libadwaita::PreferencesGroup::new();
        fs_group.set_title("Tipo de sistema de arquivos");

        let items: Vec<&str> = FilesystemType::all()
            .iter()
            .map(FilesystemType::display_name)
            .collect();
        let model = gtk::StringList::new(&items);
        let dropdown = gtk::DropDown::new(Some(model), None::<gtk::Expression>);
        dropdown.set_selected(0);
        fs_group.add(&dropdown);
        main_box.append(&fs_group);

        let label_group = libadwaita::PreferencesGroup::new();
        label_group.set_title("Nome do volume");
        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("Ex: MeuPen"));
        entry.set_hexpand(true);
        entry.set_max_length(FilesystemType::Fat32.max_label_length() as i32);
        label_group.add(&entry);

        let label_hint = gtk::Label::new(None);
        label_hint.set_xalign(0.0);
        label_hint.add_css_class("dim-label");
        label_hint.set_margin_top(4);
        label_hint.set_text(&label_validation::label_hint_for(FilesystemType::Fat32));
        label_group.add(&label_hint);

        main_box.append(&label_group);

        {
            let entry_clone = entry.clone();
            let hint_clone = label_hint.clone();
            dropdown.connect_selected_item_notify(move |dd| {
                let idx = dd.selected();
                let fs = FilesystemType::all()
                    .get(idx as usize)
                    .copied()
                    .unwrap_or(FilesystemType::Fat32);

                entry_clone.set_max_length(fs.max_label_length() as i32);
                let truncated = label_validation::truncate_to_max(&entry_clone.text(), fs);
                entry_clone.set_text(&truncated);
                hint_clone.set_text(&label_validation::label_hint_for(fs));
            });
        }

        let format_btn = gtk::Button::new();
        format_btn.set_label("Formatar");
        format_btn.add_css_class("suggested-action");
        format_btn.set_hexpand(false);
        format_btn.set_halign(gtk::Align::Center);
        format_btn.set_margin_top(8);
        main_box.append(&format_btn);

        Self {
            widget: main_box,
            fs_type_dropdown: dropdown,
            label_entry: entry,
            label_hint,
            format_button: format_btn,
        }
    }

    /**
     * Retorna o tipo de sistema de arquivos selecionado.
     */
    pub fn get_fs_type(&self) -> FilesystemType {
        let idx = self.fs_type_dropdown.selected();
        FilesystemType::all()
            .get(idx as usize)
            .copied()
            .unwrap_or(FilesystemType::Fat32)
    }

    /**
     * Retorna o nome do volume selecionado (apenas se válido).
     * Valida caracteres proibidos e tamanho máximo.
     */
    pub fn validate_label(&self) -> Result<Option<String>, String> {
        label_validation::validate_volume_label(&self.label_entry.text(), self.get_fs_type())
    }

    /**
     * Define se o formulário deve ser sensível ou não.
     */
    pub fn set_sensitive(&self, sensitive: bool) {
        self.fs_type_dropdown.set_sensitive(sensitive);
        self.label_entry.set_sensitive(sensitive);
        self.format_button.set_sensitive(sensitive);
    }
}
