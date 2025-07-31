use anyhow::{Result, anyhow};
// use ewwii_shared_util::{AttrName, VarName};
// use std::collections::HashMap;
// use rhai::Map;

use crate::{
    window_arguments::WindowArguments,
    config::WindowDefinition,
    window::{
        // monitor,
        window_geometry::WindowGeometry,
        coords::Coords,
        coords::NumWithUnit,
        monitor::MonitorIdentifier,
        backend_window_options::BackendWindowOptions,
        window_geometry::{AnchorPoint, AnchorAlignment},
        window_definition::{WindowStacking, EnumParseError},
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
    pub fn new(window_def: &WindowDefinition, args: &WindowArguments) -> Result<Self> {
        let properties = window_def.props.clone();
        let geometry = match properties.get("geometry") {
            Some(val) => Some(parse_geometry(val, args, true)?),
            // Some(geo) => Some(geo.eval(&vars)?.override_if_given(args.anchor, args.pos, args.size)),
            None => None,
        };
        let monitor = args.monitor.clone().or_else(|| {
            properties.get("monitor")?.clone().try_cast::<i64>().map(|n| MonitorIdentifier::Index(n as usize))
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

    let anchor = map.get("anchor")
        .map(|dyn_value| anchor_point_from_str(&dyn_value.to_string()))
        .transpose()?;


    let mut geom = WindowGeometry {
        offset: get_coords_from_map(&map, "x", "y")?,
        size: get_coords_from_map(&map, "width", "height")?,
        anchor_point: anchor.unwrap_or(
            AnchorPoint {
                x: AnchorAlignment::CENTER, 
                y: AnchorAlignment::START
            }),
    };

    if override_geom {
        geom = geom.override_if_given(args.anchor, args.pos, args.size);
    }

    Ok(geom)
}

fn get_coords_from_map(map: &rhai::Map, x_key: &str, y_key: &str, ) -> Result<Coords> {
    let raw_key1 = map.get(x_key)
        .ok_or_else(|| anyhow!("Missing field {}", x_key))?
        .into_string().map_err(|_| anyhow!("Expected string for field {}", x_key))?;

    let raw_key2 = map.get(y_key)
        .ok_or_else(|| anyhow!("Missing field {}", y_key))?
        .into_string().map_err(|_| anyhow!("Expected string for field {}", y_key))?;

    let key1 = NumWithUnit::from_str(&raw_key1)?;
    let key2 = NumWithUnit::from_str(&raw_key2)?;
    
    Ok(Coords { x: key1, y: key2 })
}

fn anchor_point_from_str(s: &str) -> Result<AnchorPoint, EnumParseError> {
    let parts: Vec<_> = s.trim().to_lowercase().split_whitespace().collect();

    match parts.as_slice() {
        [single] => {
            // Apply to both x and y
            let alignment = AnchorAlignment::from_x_alignment(single)
                .or_else(|_| AnchorAlignment::from_y_alignment(single))?;
            Ok(AnchorPoint {
                x: alignment,
                y: alignment,
            })
        }
        [y_part, x_part] => {
            let y = AnchorAlignment::from_y_alignment(y_part)?;
            let x = AnchorAlignment::from_x_alignment(x_part)?;
            Ok(AnchorPoint { x, y })
        }
        _ => Err(EnumParseError::UnknownValue {
            expected: "1 or 2 words like 'center' or 'top left'".to_string(),
            got: s.to_string(),
        }),
    }
}
