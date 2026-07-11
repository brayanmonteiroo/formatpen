use gtk::prelude::*;

pub fn pump_gtk_events() {
    while gtk::glib::MainContext::default().iteration(false) {}
}

pub fn find_widget_by_name<W>(root: &impl IsA<gtk::Widget>, name: &str) -> Option<W>
where
    W: IsA<gtk::Widget>,
{
    let root = root.as_ref();
    if root.widget_name() == name {
        return root.clone().downcast::<W>().ok();
    }

    let mut child = root.first_child();
    while let Some(c) = child {
        if let Some(found) = find_widget_by_name(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
