//! Graph widget with time-series visualization
//!
//! The graph renders data points in a 2D canvas with time on the horizontal axis
//! and values on the vertical axis by default (non-vertical mode).
//!
//! Canvas Layout (default horizontal orientation, no flipping):
//!
//!     value ↑
//!           │
//!       max ┼──────────────┐ (past, max)
//!           │              │
//!           │              │
//!           │    Graph     │
//!           │    Area      │
//!           │              │
//!       min ┼──────────────┘ (past, min)
//!           └──────────────┴───→ time
//!          (past)          (now)
//!
//! Key coordinates in widget space (after margins):
//!   - (0, 0)     :     Top-left corner     (past time, max value)
//!   - (width, 0) :     Top-right corner    (current time, max value)
//!   - (0, height):     Bottom-left corner  (past time, min value)
//!   - (width, height): Bottom-right corner (current time, min value)
//!
//! Time flows from left (past) to right (present).
//! Most recent data points appear at the right edge.
//! Older points scroll leftward as time progresses.
//!
//! The `time-range` property controls how many milliseconds of history are visible.
//! Points older than `now - time-range` are automatically pruned.
//!
//! The graph supports multiple render types (Line, Fill, Step variants)
//! and can be flipped/rotated via properties.

use gtk4::glib::property::PropertySet;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

mod imp {
    use super::*;

    const DEFAULT_VALUE: f64 = 0.0;
    const DEFAULT_THICKNESS: f64 = 1.0;
    const DEFAULT_MIN: f64 = 0.0;
    const DEFAULT_MAX: f64 = 1.0;
    const DEFAULT_DYNAMIC: bool = true;
    const DEFAULT_TIME_RANGE: u32 = 10_000; // ms
    const DEFAULT_FLIP_X: bool = false;
    const DEFAULT_FLIP_Y: bool = false;
    const DEFAULT_VERTICAL: bool = false;
    const DEFAULT_ANIMATE: bool = true;

    pub struct Graph {
        pub value: Cell<f64>,
        pub thickness: Cell<f64>,
        pub line_style: Cell<LineStyle>,
        pub min: Cell<f64>,
        pub max: Cell<f64>,
        pub dynamic: Cell<bool>,
        pub time_range: Cell<u32>,
        pub flip_x: Cell<bool>,
        pub flip_y: Cell<bool>,
        pub vertical: Cell<bool>,
        pub render_type: Cell<RenderType>,

        // Runtime state
        history: RefCell<VecDeque<(Instant, f64)>>,
        last_updated_at: RefCell<Instant>,
        tick_id: RefCell<Option<gtk4::TickCallbackId>>,
        min_value_cached: Cell<Option<f64>>,
        max_value_cached: Cell<Option<f64>>,
        has_received_value: Cell<bool>,

        // Cached path (Geometry)
        cached_path: RefCell<Option<gsk::Path>>,
        // The "anchor" time used to build the path (fixed time reference)
        path_anchor_time: Cell<Instant>,
        // Size when the path was built (for invalidation)
        cached_path_size: Cell<(f32, f32)>,
    }

