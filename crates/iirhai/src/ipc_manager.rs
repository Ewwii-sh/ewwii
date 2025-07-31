/*
    This file is responsible for converting the internal widgetnode tree to json.
    It needs to be converted to JSON to be send back via the ipc.
    It is also used to manage the internal ipc of iirhai.
*/

use crate::widgetnode::WidgetNode;
use rhai::{Array, Dynamic, Map};
use serde_json::{json, Value};

pub struct IpcManager {
    widgetnode: WidgetNode,
}

impl IpcManager {
    pub fn new(widgetnode: WidgetNode) -> Self {
        IpcManager { widgetnode }
    }
    pub fn transpile_to_json(&self) -> serde_json::Value {
        widget_to_json(&self.widgetnode)
    }

    pub fn watch_changes() {
        // TODO: implment the watchman
    }
}

fn widget_to_json(widget: &WidgetNode) -> Value {
    match widget {
        WidgetNode::Box { props, children } => json!({
            "type": "box",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
        WidgetNode::Label { props } => json!({
            "type": "label",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
        }),
        WidgetNode::CenterBox { props, children } => json!({
            "type": "center_box",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
        WidgetNode::Button { props, children } => json!({
            "type": "button",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
        WidgetNode::Image { props } => json!({
            "type": "image",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Input { props } => json!({
            "type": "input",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Progress { props } => json!({
            "type": "progress",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Spacer { props } => json!({
            "type": "spacer",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Slider { props } => json!({
            "type": "slider",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Revealer { props, children } => json!({
            "type": "revealer",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
        WidgetNode::Scroll { props, children } => json!({
            "type": "scroll",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
        WidgetNode::Calendar { props } => json!({
            "type": "calendar",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Graph { props } => json!({
            "type": "graph",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Include(string) => json!({
            "type": "include",
            "string": string.clone(),
        }),
        WidgetNode::DefStyle(string) => json!({
            "type": "defstyle",
            "string": string.clone(),
        }),
        WidgetNode::EventBox { props, children } => json!({
            "type": "event_box",
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),

        WidgetNode::DefWindow { name, props, node } => json!({
            "type": "window",
            "name": name.clone(),
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>(),
            "node": widget_to_json(node),
        }),
        WidgetNode::Poll { var, props } => json!({
            "type": "poll",
            "var": var.clone(),
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Listen { var, props } => json!({
            "type": "listen",
            "var": var.clone(),
            "props": props.iter().map(|(k, v)| (k.to_string(), dynamic_to_json(v))).collect::<serde_json::Map<_, _>>()
        }),
        WidgetNode::Enter(children) => json!({
            "type": "enter",
            "children": children.iter().map(widget_to_json).collect::<Vec<_>>()
        }),
    }
}

// helpers
fn dynamic_to_json(value: &Dynamic) -> Value {
    if let Some(i) = value.as_int().ok() {
        json!(i)
    } else if let Some(f) = value.as_float().ok() {
        json!(f)
    } else if let Some(b) = value.as_bool().ok() {
        json!(b)
    } else if let Some(s) = value.clone().try_cast::<String>() {
        json!(s)
    } else if value.is::<Map>() {
        let map = value.clone_cast::<Map>();
        let converted = map.into_iter().map(|(k, v)| (k.to_string(), dynamic_to_json(&v))).collect::<serde_json::Map<_, _>>();
        Value::Object(converted)
    } else if value.is::<Array>() {
        let array = value.clone_cast::<Array>();
        let converted = array.iter().map(dynamic_to_json).collect::<Vec<_>>();
        Value::Array(converted)
    } else {
        Value::Null
    }
}
