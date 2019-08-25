use glib::{
    subclass::{self, object::ObjectImpl, prelude::*, simple::ClassStruct},
    BoolError,
};
use gstreamer::{
    subclass::{prelude::*, ElementInstanceStruct},
    DebugCategory, DebugColorFlags, Element, Plugin, Rank,
};
use gstreamer_base::{
    subclass::{prelude::*, BaseTransformMode},
    BaseTransform,
};

pub fn register(plugin: &Plugin) -> Result<(), BoolError> {
    Element::register(
        Some(plugin),
        "rsrgb2gray",
        Rank::None,
        Rgb2Gray::get_type(),
    )
}

pub struct Rgb2Gray {
    #[allow(dead_code)]
    cat: DebugCategory,
}

impl ObjectSubclass for Rgb2Gray {
    type Class = ClassStruct<Self>;
    type Instance = ElementInstanceStruct<Self>;
    type ParentType = BaseTransform;

    const NAME: &'static str = "RsRgb2Gray";

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            cat: DebugCategory::new(
                "rsrgb2gray",
                DebugColorFlags::empty(),
                Some("Rust RGB-GRAY converter"),
            ),
        }
    }

    fn class_init(klass: &mut subclass::simple::ClassStruct<Self>) {
        klass.set_metadata(
            "RGB-GRAY Converter",
            "Filter/Effect/Converter/Video",
            "Converts RGB to GRAY or grayscale RGB",
            env!("CARGO_PKG_AUTHORS"),
        );

        klass.configure(BaseTransformMode::NeverInPlace, false, false);
    }
}

impl ObjectImpl for Rgb2Gray {
    glib_object_impl!();
}
impl ElementImpl for Rgb2Gray {}
impl BaseTransformImpl for Rgb2Gray {}
