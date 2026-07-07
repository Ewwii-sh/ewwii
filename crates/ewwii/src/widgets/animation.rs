use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, graphene, gsk};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

glib::wrapper! {
    pub struct AnimationWidget(ObjectSubclass<imp::AnimationWidget>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for AnimationWidget {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl AnimationWidget {
    pub fn new(child: &impl IsA<gtk4::Widget>) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_child(Some(child));
        obj
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        let imp = self.imp();
        if let Some(ref old_child) = *imp.child.borrow() {
            old_child.unparent();
        }
        if let Some(new_child) = child {
            new_child.set_parent(self);
            *imp.child.borrow_mut() = Some(new_child.clone().upcast());
        }
    }

    pub fn trigger(&self, sequence: &str) {
        if !sequence.is_empty() {
            self.imp().transition_to_sequence(sequence);
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnimProperty {
    Scale(f32),
    Rotate(f32),
    Opacity(f32),
    TranslateX(f32),
    TranslateY(f32),
}

#[derive(Debug, Clone)]
pub struct TargetGroup {
    pub properties: Vec<AnimProperty>,
    pub duration: Duration,
    pub easing: Easing,
    pub raw_sequence_fragment: String,
}

fn parse_inner_value(raw: &str, default: f32) -> f32 {
    if let (Some(start), Some(end)) = (raw.find('('), raw.find(')')) {
        raw[start + 1..end].parse::<f32>().unwrap_or(default)
    } else {
        default
    }
}

// simple animation parser engine (similar to css)
fn parse_stage_string(input: &str, width: f32, height: f32) -> TargetGroup {
    let mut properties = Vec::new();
    let mut duration = Duration::from_millis(300);
    let mut easing = Easing::Linear;

    for token in input.split('+') {
        let parts: Vec<&str> = token.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name_raw = parts[0];
        if name_raw.starts_with("scale") {
            let scale = parse_inner_value(name_raw, 1.0);
            properties.push(AnimProperty::Scale(scale));
        } else if name_raw.starts_with("rotate") {
            let deg = parse_inner_value(name_raw, 0.0);
            properties.push(AnimProperty::Rotate(deg));
        } else if name_raw.starts_with("fade") {
            let alpha = parse_inner_value(name_raw, 1.0);
            properties.push(AnimProperty::Opacity(alpha.clamp(0.0, 1.0)));
        } else if name_raw.starts_with("slide-in-") {
            properties.push(AnimProperty::TranslateX(0.0));
            properties.push(AnimProperty::TranslateY(0.0));
        } else if name_raw.starts_with("slide-out-") {
            let dir = name_raw.trim_start_matches("slide-out-");
            let (x, y) = match dir {
                "left" => (-width, 0.0),
                "right" => (width, 0.0),
                "up" => (0.0, -height),
                "down" => (0.0, height),
                _ => (0.0, 0.0),
            };
            properties.push(AnimProperty::TranslateX(x));
            properties.push(AnimProperty::TranslateY(y));
        }

        if parts.len() > 1 {
            if parts[1].ends_with("ms") {
                if let Ok(ms) = parts[1].trim_end_matches("ms").parse::<u64>() {
                    duration = Duration::from_millis(ms);
                }
            } else if parts[1].ends_with('s') {
                if let Ok(s) = parts[1].trim_end_matches('s').parse::<f32>() {
                    duration = Duration::from_secs_f32(s);
                }
            }
        }
        if parts.len() > 2 {
            easing = match parts[2] {
                "ease-in" => Easing::EaseIn,
                "ease-out" => Easing::EaseOut,
                "ease-in-out" => Easing::EaseInOut,
                _ => Easing::Linear,
            };
        }
    }

    TargetGroup { properties, duration, easing, raw_sequence_fragment: input.to_string() }
}

// splits chains by semicolon ";"
fn parse_chain_sequence(input: &str, width: f32, height: f32) -> VecDeque<TargetGroup> {
    let mut chain = VecDeque::new();
    if input.is_empty() {
        return chain;
    }

    for stage in input.split(';') {
        let trimmed = stage.trim();
        if !trimmed.is_empty() {
            chain.push_back(parse_stage_string(trimmed, width, height));
        }
    }
    chain
}

mod imp {
    use super::*;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::AnimationWidget)]
    pub struct AnimationWidget {
        #[property(get, set)]
        pub open: RefCell<String>,
        #[property(get, set)]
        pub close: RefCell<String>,
        #[property(get, set)]
        pub hover: RefCell<String>,
        #[property(get, set)]
        pub hoverlost: RefCell<String>,
        #[property(get, set)]
        pub click: RefCell<String>,
        #[property(get, set)]
        pub release: RefCell<String>,

        pub child: RefCell<Option<gtk4::Widget>>,

        pub cur_scale: Cell<f32>,
        pub cur_rotate: Cell<f32>,
        pub cur_opacity: Cell<f32>,
        pub cur_tx: Cell<f32>,
        pub cur_ty: Cell<f32>,

        pub start_scale: Cell<f32>,
        pub start_rotate: Cell<f32>,
        pub start_opacity: Cell<f32>,
        pub start_tx: Cell<f32>,
        pub start_ty: Cell<f32>,

        pub tar_scale: Cell<f32>,
        pub tar_rotate: Cell<f32>,
        pub tar_opacity: Cell<f32>,
        pub tar_tx: Cell<f32>,
        pub tar_ty: Cell<f32>,

        pub anim_start_time: Cell<Option<Instant>>,
        pub anim_duration: Cell<Duration>,
        pub easing: Cell<Easing>,

        // que handling chains
        pub queue: RefCell<VecDeque<TargetGroup>>,

        pub hover_controller: RefCell<Option<gtk4::EventControllerMotion>>,
        pub click_controller: RefCell<Option<gtk4::GestureClick>>,
    }

    impl Default for AnimationWidget {
        fn default() -> Self {
            Self {
                open: RefCell::new(String::new()),
                close: RefCell::new(String::new()),
                hover: RefCell::new(String::new()),
                hoverlost: RefCell::new(String::new()),
                click: RefCell::new(String::new()),
                release: RefCell::new(String::new()),
                child: RefCell::new(None),

                cur_scale: Cell::new(1.0),
                cur_rotate: Cell::new(0.0),
                cur_opacity: Cell::new(1.0),
                cur_tx: Cell::new(0.0),
                cur_ty: Cell::new(0.0),

                start_scale: Cell::new(1.0),
                start_rotate: Cell::new(0.0),
                start_opacity: Cell::new(1.0),
                start_tx: Cell::new(0.0),
                start_ty: Cell::new(0.0),

                tar_scale: Cell::new(1.0),
                tar_rotate: Cell::new(0.0),
                tar_opacity: Cell::new(1.0),
                tar_tx: Cell::new(0.0),
                tar_ty: Cell::new(0.0),

                anim_start_time: Cell::new(None),
                anim_duration: Cell::new(Duration::from_millis(300)),
                easing: Cell::new(Easing::Linear),

                queue: RefCell::new(VecDeque::new()),

                hover_controller: RefCell::new(None),
                click_controller: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AnimationWidget {
        const NAME: &'static str = "AnimationWidget";
        type Type = super::AnimationWidget;
        type ParentType = gtk4::Widget;
    }

    impl AnimationWidget {
        pub fn transition_to_sequence(&self, sequence: &str) {
            let width = self.obj().width() as f32;
            let height = self.obj().height() as f32;

            let chain = parse_chain_sequence(sequence, width, height);
            if chain.is_empty() {
                return;
            }

            *self.queue.borrow_mut() = chain;
            self.pop_next_stage();
        }

        pub fn pop_next_stage(&self) {
            let mut queue = self.queue.borrow_mut();
            if let Some(target_group) = queue.pop_front() {
                let width = self.obj().width() as f32;
                let height = self.obj().height() as f32;

                self.start_scale.set(self.cur_scale.get());
                self.start_rotate.set(self.cur_rotate.get());
                self.start_opacity.set(self.cur_opacity.get());
                self.start_tx.set(self.cur_tx.get());
                self.start_ty.set(self.cur_ty.get());

                let seq_frag = &target_group.raw_sequence_fragment;
                if seq_frag.contains("slide-in-") {
                    for prop in &target_group.properties {
                        if let AnimProperty::TranslateX(_) = prop {
                            self.start_tx.set(if seq_frag.contains("left") {
                                -width
                            } else {
                                width
                            });
                        }
                        if let AnimProperty::TranslateY(_) = prop {
                            self.start_ty.set(if seq_frag.contains("up") {
                                -height
                            } else {
                                height
                            });
                        }
                    }
                }

                for prop in target_group.properties {
                    match prop {
                        AnimProperty::Scale(s) => self.tar_scale.set(s),
                        AnimProperty::Rotate(r) => self.tar_rotate.set(r),
                        AnimProperty::Opacity(o) => self.tar_opacity.set(o),
                        AnimProperty::TranslateX(x) => self.tar_tx.set(x),
                        AnimProperty::TranslateY(y) => self.tar_ty.set(y),
                    }
                }

                self.anim_duration.set(target_group.duration);
                self.easing.set(target_group.easing);
                self.anim_start_time.set(Some(Instant::now()));
                self.obj().queue_draw();
            } else {
                self.anim_start_time.set(None);
            }
        }
    }

    impl ObjectImpl for AnimationWidget {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let motion = gtk4::EventControllerMotion::new();
            motion.connect_enter(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _, _| {
                    let f = imp.hover.borrow().clone();
                    imp.transition_to_sequence(&f);
                }
            ));
            motion.connect_leave(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_| {
                    let f = imp.hoverlost.borrow().clone();
                    imp.transition_to_sequence(&f);
                }
            ));
            obj.add_controller(motion.clone());
            *self.hover_controller.borrow_mut() = Some(motion);

            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _, _, _| {
                    let f = imp.click.borrow().clone();
                    imp.transition_to_sequence(&f);
                }
            ));
            gesture.connect_released(glib::clone!(
                #[weak(rename_to = imp)]
                self,
                move |_, _, _, _| {
                    let f = imp.release.borrow().clone();
                    imp.transition_to_sequence(&f);
                }
            ));
            obj.add_controller(gesture.clone());
            *self.click_controller.borrow_mut() = Some(gesture);

            obj.add_tick_callback(|widget, _frame_clock| {
                let imp = widget.imp();
                if let Some(start_time) = imp.anim_start_time.get() {
                    let elapsed = Instant::now().duration_since(start_time);
                    let duration = imp.anim_duration.get();

                    let norm = (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0);
                    let t = imp.easing.get().apply(norm);

                    imp.cur_scale.set(
                        imp.start_scale.get() + (imp.tar_scale.get() - imp.start_scale.get()) * t,
                    );
                    imp.cur_rotate.set(
                        imp.start_rotate.get()
                            + (imp.tar_rotate.get() - imp.start_rotate.get()) * t,
                    );
                    imp.cur_opacity.set(
                        imp.start_opacity.get()
                            + (imp.tar_opacity.get() - imp.start_opacity.get()) * t,
                    );
                    imp.cur_tx
                        .set(imp.start_tx.get() + (imp.tar_tx.get() - imp.start_tx.get()) * t);
                    imp.cur_ty
                        .set(imp.start_ty.get() + (imp.tar_ty.get() - imp.start_ty.get()) * t);

                    widget.queue_draw();

                    if norm >= 1.0 {
                        // process the next in chain
                        imp.pop_next_stage();
                    }
                }
                glib::ControlFlow::Continue
            });
        }
    }

    impl WidgetImpl for AnimationWidget {
        fn map(&self) {
            self.parent_map();
            let open_seq = self.open.borrow().clone();
            if !open_seq.is_empty() {
                self.transition_to_sequence(&open_seq);
            }
        }

        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.child
                .borrow()
                .as_ref()
                .map_or((0, 0, -1, -1), |c| c.measure(orientation, for_size))
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(ref c) = *self.child.borrow() {
                c.allocate(width, height, baseline, None);
            }
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            if let Some(ref child) = *self.child.borrow() {
                let w = self.obj().width() as f32;
                let h = self.obj().height() as f32;

                let mut transform = gsk::Transform::new();

                transform = transform.translate(&graphene::Point::new(
                    w / 2.0 + self.cur_tx.get(),
                    h / 2.0 + self.cur_ty.get(),
                ));
                transform = transform.rotate(self.cur_rotate.get());
                transform = transform.scale(self.cur_scale.get(), self.cur_scale.get());
                transform = transform.translate(&graphene::Point::new(-w / 2.0, -h / 2.0));

                snapshot.save();
                snapshot.transform(Some(&transform));

                let o = self.cur_opacity.get();
                if o < 1.0 {
                    snapshot.push_opacity(o as f64);
                    self.obj().snapshot_child(child, snapshot);
                    snapshot.pop();
                } else {
                    self.obj().snapshot_child(child, snapshot);
                }
                snapshot.restore();
            }
        }
    }
}
