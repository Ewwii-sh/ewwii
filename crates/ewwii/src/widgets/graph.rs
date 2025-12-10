use anyhow::{anyhow, Result};
use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{cairo, gdk, graphene};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::time::Instant;

mod imp {
    use super::*;

    pub struct Graph {
        pub value: Cell<f64>,
        pub thickness: Cell<f64>,
        pub line_style: RefCell<String>,
        pub min: Cell<f64>,
        pub max: Cell<f64>,
        pub dynamic: Cell<bool>,
        pub time_range: Cell<u32>,
        pub flip_x: Cell<bool>,
        pub flip_y: Cell<bool>,
        pub vertical: Cell<bool>,
        pub r#type: RefCell<String>,

        // Runtime state
        pub history: RefCell<VecDeque<(Instant, f64)>>,
        pub extra_point: RefCell<Option<(Instant, f64)>>,
        pub last_updated_at: RefCell<Instant>,
    }

    impl Default for Graph {
        fn default() -> Self {
            Self {
                value: Cell::new(0.0),
                thickness: Cell::new(1.0),
                line_style: RefCell::new("miter".to_string()),
                min: Cell::new(0.0),
                max: Cell::new(100.0),
                dynamic: Cell::new(true),
                time_range: Cell::new(10),
                flip_x: Cell::new(false),
                flip_y: Cell::new(false),
                vertical: Cell::new(false),
                r#type: RefCell::new("line".to_string()),

                history: RefCell::new(VecDeque::new()),
                extra_point: RefCell::new(None),
                last_updated_at: RefCell::new(Instant::now()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Graph {
        const NAME: &'static str = "Graph";
        type Type = super::Graph;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for Graph {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_css_class("graph");
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::builder("value")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("thickness")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(1.0)
                        .build(),
                    glib::ParamSpecString::builder("line-style").default_value("miter").build(),
                    glib::ParamSpecDouble::builder("min")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("max")
                        .minimum(0.0)
                        .maximum(f64::MAX)
                        .default_value(100.0)
                        .build(),
                    glib::ParamSpecBoolean::builder("dynamic").default_value(true).build(),
                    glib::ParamSpecUInt::builder("time-range")
                        .minimum(0)
                        .maximum(u32::MAX)
                        .default_value(10)
                        .build(),
                    glib::ParamSpecBoolean::builder("flip-x").default_value(false).build(),
                    glib::ParamSpecBoolean::builder("flip-y").default_value(false).build(),
                    glib::ParamSpecBoolean::builder("vertical").default_value(false).build(),
                    glib::ParamSpecString::builder("type").default_value("line").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "value" => {
                    let v: f64 = value.get().unwrap();
                    self.value.set(v);
                    // Update history and timestamp
                    self.update_history((Instant::now(), v));
                }
                "thickness" => self.thickness.set(value.get().unwrap()),
                "line-style" => *self.line_style.borrow_mut() = value.get().unwrap(),
                "min" => self.min.set(value.get().unwrap()),
                "max" => self.max.set(value.get().unwrap()),
                "dynamic" => self.dynamic.set(value.get().unwrap()),
                "time-range" => self.time_range.set(value.get().unwrap()),
                "flip-x" => self.flip_x.set(value.get().unwrap()),
                "flip-y" => self.flip_y.set(value.get().unwrap()),
                "vertical" => self.vertical.set(value.get().unwrap()),
                "type" => *self.r#type.borrow_mut() = value.get().unwrap(),
                x => panic!("Tried to set inexistent property of Graph: {}", x),
            }
            // Queue redraw for any property change
            self.obj().queue_draw();
        }

        fn property(&self, _: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "value" => self.value.get().to_value(),
                "thickness" => self.thickness.get().to_value(),
                "line-style" => self.line_style.borrow().to_value(),
                "min" => self.min.get().to_value(),
                "max" => self.max.get().to_value(),
                "dynamic" => self.dynamic.get().to_value(),
                "time-range" => (self.time_range.get()).to_value(),
                "flip-x" => self.flip_x.get().to_value(),
                "flip-y" => self.flip_y.get().to_value(),
                "vertical" => self.vertical.get().to_value(),
                "type" => self.r#type.borrow().to_value(),
                x => panic!("Tried to get inexistent property of Graph: {}", x),
            }
        }
    }

    impl WidgetImpl for Graph {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let t = self.thickness.get().max(1.0) as i32;
            // min, natural, -, -
            (t, t * 4, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let styles = self.obj().style_context();

            // Capture state
            let thickness = self.thickness.get();
            let line_style = self.line_style.borrow().clone();
            let min = self.min.get();
            let mut max = self.max.get();
            let dynamic = self.dynamic.get();
            let time_range = self.time_range.get() as f64;
            let flip_x = self.flip_x.get();
            let flip_y = self.flip_y.get();
            let vertical = self.vertical.get();
            let r#type = self.r#type.borrow().clone();

            // Margins
            let margin_start = self.obj().margin_start() as f64;
            let margin_end = self.obj().margin_end() as f64;
            let margin_top = self.obj().margin_top() as f64;
            let margin_bottom = self.obj().margin_bottom() as f64;

            // Allocated size
            let total_width = self.obj().allocated_width() as f64;
            let total_height = self.obj().allocated_height() as f64;
            let width = (total_width - margin_start - margin_end).max(0.0);
            let height = (total_height - margin_top - margin_bottom).max(0.0);

            // Colors
            let fg_color = styles.color();
            let bg_color = match styles.lookup_color("background-color") {
                Some(color) => color,
                None => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0), // fallback
            };

            // Prepare points from history
            let points = {
                let history = self.history.borrow();
                let extra_point = *self.extra_point.borrow();
                // Determine dynamic max if requested
                if dynamic {
                    for &(_, v) in history.iter() {
                        if v > max {
                            max = v;
                        }
                    }
                    if let Some((_, v)) = extra_point {
                        if v > max {
                            max = v;
                        }
                    }
                }
                let value_range = (max - min).max(std::f64::EPSILON);

                let last_updated_at = *self.last_updated_at.borrow();
                let mut points: VecDeque<(f64, f64)> = history
                    .iter()
                    .map(|(instant, value)| {
                        let t = last_updated_at.duration_since(*instant).as_millis() as f64;
                        let nx = t / time_range;
                        let ny = (value - min) / value_range;
                        value_to_point(width, height, nx, ny, flip_x, flip_y, vertical)
                    })
                    .collect();

                // Add an extra point outside of the graph to extend the line to the left
                if let Some((instant, value)) = extra_point {
                    let t = last_updated_at.duration_since(instant).as_millis() as f64;
                    let nx = (t - time_range) / time_range;
                    let ny = (value - min) / value_range;
                    let (x, y) = value_to_point(width, height, nx, ny, flip_x, flip_y, vertical);
                    points.push_front(if vertical { (x, -y) } else { (-x, y) });
                }

                points
            };

            // Append cairo node
            let cr = snapshot.append_cairo(&graphene::Rect::new(
                0.0_f32,
                0.0_f32,
                total_width as f32,
                total_height as f32,
            ));

            // Actually draw the graph
            cr.save().unwrap();
            cr.translate(margin_start, margin_top);

            // Clip to graph area
            cr.rectangle(0.0, 0.0, width, height);
            cr.clip();

            // Draw Background
            if bg_color.alpha() > 0.0 {
                if let Some(first_point) = points.front() {
                    cr.line_to(first_point.0, height + margin_bottom);
                }
                for (x, y) in points.iter() {
                    cr.line_to(*x, *y);
                }
                cr.line_to(width, height);

                cr.set_source_rgba(
                    bg_color.red().into(),
                    bg_color.green().into(),
                    bg_color.blue().into(),
                    bg_color.alpha().into(),
                );
                cr.fill().unwrap();
            }

            // Draw graph data
            if points.is_empty() == false && fg_color.alpha() > 0.0 {
                match r#type.as_str() {
                    // Fill area under the graph
                    "fill" => {
                        let first_point = points.front().unwrap();
                        let baseline_y = if flip_y { 0.0 } else { height };

                        cr.move_to(first_point.0, baseline_y); // baseline-left
                        cr.line_to(first_point.0, first_point.1); // start at first data point

                        for (x, y) in points.iter().skip(1) {
                            cr.line_to(*x, *y);
                        }

                        let last_point = points.back().unwrap();
                        cr.line_to(last_point.0, baseline_y); // back to baseline
                        cr.close_path();

                        cr.set_source_rgba(
                            fg_color.red().into(),
                            fg_color.green().into(),
                            fg_color.blue().into(),
                            fg_color.alpha().into(),
                        );
                        cr.fill().unwrap();
                    }

                    // Default line graph
                    _ => {
                        if thickness > 0.0 {
                            for (x, y) in points.iter() {
                                cr.line_to(*x, *y);
                            }

                            apply_line_style(line_style.as_str(), &cr).unwrap_or_else(|e| {
                                // Fallback to defaults on error
                                eprintln!("apply_line_style error: {:?}", e);
                            });
                            cr.set_line_width(thickness);
                            cr.set_source_rgba(
                                fg_color.red().into(),
                                fg_color.green().into(),
                                fg_color.blue().into(),
                                fg_color.alpha().into(),
                            );
                            cr.stroke().unwrap();
                        }
                    }
                }
            }

            cr.reset_clip();
            cr.restore().unwrap();
        }
    }

