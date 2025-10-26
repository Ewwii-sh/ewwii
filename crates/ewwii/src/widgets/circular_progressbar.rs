use glib::Object;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{cairo, gdk, graphene};
use std::cell::Cell;

mod imp {
    use super::*;

    pub struct CircProg {
        pub value: Cell<f64>,
        pub start_at: Cell<f64>,
        pub thickness: Cell<f64>,
        pub clockwise: Cell<bool>,
        pub fg_color: Cell<gdk::RGBA>,
        pub bg_color: Cell<gdk::RGBA>,
    }

    impl Default for CircProg {
        fn default() -> Self {
            Self {
                value: Cell::new(0.0),
                start_at: Cell::new(0.0),
                thickness: Cell::new(8.0),
                clockwise: Cell::new(true),
                fg_color: Cell::new(gdk::RGBA::new(1.0, 0.0, 0.0, 1.0)),
                bg_color: Cell::new(gdk::RGBA::new(0.0, 0.0, 0.0, 0.1)),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CircProg {
        const NAME: &'static str = "CircProg";
        type Type = super::CircProg;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for CircProg {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.add_css_class("circular-progress");
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::builder("value")
                        .minimum(0.0)
                        .maximum(100.0)
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("start-at")
                        .minimum(0.0)
                        .maximum(100.0)
                        .default_value(0.0)
                        .build(),
                    glib::ParamSpecDouble::builder("thickness")
                        .minimum(1.0)
                        .maximum(50.0)
                        .default_value(8.0)
                        .build(),
                    glib::ParamSpecBoolean::builder("clockwise").default_value(true).build(),
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("fg-color").build(),
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("bg-color").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "value" => self.value.set(value.get().unwrap()),
                "start-at" => self.start_at.set(value.get().unwrap()),
                "thickness" => self.thickness.set(value.get().unwrap()),
                "clockwise" => self.clockwise.set(value.get().unwrap()),
                "fg-color" => self.fg_color.set(value.get().unwrap()),
                "bg-color" => self.bg_color.set(value.get().unwrap()),
                x => panic!("Tried to set inexistant property of CircProg: {}", x,),
            }
            self.obj().queue_draw();
        }

        fn property(&self, _: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "value" => self.value.get().to_value(),
                "start-at" => self.start_at.get().to_value(),
                "thickness" => self.thickness.get().to_value(),
                "clockwise" => self.clockwise.get().to_value(),
                "fg-color" => self.fg_color.get().to_value(),
                "bg-color" => self.bg_color.get().to_value(),
                x => panic!("Tried to get inexistant property of CircProg: {}", x,),
            }
        }
    }

    impl WidgetImpl for CircProg {
        fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let min_size = 32;
            let natural_size = 64;
            (min_size, natural_size, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let value = self.value.get();
            let start_at = self.start_at.get();
            let thickness = self.thickness.get();
            let clockwise = self.clockwise.get();
            let fg_color = self.fg_color.get();
            let bg_color = self.bg_color.get();

            let margin_start = self.obj().margin_start() as f64;
            let margin_end = self.obj().margin_end() as f64;
            let margin_top = self.obj().margin_top() as f64;
            let margin_bottom = self.obj().margin_bottom() as f64;
            // Padding is not supported yet

            let (start_angle, end_angle) = if clockwise {
                (0.0, perc_to_rad(value))
            } else {
                (perc_to_rad(100.0 - value), 2f64 * std::f64::consts::PI)
            };

            let total_width = self.obj().allocated_width() as f64;
            let total_height = self.obj().allocated_height() as f64;
            let center = (total_width / 2.0, total_height / 2.0);

            let circle_width = total_width - margin_start - margin_end;
            let circle_height = total_height - margin_top - margin_bottom;
            let outer_ring = f64::min(circle_width, circle_height) / 2.0;
            let inner_ring = (f64::min(circle_width, circle_height) / 2.0) - thickness;

            // Snapshot Cairo node
            let cr = snapshot.append_cairo(&graphene::Rect::new(
                0.0_f32,
                0.0_f32,
                total_width as f32,
                total_height as f32,
            ));

            cr.save().unwrap();

            // Centering
            cr.translate(center.0, center.1);
            cr.rotate(perc_to_rad(start_at));
            cr.translate(-center.0, -center.1);

            // Background Ring
            cr.move_to(center.0, center.1);
            cr.arc(center.0, center.1, outer_ring, 0.0, perc_to_rad(100.0));
            cr.set_source_rgba(
                bg_color.red().into(),
                bg_color.green().into(),
                bg_color.blue().into(),
                bg_color.alpha().into(),
            );
            cr.move_to(center.0, center.1);
            cr.arc(center.0, center.1, inner_ring, 0.0, perc_to_rad(100.0));
            cr.set_fill_rule(cairo::FillRule::EvenOdd); // Substract one circle from the other
            cr.fill().unwrap();

            // Foreground Ring
            cr.move_to(center.0, center.1);
            cr.arc(center.0, center.1, outer_ring, start_angle, end_angle);
            cr.set_source_rgba(
                fg_color.red().into(),
                fg_color.green().into(),
                fg_color.blue().into(),
                fg_color.alpha().into(),
            );
            cr.move_to(center.0, center.1);
            cr.arc(center.0, center.1, inner_ring, start_angle, end_angle);
            cr.set_fill_rule(cairo::FillRule::EvenOdd); // Substract one circle from the other
            cr.fill().unwrap();

            cr.restore().unwrap();
        }
    }
}

glib::wrapper! {
    pub struct CircProg(ObjectSubclass<imp::CircProg>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl CircProg {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

fn perc_to_rad(n: f64) -> f64 {
    (n / 100f64) * 2f64 * std::f64::consts::PI
}
