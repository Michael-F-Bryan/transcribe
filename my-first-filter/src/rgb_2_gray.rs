use glib::{
    subclass::{self, object::ObjectImpl, prelude::*, simple::ClassStruct},
    BoolError,
};
use gstreamer::{
    subclass::{prelude::*, ElementInstanceStruct},
    Caps, DebugCategory, DebugColorFlags, Element, Fraction, FractionRange,
    IntRange, List, PadDirection, PadPresence, PadTemplate, Plugin, Rank,
};
use gstreamer_base::{
    subclass::{prelude::*, BaseTransformMode},
    BaseTransform,
};
use gstreamer_video::VideoFormat;

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

        let caps = Caps::new_simple(
            "video/x-raw",
            &[
                (
                    "format",
                    &List::new(&[
                        &VideoFormat::Bgrx.to_string(),
                        &VideoFormat::Gray8.to_string(),
                    ]),
                ),
                ("width", &IntRange::<i32>::new(0, std::i32::MAX)),
                ("height", &IntRange::<i32>::new(0, std::i32::MAX)),
                (
                    "framerate",
                    &FractionRange::new(
                        Fraction::new(0, 1),
                        Fraction::new(std::i32::MAX, 1),
                    ),
                ),
            ],
        );
        let src_pad_template = PadTemplate::new(
            "src",
            PadDirection::Src,
            PadPresence::Always,
            &caps,
        )
        .unwrap();
        klass.add_pad_template(src_pad_template);

        let caps = Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &VideoFormat::Bgrx.to_string()),
                ("width", &IntRange::<i32>::new(0, std::i32::MAX)),
                ("height", &IntRange::<i32>::new(0, std::i32::MAX)),
                (
                    "framerate",
                    &FractionRange::new(
                        Fraction::new(0, 1),
                        Fraction::new(std::i32::MAX, 1),
                    ),
                ),
            ],
        );
        let sink_pad_template = PadTemplate::new(
            "sink",
            PadDirection::Sink,
            PadPresence::Always,
            &caps,
        )
        .unwrap();
        klass.add_pad_template(sink_pad_template);

        klass.configure(BaseTransformMode::NeverInPlace, false, false);
    }
}

impl ObjectImpl for Rgb2Gray {
    glib_object_impl!();
}
impl ElementImpl for Rgb2Gray {}
impl BaseTransformImpl for Rgb2Gray {}