    impl Graph {
        // Updates the history, removing points outside the range
        fn update_history(&self, v: (Instant, f64)) {
            let mut history = self.history.borrow_mut();
            let mut last_value = self.extra_point.borrow_mut();
            let mut last_updated_at = self.last_updated_at.borrow_mut();
            *last_updated_at = Instant::now();

            while let Some(entry) = history.front() {
                if last_updated_at.duration_since(entry.0).as_millis() as u32
                    > self.time_range.get()
                {
                    *last_value = history.pop_front();
                } else {
                    break;
                }
            }
            history.push_back(v);
        }
    }

    // helper: convert normalized coords (0..1) into widget space, with flips/vertical
    fn value_to_point(
        width: f64,
        height: f64,
        x: f64,
        y: f64,
        flip_x: bool,
        flip_y: bool,
        vertical: bool,
    ) -> (f64, f64) {
        let x = if flip_x { x } else { 1.0 - x };
        let y = if flip_y { y } else { 1.0 - y };
        let (x, y) = if vertical { (y, x) } else { (x, y) };
        (width * x, height * y)
    }

    fn apply_line_style(style: &str, cr: &cairo::Context) -> Result<()> {
        match style {
            "miter" => {
                cr.set_line_cap(cairo::LineCap::Butt);
                cr.set_line_join(cairo::LineJoin::Miter);
            }
            "bevel" => {
                cr.set_line_cap(cairo::LineCap::Square);
                cr.set_line_join(cairo::LineJoin::Bevel);
            }
            "round" => {
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
            }
            _ => Err(anyhow!("Invalid line style: '{}'", style))?,
        };
        Ok(())
    }
}

// public wrapper
glib::wrapper! {
    pub struct Graph(ObjectSubclass<imp::Graph>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Graph {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
