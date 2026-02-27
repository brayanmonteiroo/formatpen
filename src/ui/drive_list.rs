use crate::models::Drive;
use gtk::prelude::*;
use libadwaita::prelude::PreferencesGroupExt;
use std::cell::RefCell;
use std::rc::Rc;

/**
 * A lista de dispositivos removíveis é composta por um dropdown para selecionar o dispositivo a formatar.
 */
#[derive(Clone)]
pub struct DriveList {
    pub widget: gtk::Box,
    dropdown: gtk::DropDown,
    drives: Rc<RefCell<Vec<Drive>>>,
}

/**
 * Implementação da lista de dispositivos removíveis.
 */
impl DriveList {
    /**
     * Cria uma nova instância da lista de dispositivos removíveis.
     */
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let group = libadwaita::PreferencesGroup::new();
        group.set_title("Selecione seu dispositivo");

        let model = gtk::StringList::new(&[] as &[&str]);
        let dropdown = gtk::DropDown::new(Some(model), None::<gtk::Expression>);

        group.add(&dropdown);
        container.append(&group);

        Self {
            widget: container,
            dropdown,
            drives: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /**
     * Define a lista de dispositivos removíveis.
     */
    pub fn set_drives(&self, drives: Vec<Drive>) {
        *self.drives.borrow_mut() = drives.clone();

        let items: Vec<String> = drives
            .iter()
            .map(|d| {
                format!(
                    "{} - {} - {}",
                    d.display_name(),
                    d.device_path.display(),
                    d.formatted_size()
                )
            })
            .collect();

        let strings: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        let model = gtk::StringList::new(&strings);
        self.dropdown.set_model(Some(&model));

        if drives.len() == 1 {
            self.dropdown.set_selected(0);
        } else if drives.is_empty() {
            self.dropdown.set_selected(gtk::INVALID_LIST_POSITION);
        } else {
            self.dropdown.set_selected(0);
        }
    }

    /**
     * Conecta o sinal de mudança de seleção.
     */
    pub fn connect_selection_changed<F: Fn(Option<Drive>) + 'static>(&self, f: F) {
        let drives = self.drives.clone();
        let dropdown = self.dropdown.clone();
        self.dropdown.connect_selected_item_notify(move |_| {
            let idx = dropdown.selected();
            let selected = if idx != gtk::INVALID_LIST_POSITION {
                drives.borrow().get(idx as usize).cloned()
            } else {
                None
            };
            f(selected);
        });
    }

    /**
     * Retorna o dispositivo selecionado.
     */
    pub fn selected_drive(&self) -> Option<Drive> {
        let idx = self.dropdown.selected();
        if idx != gtk::INVALID_LIST_POSITION {
            self.drives.borrow().get(idx as usize).cloned()
        } else {
            None
        }
    }
}
