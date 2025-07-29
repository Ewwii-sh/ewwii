use anyhow::Result;
use ewwii_shared_util::{AttrName, VarName};
use std::collections::HashMap;
use rhai::Map;

use crate::{
    window_arguments::WindowArguments,
    config::WindowDefinition,
    window::{
        monitor,
        window_geometry::WindowGeometry,
        coords::Coords,
        monitor::MonitorIdentifier,
        backend_window_options::BackendWindowOptions,
        window_geometry::AnchorPoint,
    },
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
        let monitor = args.monitor.clone().or_else(|| {
            window_def.props.get("monitor")?.clone().try_cast::<i64>().map(|n| MonitorIdentifier::Index(n as usize))
        });
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

fn parse_geometry(val: &rhai::Dynamic, args: &WindowArguments, override_geom: bool) -> Result<WindowGeometry> {
    let map = val.clone().cast::<rhai::Map>();
    let mut geom = WindowGeometry {
        pos: Some(Coords::from_map(&map)?),
        size: Some(Size::from_map(&map)?),
        anchor: Some(AnchorPoint::TopLeft),
    };

    if override_geom {
        // You apply CLI args if passed:
        geom = geom.override_if_given(args.anchor, args.pos, args.size);
    }

    Ok(geom)
}
