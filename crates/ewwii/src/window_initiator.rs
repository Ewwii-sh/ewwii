use anyhow::{anyhow, Result};
// use ewwii_shared_utils::{AttrName, VarName};
// use std::collections::HashMap;
// use rhai::Map;

use crate::{
    config::WindowDefinition,
    window::{
        backend_window_options::BackendWindowOptions,
        // coords::Coords,
        coords::NumWithUnit,
        monitor::MonitorIdentifier,
        window_definition::WindowStacking,
        window_geometry::{AnchorAlignment, AnchorPoint},
        // monitor,
        window_geometry::{Coords, WindowGeometry},
    },
    window_arguments::WindowArguments,
};

use ewwii_shared_utils::prop::{Property, PropertyMap};
use std::str::FromStr;

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
    pub fn new(window_def: &WindowDefinition, args: &WindowArguments) -> Result<Self> {
        let properties = &window_def.props;
        let geometry = match properties.get("geometry") {
            Some(val) => Some(parse_geometry(val, args, true)?),
            // Some(geo) => Some(geo.eval(&vars)?.override_if_given(args.anchor, args.pos, args.size)),
            None => None,
        };
        let monitor = args.monitor.clone().or_else(|| {
            properties
                .get("monitor")?
                .clone()
                .as_int()
                .map(|n| MonitorIdentifier::Numeric(n as i32))
        });
        Ok(WindowInitiator {
            backend_options: window_def.backend_options.eval(properties.clone())?,
            geometry,
            monitor,
            name: window_def.name.clone(),
            resizable: properties.get("resizable").and_then(|d| d.as_bool()).unwrap_or(true),
            stacking: match properties.get("stacking") {
                Some(d) => WindowStacking::from_str(&d.as_str().unwrap_or_default())?,
                None => WindowStacking::Foreground, // or error
            },
        })
    }

    // pub fn get_scoped_vars(&self) -> HashMap<AttrName, DynVal> {
    //     self.local_variables.iter().map(|(k, v)| (AttrName::from(k.clone()), v.clone())).collect()
    // }
}

fn parse_geometry(
    val: &Property,
    args: &WindowArguments,
    override_geom: bool,
) -> Result<WindowGeometry> {
    let map = val.as_map().unwrap();

    let anchor = map
        .get("anchor")
        .and_then(|v| v.as_str())
        .map(|dyn_value| anchor_point_from_str(&dyn_value))
        .transpose()?;

    let mut geom = WindowGeometry {
        offset: get_coords_from_map(&map, "x", "y")?,
        size: get_coords_from_map(&map, "width", "height")?,
        anchor_point: anchor
            .unwrap_or(AnchorPoint { x: AnchorAlignment::CENTER, y: AnchorAlignment::START }),
    };

    if override_geom {
        geom = geom.override_with(
            args.anchor,
            // both are converted into window_geometry::Coords from coords::Coords
            args.pos.map(Into::into),
            args.size.map(Into::into),
        );
    }

    Ok(geom)
}

fn get_coords_from_map(map: &PropertyMap, x_key: &str, y_key: &str) -> Result<Coords> {
    let key1 = map
        .get(x_key)
        .and_then(|v| v.as_str())
        .map(|s| NumWithUnit::from_str(&s))
        .transpose()?
        .unwrap_or_else(NumWithUnit::default);

    let key2 = map
        .get(y_key)
        .and_then(|v| v.as_str())
        .map(|s| NumWithUnit::from_str(&s))
        .transpose()?
        .unwrap_or_else(NumWithUnit::default);

    Ok(Coords { x: key1, y: key2 })
}

fn anchor_point_from_str(s: &str) -> Result<AnchorPoint> {
    let binding = s.trim().to_lowercase();
    let parts: Vec<_> = binding.split_whitespace().collect();

    match parts.as_slice() {
        [single] => {
            // Apply to both x and y
            let alignment = AnchorAlignment::from_x_alignment(single)
                .or_else(|_| AnchorAlignment::from_y_alignment(single))?;
            Ok(AnchorPoint { x: alignment, y: alignment })
        }
        [y_part, x_part] => {
            let y = AnchorAlignment::from_y_alignment(y_part)?;
            let x = AnchorAlignment::from_x_alignment(x_part)?;
            Ok(AnchorPoint { x, y })
        }
        _ => Err(anyhow!("Expected 1 or 2 words like 'center' or 'top left'")),
    }
}
