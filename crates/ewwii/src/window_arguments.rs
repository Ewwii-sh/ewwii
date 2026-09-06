use crate::{
    window::coords::Coords, window::monitor::MonitorIdentifier,
    window::window_geometry::AnchorPoint,
};
use anyhow::Result;
use std::collections::HashMap;
use ewwii_shared_utils::prop_utils::parse_duration_str;

/// This stores the arguments given in the command line to create a window
/// While creating a window, we combine this with information from the
/// [`WindowDefinition`] to create a [WindowInitiator](`crate::window_initiator::WindowInitiator`), which stores all the
/// information required to start a window
#[derive(Debug, Clone)]
pub struct WindowArguments {
    /// Name of the window as defined in the eww config
    pub window_name: String,
    /// Instance ID of the window
    pub instance_id: String,
    pub anchor: Option<AnchorPoint>,
    pub duration: Option<std::time::Duration>,
    pub monitor: Option<MonitorIdentifier>,
    pub pos: Option<Coords>,
    pub size: Option<Coords>,
}

impl WindowArguments {
    pub fn new_from_args(id: String, config_name: String, args: HashMap<String, String>) -> Result<Self> {
        let pos_str = args.get("pos");
        let size_str = args.get("size");
        let screen_str = args.get("screen");
        let anchor_str = args.get("anchor");
        let duration_str = args.get("duration");

        let pos = pos_str.and_then(|s| s.parse::<Coords>().ok());
        let size = size_str.and_then(|s| s.parse::<Coords>().ok());
        let monitor = screen_str.and_then(|s| s.parse::<MonitorIdentifier>().ok());
        let anchor = anchor_str.and_then(|s| s.parse::<AnchorPoint>().ok());
        let duration = duration_str.and_then(|s| parse_duration_str(s));

        let initiator = WindowArguments {
            window_name: config_name,
            instance_id: id,
            pos,
            size,
            monitor,
            anchor,
            duration
        };

        Ok(initiator)
    }

    // /// Return a hashmap of all arguments the window was passed and expected, returning
    // /// an error in case required arguments are missing or unexpected arguments are passed.
    // pub fn get_local_window_variables(&self, window_def: &WindowDefinition) -> Result<HashMap<VarName, DynVal>> {
    //     let expected_args: HashSet<&String> = window_def.expected_args.iter().map(|x| &x.name.0).collect();
    //     let mut local_variables: HashMap<VarName, DynVal> = HashMap::new();

    //     // Ensure that the arguments passed to the window that are already interpreted by eww (id, screen)
    //     // are set to the correct values
    //     if expected_args.contains(&String::from("id")) {
    //         local_variables.insert(VarName::from("id"), DynVal::from(self.instance_id.clone()));
    //     }
    //     if self.monitor.is_some() && expected_args.contains(&String::from("screen")) {
    //         let mon_dyn = DynVal::from(&self.monitor.clone().unwrap());
    //         local_variables.insert(VarName::from("screen"), mon_dyn);
    //     }

    //     local_variables.extend(self.args.clone());

    //     for attr in &window_def.expected_args {
    //         let name = VarName::from(attr.name.clone());
    //         if !local_variables.contains_key(&name) && !attr.optional {
    //             bail!("Error, missing argument '{}' when creating window with id '{}'", attr.name, self.instance_id);
    //         }
    //     }

    //     if local_variables.len() != window_def.expected_args.len() {
    //         let unexpected_vars: Vec<_> = local_variables.keys().filter(|&n| !expected_args.contains(&n.0)).cloned().collect();
    //         bail!(
    //             "variables {} unexpectedly defined when creating window with id '{}'",
    //             unexpected_vars.join(", "),
    //             self.instance_id,
    //         );
    //     }

    //     Ok(local_variables)
    // }
}
