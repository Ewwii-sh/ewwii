//! Module exposing structures and types from the
//! Widget rendering and definition backend in ewwii.

#[cfg(feature = "include-gtk4")]
mod gtk4_included {
    use gtk4::Widget as GtkWidget;
    use std::collections::HashMap;

    /// _(include-gtk4)_ A representation of widget registry which holds all the
    /// information needed for the dynamic runtime engine in ewwii.
    ///
    /// Not every change in this structure will be represented in the
    /// original WidgetRegistry in ewwii. Only the change on gtk4::Widget
    /// is reflected back.
    pub struct WidgetRegistryRepr<'a> {
        pub widgets: HashMap<u64, &'a mut GtkWidget>,
    }
}

#[cfg(feature = "include-gtk4")]
pub use gtk4_included::*;