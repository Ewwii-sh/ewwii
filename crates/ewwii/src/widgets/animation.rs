use adw::prelude::*;
use adw::{CallbackAnimationTarget, TimedAnimation};
use gtk4::subclass::prelude::*;
use gtk4::{glib, graphene, gsk};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

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
            return;
        }

        if self.is_mapped() {
            self.imp().transition_to_sequence(sequence);
        } else {
            log::error("Trying to trigger animation before widget map. Did you add 'trigger' property to the widget directly? You are not supposed to do that. The trigger property is meant to be applied with the property update widget control.");
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnimState {
    pub scale: f32,
    pub rotate: f32,
    pub opacity: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for AnimState {
    fn default() -> Self {
        Self { scale: 1.0, rotate: 0.0, opacity: 1.0, tx: 0.0, ty: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub target: AnimState,
    pub duration: Duration,
    pub easing: adw::Easing,
}

mod imp {
    use super::*;

    #[derive(glib::Properties, Default)]
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

        // Active animated state
        pub current_state: RefCell<AnimState>,
        pub start_state: RefCell<AnimState>,
        pub target_state: RefCell<AnimState>,

        // Libadwaita Animation controller
        pub animation: RefCell<Option<TimedAnimation>>,
        pub queue: RefCell<VecDeque<Stage>>,
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

            let stages = parse_sequence(sequence, width, height);
            if stages.is_empty() {
                return;
            }

            *self.queue.borrow_mut() = stages;
            self.play_next_stage();
        }

        pub fn play_next_stage(&self) {
            let mut queue = self.queue.borrow_mut();
            let stage = match queue.pop_front() {
                Some(s) => s,
                None => return,
            };

            let obj = self.obj();
            let start = self.current_state.borrow().clone();
            *self.start_state.borrow_mut() = start;
            *self.target_state.borrow_mut() = stage.target;

            // Target function called automatically during libadwaita tick
            let target = CallbackAnimationTarget::new(glib::clone!(
                #[weak]
                obj,
                move |value| {
                    let imp = obj.imp();
                    let s = imp.start_state.borrow();
                    let t = imp.target_state.borrow();
                    let v = value as f32;

                    *imp.current_state.borrow_mut() = AnimState {
                        scale: s.scale + (t.scale - s.scale) * v,
                        rotate: s.rotate + (t.rotate - s.rotate) * v,
                        opacity: s.opacity + (t.opacity - s.opacity) * v,
                        tx: s.tx + (t.tx - s.tx) * v,
                        ty: s.ty + (t.ty - s.ty) * v,
                    };

                    obj.queue_draw();
                }
            ));

            let anim = TimedAnimation::builder()
                .widget(&*obj)
                .value_from(0.0)
                .value_to(1.0)
                .duration(stage.duration.as_millis() as u32)
                .easing(stage.easing)
                .target(&target)
                .build();

            // Automatically queue next stage when current finishes
            anim.connect_done(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.imp().play_next_stage();
                }
            ));

            anim.play();
            *self.animation.borrow_mut() = Some(anim);
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
                #[weak]
                obj,
                move |_, _, _| {
                    let seq = obj.imp().hover.borrow().clone();
                    obj.trigger(&seq);
                }
            ));
            motion.connect_leave(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    let seq = obj.imp().hoverlost.borrow().clone();
                    obj.trigger(&seq);
                }
            ));
            obj.add_controller(motion);

            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(glib::clone!(
                #[weak]
                obj,
                move |_, _, _, _| {
                    let seq = obj.imp().click.borrow().clone();
                    obj.trigger(&seq);
                }
            ));
            gesture.connect_released(glib::clone!(
                #[weak]
                obj,
                move |_, _, _, _| {
                    let seq = obj.imp().release.borrow().clone();
                    obj.trigger(&seq);
                }
            ));
            obj.add_controller(gesture);
        }
    }

    impl WidgetImpl for AnimationWidget {
        fn map(&self) {
            self.parent_map();
            let open_seq = self.open.borrow().clone();
            if !open_seq.is_empty() {
                self.obj().trigger(&open_seq);
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
                let state = self.current_state.borrow();

                let mut transform = gsk::Transform::new();
                transform = transform.translate(&graphene::Point::new(
                    w / 2.0 + state.tx,
                    h / 2.0 + state.ty,
                ));
                transform = transform.rotate(state.rotate);
                transform = transform.scale(state.scale, state.scale);
                transform = transform.translate(&graphene::Point::new(-w / 2.0, -h / 2.0));

                snapshot.save();
                snapshot.transform(Some(&transform));

                if state.opacity < 1.0 {
                    snapshot.push_opacity(state.opacity as f64);
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

// Minimal String Sequence Parser Helper
fn parse_sequence(input: &str, width: f32, height: f32) -> VecDeque<Stage> {
    let mut stages = VecDeque::new();
    for token in input.split(';') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut target = AnimState::default();
        let mut duration = Duration::from_millis(300);
        let mut easing = adw::Easing::Linear;

        for part in trimmed.split('+') {
            let parts: Vec<&str> = part.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let cmd = parts[0];
            if cmd.starts_with("scale") {
                target.scale = parse_val(cmd, 1.0);
            } else if cmd.starts_with("rotate") {
                target.rotate = parse_val(cmd, 0.0);
            } else if cmd.starts_with("fade") {
                target.opacity = parse_val(cmd, 1.0);
            } else if cmd.starts_with("slide-out-") {
                match cmd.trim_start_matches("slide-out-") {
                    "left" => target.tx = -width,
                    "right" => target.tx = width,
                    "up" => target.ty = -height,
                    "down" => target.ty = height,
                    _ => (),
                }
            }

            if parts.len() > 1 {
                if let Ok(ms) = parts[1].trim_end_matches("ms").parse::<u64>() {
                    duration = Duration::from_millis(ms);
                }
            }

            if parts.len() > 2 {
                easing = match parts[2] {
                    "ease-in" => adw::Easing::EaseInCubic,
                    "ease-out" => adw::Easing::EaseOutCubic,
                    "ease-in-out" => adw::Easing::EaseInOutCubic,
                    _ => adw::Easing::Linear,
                };
            }
        }
        stages.push_back(Stage { target, duration, easing });
    }
    stages
}

fn parse_val(raw: &str, default: f32) -> f32 {
    if let (Some(s), Some(e)) = (raw.find('('), raw.find(')')) {
        raw[s + 1..e].parse::<f32>().unwrap_or(default)
    } else {
        default
    }
}
