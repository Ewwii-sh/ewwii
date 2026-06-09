use gtk4::glib;
use gtk4::pango;
use gtk4::glib::Properties;
use glib::Object;
use gtk4::subclass::prelude::*;
use gtk4::prelude::*;
use std::cell::{Cell, OnceCell, RefCell};
use crate::util;

mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::EwwiiLabel)]
    pub struct EwwiiLabel {
        pub inner_label: OnceCell<gtk4::Label>,

        #[property(get, set = Self::set_text, nullable)]
        text: RefCell<Option<String>>,

        #[property(get, set = Self::set_markup, nullable)]
        markup: RefCell<Option<String>>,

        #[property(get, set = Self::set_max_chars)]
        max_chars: Cell<i32>,

        #[property(get, set = Self::set_ellipsize)]
        ellipsize: Cell<bool>,

        /// Ellipsize/truncate from the start instead of the end
        #[property(get, set = Self::set_ellipsize_start)]
        ellipsize_start: Cell<bool>,

        #[property(get, set = Self::set_unescape)]
        unescape: Cell<bool>,

        #[property(get, set = Self::set_unindent)]
        unindent: Cell<bool>,
    }

    impl Default for EwwiiLabel {
        fn default() -> Self {
            Self {
                max_chars: Cell::new(-1),
                inner_label: OnceCell::new(),
                text: RefCell::new(None),
                markup: RefCell::new(None),
                ellipsize: Cell::new(false),
                ellipsize_start: Cell::new(false),
                unescape: Cell::new(false),
                unindent: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EwwiiLabel {
        const NAME: &'static str = "EwwiiLabel";
        type Type = super::EwwiiLabel;
        type ParentType = gtk4::Widget;
    }

    #[glib::derived_properties]
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

    impl EwwiiLabel {
        fn set_text(&self, value: Option<String>) {
            *self.text.borrow_mut() = value;
            *self.markup.borrow_mut() = None;
            self.sync();
        }

        fn set_markup(&self, value: Option<String>) {
            *self.markup.borrow_mut() = value;
            *self.text.borrow_mut() = None;
            self.sync();
        }

        fn set_max_chars(&self, value: i32) {
            self.max_chars.set(value);
            self.sync();
        }

        fn set_ellipsize(&self, value: bool) {
            self.ellipsize.set(value);
            self.sync();
        }

        fn set_ellipsize_start(&self, value: bool) {
            self.ellipsize_start.set(value);
            self.sync();
        }

        fn set_unescape(&self, value: bool) {
            self.unescape.set(value);
            self.sync();
        }

        fn set_unindent(&self, value: bool) {
            self.unindent.set(value);
            self.sync();
        }

        fn sync(&self) {
            let Some(label) = self.inner_label.get() else { return };

            let max_chars = self.max_chars.get();
            let ellipsize = self.ellipsize.get();
            let ellipsize_start = self.ellipsize_start.get();

            if let Some(text) = self.text.borrow().as_deref() {
                let content = if !ellipsize && max_chars != -1 {
                    label.set_ellipsize(pango::EllipsizeMode::None);
                    let limit = max_chars as usize;
                    let char_count = text.chars().count();
                    if char_count > limit {
                        if ellipsize_start {
                            text.chars().skip(char_count - limit).collect()
                        } else {
                            text.chars().take(limit).collect()
                        }
                    } else {
                        text.to_string()
                    }
                } else {
                    label.set_max_width_chars(max_chars);
                    label.set_width_chars(-1);
                    label.set_ellipsize(if ellipsize {
                        if ellipsize_start {
                            pango::EllipsizeMode::Start
                        } else {
                            pango::EllipsizeMode::End
                        }
                    } else {
                        pango::EllipsizeMode::None
                    });
                    text.to_string()
                };


                let content = if self.unescape.get() {
                    match unescape::unescape(&content) {
                        Some(u) => u,
                        None => {
                            log::error!("EwwiiLabel: failed to unescape text");
                            content
                        }
                    }
                } else {
                    content
                };

                let content = if self.unindent.get() {
                    util::unindent(&content)
                } else {
                    content
                };

                label.set_text(&content);

            } else if let Some(markup) = self.markup.borrow().as_deref() {
                label.set_ellipsize(if ellipsize {
                    if ellipsize_start {
                        pango::EllipsizeMode::Start
                    } else {
                        pango::EllipsizeMode::End
                    }
                } else {
                    pango::EllipsizeMode::None
                });
                label.set_max_width_chars(max_chars);
                label.set_markup(markup);
            }
        }
    }
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

    pub fn set_wrap(&self, value: bool) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_wrap(value);
        }
    }

    pub fn set_xalign(&self, value: f32) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_xalign(value);
        }
    }

    pub fn set_yalign(&self, value: f32) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_yalign(value);
        }
    }

    pub fn set_lines(&self, value: i32) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_lines(value);
        }
    }

    pub fn set_justify(&self, value: u32) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_justify(match value {
                1 => gtk4::Justification::Right,
                2 => gtk4::Justification::Center,
                3 => gtk4::Justification::Fill,
                _ => gtk4::Justification::Left,
            });
        }
    }

    pub fn set_wrap_mode(&self, value: u32) {
        if let Some(label) = self.imp().inner_label.get() {
            label.set_wrap_mode(match value {
                1 => pango::WrapMode::Char,
                2 => pango::WrapMode::WordChar,
                _ => pango::WrapMode::Word,
            });
        }
    }
}

impl Default for EwwiiLabel {
    fn default() -> Self {
        Self::new()
    }
}
