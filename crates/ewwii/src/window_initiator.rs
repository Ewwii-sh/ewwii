use anyhow::Result;
use ewwii_shared_util::{AttrName, VarName};
use std::collections::HashMap;
use rhai::Map;

use crate::{
    window_arguments::WindowArguments,
    config::WindowDefinition,
    window::monitor,
};

/// This stores all the information required to create a window and is created
/// via combining information from the [`WindowDefinition`] and the [`WindowInitiator`]
#[derive(Debug, Clone)]
pub struct WindowInitiator {
    pub backend_options: BackendWindowOptions,
    pub geometry: Option<WindowGeometry>,
    pub monitor: Option<MonitorIdentifier>,
    pub name: String,
    pub resizable: bool,
    pub stacking: WindowStacking,
}

impl WindowInitiator {
    // pass `EwwConfig::read_from_dir(&eww_paths).windows.into()` here
    pub fn new(window_def: &WindowDefinition, args: &WindowArguments) -> Result<Self> {
        let geometry = match &window_def.props.get("geometry") {
            Some(val) => Some(parse_geometry(val, args, true)?),
            // Some(geo) => Some(geo.eval(&vars)?.override_if_given(args.anchor, args.pos, args.size)),
            None => None,
        };
        let monitor = if args.monitor.is_none() { window_def.eval_monitor(&vars)? } else { args.monitor.clone() };
        Ok(WindowInitiator {
            backend_options: window_def.backend_options.eval(&vars)?,
            geometry,
            monitor,
            name: window_def.name.clone(),
            resizable: window_def.eval_resizable(&vars)?,
            stacking: window_def.eval_stacking(&vars)?,
        })
    }

    // pub fn get_scoped_vars(&self) -> HashMap<AttrName, DynVal> {
    //     self.local_variables.iter().map(|(k, v)| (AttrName::from(k.clone()), v.clone())).collect()
    // }
}

fn parse_geometry(val: rhai::Map, args: &WindowArguments, override_str: bool) {
    todo!("Implement parse_geometry");
}