    impl Default for Graph {
        fn default() -> Self {
            Self {
                value: Cell::new(DEFAULT_VALUE),
                thickness: Cell::new(DEFAULT_THICKNESS),
                line_style: Cell::new(LineStyle::default()),
                min: Cell::new(DEFAULT_MIN),
                max: Cell::new(DEFAULT_MAX),
                dynamic: Cell::new(DEFAULT_DYNAMIC),
                time_range: Cell::new(DEFAULT_TIME_RANGE),
                flip_x: Cell::new(DEFAULT_FLIP_X),
                flip_y: Cell::new(DEFAULT_FLIP_Y),
                vertical: Cell::new(DEFAULT_VERTICAL),
                render_type: Cell::new(RenderType::default()),

                history: RefCell::new(VecDeque::new()),
                last_updated_at: RefCell::new(Instant::now()),
                tick_id: RefCell::new(None),
                min_value_cached: Cell::new(None),
                max_value_cached: Cell::new(None),
                has_received_value: Cell::new(false),

                cached_path: RefCell::new(None),
                path_anchor_time: Cell::new(Instant::now()),
                cached_path_size: Cell::new((0.0, 0.0)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Graph {
        const NAME: &'static str = "EwwiiGraph";
        type Type = super::Graph;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Graph {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_css_class("graph");

            if DEFAULT_ANIMATE {
                self.set_animate(&obj, true);
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::builder("value")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(DEFAULT_VALUE)
                        .build(),
                    glib::ParamSpecDouble::builder("thickness")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(DEFAULT_THICKNESS)
                        .build(),
                    glib::ParamSpecEnum::builder::<LineStyle>("line-style")
                        .default_value(LineStyle::default())
                        .build(),
                    glib::ParamSpecDouble::builder("min")
                        .minimum(f64::MIN)
                        .maximum(f64::MAX)
                        .default_value(DEFAULT_MIN)
                        .build(),
                    glib::ParamSpecDouble::builder("max")
                        .minimum(f64::MIN)
                        .maximum(f64::MAX)
                        .default_value(DEFAULT_MAX)
                        .build(),
                    glib::ParamSpecBoolean::builder("dynamic")
                        .default_value(DEFAULT_DYNAMIC)
                        .build(),
                    glib::ParamSpecUInt::builder("time-range")
                        .minimum(0)
                        .maximum(u32::MAX)
                        .default_value(DEFAULT_TIME_RANGE)
                        .build(),
                    glib::ParamSpecBoolean::builder("flip-x").default_value(DEFAULT_FLIP_X).build(),
                    glib::ParamSpecBoolean::builder("flip-y").default_value(DEFAULT_FLIP_Y).build(),
                    glib::ParamSpecBoolean::builder("vertical")
                        .default_value(DEFAULT_VERTICAL)
                        .build(),
                    glib::ParamSpecEnum::builder::<RenderType>("type")
                        .default_value(RenderType::default())
                        .build(),
                    glib::ParamSpecBoolean::builder("animate")
                        .default_value(DEFAULT_ANIMATE)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let needs_path_rebuild = match pspec.name() {
                "value" => {
                    let v: f64 = value.get().unwrap();
                    self.value.set(v);

                    // Don't record first value (default value)
                    if self.has_received_value.get() {
                        self.update_history((Instant::now(), v));
                    } else {
                        self.has_received_value.replace(true);
                    }

                    true
                }
                "thickness" => {
                    self.thickness.set(value.get().unwrap());
                    false // Thickness doesn't affect path geometry
                }
                "line-style" => {
                    self.line_style.set(value.get::<LineStyle>().unwrap());
                    false // Line style doesn't affect path geometry
                }
                "min" => {
                    self.min.set(value.get().unwrap());
                    true
                }
                "max" => {
                    self.max.set(value.get().unwrap());
                    true
                }
                "dynamic" => {
                    self.dynamic.set(value.get().unwrap());
                    true
                }
                "time-range" => {
                    self.time_range.set(value.get().unwrap());
                    true
                }
                "flip-x" => {
                    self.flip_x.set(value.get().unwrap());
                    true
                }
                "flip-y" => {
                    self.flip_y.set(value.get().unwrap());
                    true
                }
                "vertical" => {
                    self.vertical.set(value.get().unwrap());
                    true
                }
                "type" => {
                    self.render_type.set(value.get::<RenderType>().unwrap());
                    true
                }
                "animate" => {
                    let animate = value.get().unwrap();
                    self.set_animate(&self.obj(), animate);
                    false
                }
                x => panic!("Tried to set inexistent property of Graph: {}", x),
            };

            if needs_path_rebuild {
                self.cached_path.replace(None);
            }

            // Queue redraw for any property change
            self.obj().queue_draw();
        }

        fn property(&self, _: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "value" => self.value.get().to_value(),
                "thickness" => self.thickness.get().to_value(),
                "line-style" => self.line_style.get().to_value(),
                "min" => self.min.get().to_value(),
                "max" => self.max.get().to_value(),
                "dynamic" => self.dynamic.get().to_value(),
                "time-range" => (self.time_range.get()).to_value(),
                "flip-x" => self.flip_x.get().to_value(),
                "flip-y" => self.flip_y.get().to_value(),
                "vertical" => self.vertical.get().to_value(),
                "type" => self.render_type.get().to_value(),
                "animate" => self.is_animate().to_value(),
                x => panic!("Tried to get inexistent property of Graph: {}", x),
            }
        }
    }

    struct GraphGeometry {
        width: f32,
        height: f32,
        flip_x: bool,
        flip_y: bool,
        vertical: bool,
    }

    struct GraphRange {
        min: f64,
        max: f64,
        time_range: f32,
        anchor_time: Instant,
    }

    struct GraphStyle {
        render_type: RenderType,
        thickness: f32,
        line_style: LineStyle,
        color: gdk::RGBA,
        animate: bool,
    }

    impl WidgetImpl for Graph {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let t = self.thickness.get().max(1.0) as i32;
            // min, natural, -, -
            (t, t * 4, -1, -1)
        }

        /// Snapshot render nodes.
        ///
        /// Since this is potentially a hot code path, we want to keep it O(N) for maximum performance.
        /// - min/max value calculation cached
        /// - graph curve path cached
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let obj = self.obj();
            let history = self.history.borrow();

            if history.is_empty() {
                return;
            }

            // Margins
            let margin_start = obj.margin_start() as f32;
            let margin_end = obj.margin_end() as f32;
            let margin_top = obj.margin_top() as f32;
            let margin_bottom = obj.margin_bottom() as f32;

            // Allocated size
            let total_width = obj.width() as f32;
            let total_height = obj.height() as f32;
            let width = (total_width - margin_start - margin_end).max(0.0);
            let height = (total_height - margin_top - margin_bottom).max(0.0);

            let geom = GraphGeometry {
                width,
                height,
                flip_x: self.flip_x.get(),
                flip_y: self.flip_y.get(),
                vertical: self.vertical.get(),
            };

            let style = GraphStyle {
                render_type: self.render_type.get(),
                thickness: self.thickness.get() as f32,
                line_style: self.line_style.get(),
                color: obj.color(),
                animate: self.is_animate(),
            };

            let time_range_millis = self.time_range.get();
            if time_range_millis == 0 {
                return;
            }

            let render_time = Instant::now();
            let time_range = time_range_millis as f32;

            // Rebuild path?
            let current_size = (width, height);
            let mut cached_path = self.cached_path.borrow_mut();
            if cached_path.is_none() || self.cached_path_size.get() != current_size {
                self.cached_path_size.set(current_size);
                self.path_anchor_time.set(render_time);

                let range = GraphRange {
                    min: self.min_value(),
                    max: self.max_value(),
                    time_range,
                    anchor_time: render_time,
                };

                let new_path = build_path(&history, &geom, &range, &style);
                *cached_path = Some(new_path);
            }

            // Draw the cached path with translation
            snapshot.save();
            snapshot.translate(&graphene::Point::new(margin_start, margin_top));
            snapshot.push_clip(&graphene::Rect::new(0.0, 0.0, width, height));

            // Scroll animation
            if style.animate && history.len() >= 2 {
                // Shift by point interval to hide the gap between latest point and right edge
                let interval = {
                    let t_old = history[history.len() - 2].0;
                    let t_new = history[history.len() - 1].0;
                    t_new.checked_duration_since(t_old).unwrap_or_default()
                }
                .as_millis() as f32;

                // Calculate pixel shift based on time
                let anchor_time = self.path_anchor_time.get();
                let time_shift = render_time.duration_since(anchor_time).as_millis() as f32;
                let pixel_shift = (time_shift - interval) / time_range
                    * if geom.vertical { geom.height } else { geom.width }
                    * if geom.flip_x { 1.0 } else { -1.0 };

                // Apply the camera translation
                if geom.vertical {
                    snapshot.translate(&graphene::Point::new(0.0, pixel_shift));
                } else {
                    snapshot.translate(&graphene::Point::new(pixel_shift, 0.0));
                }
            };

            // Draw path
            if let Some(path) = cached_path.as_ref() {
                if matches!(style.render_type, RenderType::Line | RenderType::StepLine) {
                    // Render as stroked path
                    let stroke = gsk::Stroke::new(style.thickness);

                    // Configure line style
                    match style.line_style {
                        LineStyle::Miter => {
                            stroke.set_line_cap(gsk::LineCap::Butt);
                            stroke.set_line_join(gsk::LineJoin::Miter);
                        }
                        LineStyle::Bevel => {
                            stroke.set_line_cap(gsk::LineCap::Square);
                            stroke.set_line_join(gsk::LineJoin::Bevel);
                        }
                        LineStyle::Round => {
                            stroke.set_line_cap(gsk::LineCap::Round);
                            stroke.set_line_join(gsk::LineJoin::Round);
                        }
                    }

                    snapshot.append_stroke(path, &stroke, &style.color);
                } else {
                    // Render as filled path
                    snapshot.append_fill(path, gsk::FillRule::Winding, &style.color);
                }
            }

            snapshot.pop(); // pop clip
            snapshot.restore(); // restore translate
        }
    }

    impl Graph {
        // Updates the history, removing points outside the range
        fn update_history(&self, v: (Instant, f64)) {
            let mut history = self.history.borrow_mut();
            let now = Instant::now();
            self.last_updated_at.set(now);

            let time_range_dur = Duration::from_millis(self.time_range.get().into());
            let visible_start = now
                .checked_sub(time_range_dur)
                .unwrap_or_else(|| history.front().map(|(t, _)| *t).unwrap_or(now));

            // Animate: Need one extra point more left of the visible area to avoid showing a gap
            let points_to_keep_off_canvas = if self.is_animate() { 2 } else { 1 };

            // Prune history from the front (oldest).
            while history.len() > points_to_keep_off_canvas {
                // We check the timestamp of the point at index `points_to_keep_off_canvas`.
                // If that point is outside the canvas, we can safely remove the oldest point (index 0).
                if history[points_to_keep_off_canvas].0 < visible_start {
                    if let Some(val) = history.pop_front() {
                        // Value dropped: min/max values need recalc?
                        if self.min_value_cached.get().map_or(false, |min| min == val.1) {
                            self.min_value_cached.set(None);
                        }
                        if self.max_value_cached.get().map_or(false, |max| max == val.1) {
                            self.max_value_cached.set(None);
                        }
                    }
                } else {
                    break;
                }
            }

            history.push_back(v);

            // New value: Update cached min/max value?
            if let Some(min) = self.min_value_cached.get() {
                self.min_value_cached.set(Some(min.min(v.1)));
            }
            if let Some(max) = self.max_value_cached.get() {
                self.max_value_cached.set(Some(max.max(v.1)));
            }
        }

        fn set_animate(&self, obj: &super::Graph, animate: bool) {
            let mut tick_id_storage = self.tick_id.borrow_mut();

            if animate {
                // If we want it ON, and it's currently None (OFF)
                if tick_id_storage.is_none() {
                    let id = obj.add_tick_callback(|obj, _clock| {
                        obj.queue_draw();
                        glib::ControlFlow::Continue
                    });
                    *tick_id_storage = Some(id);
                }
            } else {
                // We want it OFF
                if let Some(id) = tick_id_storage.take() {
                    id.remove();
                }
            }
        }

        fn is_animate(&self) -> bool {
            self.tick_id.borrow().is_some()
        }

        fn min_value(&self) -> f64 {
            if self.dynamic.get() {
                self.min_value_cached.get().unwrap_or_else(|| {
                    // Calculate from history values
                    let history = self.history.borrow();
                    let val = history.iter().fold(f64::INFINITY, |acc, &(_, v)| acc.min(v));
                    self.min_value_cached.set(Some(val));
                    val
                })
            } else {
                self.min.get()
            }
        }

        fn max_value(&self) -> f64 {
            if self.dynamic.get() {
                self.max_value_cached.get().unwrap_or_else(|| {
                    // Calculate from history values
                    let history = self.history.borrow();
                    let val = history.iter().fold(f64::NEG_INFINITY, |acc, &(_, v)| acc.max(v));
                    self.max_value_cached.set(Some(val));
                    val
                })
            } else {
                self.max.get()
            }
        }
    }

    /// Builds a path relative to a fixed anchor time
    fn build_path(
        points: &VecDeque<(Instant, f64)>,
        geom: &GraphGeometry,
        range: &GraphRange,
        style: &GraphStyle,
    ) -> gsk::Path {
        if points.is_empty() {
            return gsk::PathBuilder::new().to_path();
        }

        if range.time_range <= 0.0 {
            return gsk::PathBuilder::new().to_path();
        }

        let builder = gsk::PathBuilder::new();
        let is_line = matches!(style.render_type, RenderType::Line | RenderType::StepLine);
        let is_step = matches!(style.render_type, RenderType::StepLine | RenderType::StepFill);

        // Transform point relative to fixed anchor
        let transform_point = |t_point: Instant, value: f64| -> (f32, f32) {
            let t = range.anchor_time.duration_since(t_point).as_millis() as f32;

            let nx = t / range.time_range;
            let ny = ((value - range.min) / (range.max - range.min).max(f64::EPSILON)) as f32;

            let x = if geom.flip_x { nx } else { 1.0 - nx };
            let y = if geom.flip_y { ny } else { 1.0 - ny };

            if geom.vertical {
                (y * geom.width, x * geom.height)
            } else {
                (x * geom.width, y * geom.height)
            }
        };

        // Calculate first point coordinates
        let &(t0, v0) = &points[0];
        let (mut last_x, mut last_y) = transform_point(t0, v0);

        let (base_x, base_y) = transform_point(range.anchor_time, range.min);

        if is_line {
            // For lines: start at first point
            builder.move_to(last_x, last_y);
        } else {
            // For fills: start at baseline then go to first point
            if geom.vertical {
                builder.move_to(base_x, last_y);
                builder.line_to(last_x, last_y);
            } else {
                builder.move_to(last_x, base_y);
                builder.line_to(last_x, last_y);
            }
        }

        // Draw the main graph line
        for i in 1..points.len() {
            let &(_, v_prev) = &points[i - 1];
            let &(t_curr, v_curr) = &points[i];

            let (x_curr, y_curr) = transform_point(t_curr, v_curr);

            if is_step {
                // Create horizontal step segment
                let (x_step, y_step) = transform_point(t_curr, v_prev);
                builder.line_to(x_step, y_step);
            }

            builder.line_to(x_curr, y_curr);
            last_x = x_curr;
            last_y = y_curr;
        }

        if is_line {
            // For lines, we're done
            builder.to_path()
        } else {
            // For fills, close the path to baseline
            if geom.vertical {
                builder.line_to(base_x, last_y);
            } else {
                builder.line_to(last_x, base_y);
            }
            builder.close();
            builder.to_path()
        }
    }
}

#[derive(glib::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[enum_type(name = "EwwiiGraphLineStyle")]
pub enum LineStyle {
    #[default]
    Miter,
    Bevel,
    Round,
}

#[derive(glib::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[enum_type(name = "EwwiiGraphRenderType")]
pub enum RenderType {
    #[default]
    Line,
    StepLine,
    Fill,
    StepFill,
}

// public wrapper
glib::wrapper! {
    pub struct Graph(ObjectSubclass<imp::Graph>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Graph {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
