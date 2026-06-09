use gtk4::glib;
use glib::Object;
use gtk4::subclass::prelude::*;
use gtk4::prelude::*;
use std::cell::OnceCell;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct EwwiiLabel {
        inner_label: OnceCell<gtk4::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EwwiiLabel {
        const NAME: &'static str = "EwwiiLabel";
        type Type = super::EwwiiLabel;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for EwwiiLabel {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk4::BinLayout::new()));
            let label = gtk4::Label::new(None);
            label.set_parent(&*self.obj());
            self.inner_label.set(label).unwrap();
        }

        fn dispose(&self) {
            if let Some(label) = self.inner_label.get() {
                label.unparent();
            }
        }
    }
    impl WidgetImpl for EwwiiLabel {}
}

glib::wrapper! {
    pub struct EwwiiLabel(ObjectSubclass<imp::EwwiiLabel>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl EwwiiLabel {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

