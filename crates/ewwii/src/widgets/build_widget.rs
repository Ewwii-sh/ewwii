use anyhow::{Result, anyhow};
use codespan_reporting::diagnostic::Severity;
use ewwii_shared_util::{AttrName, Spanned};
use gtk::{
    gdk::prelude::Cast,
    prelude::{BoxExt, ContainerExt, WidgetExt},
    Orientation,
};
use itertools::Itertools;
use maplit::hashmap;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error_handling_ctx,
    dynval::DynVal,
    widgets::widget_definitions,
    gen_diagnostic,
};

use iirhai::widgetnode::WidgetNode;

pub struct BuilderArgs {
    pub window_defs: Rc<HashMap<String, WidgetDefinition>>,
}

// pass `EwwConfig::read_from_dir(&eww_paths).into()` here
pub fn build_gtk_widget(window_defs: Rc<HashMap<String, WindowDefinition>>) -> Result<gtk::Widget> {
    let def = window_defs.values().next().ok_or_else(|| anyhow!("No WindowDefinition passed to build_gtk_widget()"))?;

    let root_node = &def.root_widget;

    // build_gtk_widget_from_node(root_node)
    root_node
}

// fn build_gtk_widget_from_node(root_node: WidgetNode) {

// }