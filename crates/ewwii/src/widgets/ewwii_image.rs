use gtk4::glib;
use gtk4::pango;
use gtk4::glib::Properties;
use glib::Object;
use gtk4::subclass::prelude::*;
use gtk4::prelude::*;
use std::cell::{Cell, OnceCell, RefCell};

mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::EwwiiImage)]
    pub struct EwwiiImage {
        pub inner_image: OnceCell<gtk4::Picture>,

        #[property(get, set = Self::set_path)]
        path: RefCell<String>,

        #[property(get, set = Self::set_image_width, default = -1)]
        image_width: Cell<i32>,

        #[property(get, set = Self::set_image_height, default = -1)]
        image_height: Cell<i32>,

        #[property(get, set = Self::set_preserve_aspect_ratio, default = true)]
        preserve_aspect_ratio: Cell<bool>,

        #[property(get, set = Self::set_fill_svg)]
        fill_svg: RefCell<String>,

        gif_animation_source: RefCell<Option<glib::SourceId>>,
    }

    impl Default for EwwiiImage {
        fn default() -> Self {
            Self {
                inner_image: OnceCell::new(),
                path: RefCell::new(String::new()),
                image_width: Cell::new(-1),
                image_height: Cell::new(-1),
                preserve_aspect_ratio: Cell::new(true),
                fill_svg: RefCell::new(String::new()),
                gif_animation_source: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EwwiiImage {
        const NAME: &'static str = "EwwiiImage";
        type Type = super::EwwiiImage;
        type ParentType = gtk4::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for EwwiiImage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk4::BinLayout::new()));
            let image = gtk4::Picture::new();
            image.set_parent(&*self.obj());
            self.inner_image.set(image).unwrap();
        }

        fn dispose(&self) {
            if let Some(source_id) = self.gif_animation_source.borrow_mut().take() {
                source_id.remove();
            }
            if let Some(image) = self.inner_image.get() {
                image.unparent();
            }
        }
    }

    impl WidgetImpl for EwwiiImage {}

    impl EwwiiImage {
        fn set_path(&self, value: String) {
            *self.path.borrow_mut() = value;
            self.sync();
        }

        fn set_image_width(&self, value: i32) {
            self.image_width.set(value);
            self.sync();
        }

        fn set_image_height(&self, value: i32) {
            self.image_height.set(value);
            self.sync();
        }

        fn set_preserve_aspect_ratio(&self, value: bool) {
            self.preserve_aspect_ratio.set(value);
            self.sync();
        }

        fn set_fill_svg(&self, value: String) {
            *self.fill_svg.borrow_mut() = value;
            self.sync();
        }

        fn sync(&self) {
            let Some(image) = self.inner_image.get() else { return };

            let path = self.path.borrow().clone();
            let image_width = self.image_width.get();
            let image_height = self.image_height.get();
            let preserve_aspect_ratio = self.preserve_aspect_ratio.get();
            let fill_svg = self.fill_svg.borrow().clone();

            image.set_height_request(image_height);
            image.set_width_request(image_width);

            if !path.ends_with(".svg") && !fill_svg.is_empty() {
                log::warn!("Fill attribute ignored, file is not an svg image");
            }

            if path.ends_with(".gif") {
                let pixbuf_animation = match gtk4::gdk_pixbuf::PixbufAnimation::from_file(
                    std::path::PathBuf::from(&path),
                ) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!("Failed to load GIF `{path}`: {e}");
                        return;
                    }
                };
                let iter = pixbuf_animation.iter(None);
                let frame_pixbuf = iter.pixbuf();
                image.set_pixbuf(Some(&frame_pixbuf));
                let widget_clone = image.clone();
                if let Some(delay) = iter.delay_time() {
                    let source_id = glib::timeout_add_local(delay, move || {
                        let now = std::time::SystemTime::now();
                        if iter.advance(now) {
                            let frame_pixbuf = iter.pixbuf();
                            widget_clone.set_pixbuf(Some(&frame_pixbuf));
                        }
                        glib::ControlFlow::Continue
                    });
                    *self.gif_animation_source.borrow_mut() = Some(source_id);
                }
            } else {
                let scale = image.scale_factor();
                let width = if image_width > 0 { image_width * scale } else { -1 };
                let height = if image_height > 0 { image_height * scale } else { -1 };

                let pixbuf = if path.ends_with(".svg") && !fill_svg.is_empty() {
                    let svg_data = match std::fs::read_to_string(std::path::PathBuf::from(&path)) {
                        Ok(d) => d,
                        Err(e) => {
                            log::error!("Failed to read SVG `{path}`: {e}");
                            return;
                        }
                    };
                    let svg_data = if svg_data.contains("fill=") {
                        let reg = match regex::Regex::new(r#"fill="[^"]*""#) {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("Regex error: {e}");
                                return;
                            }
                        };
                        reg.replace(&svg_data, &format!("fill=\"{}\"", fill_svg))
                    } else {
                        let reg = match regex::Regex::new(r"<svg") {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("Regex error: {e}");
                                return;
                            }
                        };
                        reg.replace(&svg_data, &format!("<svg fill=\"{}\"", fill_svg))
                    };
                    let stream = gtk4::gio::MemoryInputStream::from_bytes(
                        &gtk4::glib::Bytes::from(svg_data.as_bytes()),
                    );
                    let result = gtk4::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                        &stream,
                        width,
                        height,
                        preserve_aspect_ratio,
                        None::<&gtk4::gio::Cancellable>,
                    );
                    if let Err(e) = stream.close(None::<&gtk4::gio::Cancellable>) {
                        log::error!("Failed to close SVG stream: {e}");
                    }
                    match result {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed to render SVG `{path}`: {e}");
                            return;
                        }
                    }
                } else {
                    match gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(
                        std::path::PathBuf::from(&path),
                        width,
                        height,
                        preserve_aspect_ratio,
                    ) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed to load image `{path}`: {e}");
                            return;
                        }
                    }
                };

                let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
                image.set_paintable(Some(&texture));
            }
        }
    }
}

glib::wrapper! {
    pub struct EwwiiImage(ObjectSubclass<imp::EwwiiImage>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl EwwiiImage {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_content_fit(&self, fit: gtk4::ContentFit) {
        if let Some(image) = self.imp().inner_image.get() {
            image.set_content_fit(fit);
        }
    }

    pub fn set_can_shrink(&self, shrink: bool) {
        if let Some(image) = self.imp().inner_image.get() {
            image.set_can_shrink(shrink);
        }
    }
}

impl Default for EwwiiImage {
    fn default() -> Self {
        Self::new()
    }
}
