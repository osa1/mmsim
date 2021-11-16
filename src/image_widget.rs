use std::path::Path;
use std::rc::Rc;

use gio::prelude::Cast;
use gtk::prelude::ImageExt;

#[derive(Clone)]
pub struct Image {
    inner: Rc<ImageInner>,
}

pub struct ImageInner {
    widget: gtk::Image,
}

impl Image {
    pub fn new() -> Self {
        Image {
            inner: Rc::new(ImageInner {
                widget: gtk::Image::new(),
            }),
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.inner.widget.upcast_ref()
    }

    pub fn set_image(&self, path: &Path) {
        self.inner.widget.set_from_file(path)
    }
}
