use glib::{
    subclass::{object::ObjectImpl, prelude::*, simple::ClassStruct, Property},
    BoolError, Cast, Object, ParamFlags, ParamSpec, ToValue, Value,
};
use gstreamer::{
    subclass::{prelude::*, ElementInstanceStruct},
    Buffer, BufferRef, Caps, CapsIntersectMode, CoreError, DebugCategory,
    DebugColorFlags, Element, ErrorMessage, FlowError, FlowSuccess, Fraction,
    FractionRange, IntRange, List, PadDirection, PadPresence, PadTemplate,
    Plugin, Rank,
};
use gstreamer_base::{
    subclass::{prelude::*, BaseTransformMode},
    BaseTransform,
};
use gstreamer_video::{VideoFormat, VideoFrameRef, VideoInfo};
use std::{convert::TryFrom, sync::Mutex};

pub fn register(plugin: &Plugin) -> Result<(), BoolError> {
    Element::register(
        Some(plugin),
        "rsrgb2gray",
        Rank::None,
        Rgb2Gray::get_type(),
    )
}

pub struct Rgb2Gray {
    cat: DebugCategory,
    state: Mutex<Option<State>>,
    settings: Mutex<Settings>,
}

fn bgrx_to_gray(in_p: &[u8], shift: u8, invert: bool) -> u8 {
    // See https://en.wikipedia.org/wiki/YUV#SDTV_with_BT.601
    const R_Y: u32 = 19595; // 0.299 * 65536
    const G_Y: u32 = 38470; // 0.587 * 65536
    const B_Y: u32 = 7471; // 0.114 * 65536

    assert_eq!(in_p.len(), 4);

    let b = u32::from(in_p[0]);
    let g = u32::from(in_p[1]);
    let r = u32::from(in_p[2]);

    let gray = ((r * R_Y) + (g * G_Y) + (b * B_Y)) / 65536;
    let gray = (gray as u8).wrapping_add(shift);

    if invert {
        255 - gray
    } else {
        gray
    }
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
            settings: Mutex::new(Settings::default()),
            state: Mutex::new(None),
        }
    }

    fn class_init(klass: &mut ClassStruct<Self>) {
        klass.set_metadata(
            "RGB-GRAY Converter",
            "Filter/Effect/Converter/Video",
            "Converts RGB to GRAY or grayscale RGB",
            env!("CARGO_PKG_AUTHORS"),
        );
        klass.install_properties(&PROPERTIES);

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

    fn set_property(&self, obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];
        let element = obj.downcast_ref::<BaseTransform>().unwrap();

        match *prop {
            Property("invert", ..) => {
                let mut settings = self.settings.lock().unwrap();
                settings.invert = value.get().unwrap().unwrap();
                gst_info!(
                    self.cat,
                    obj: element,
                    "Changing invert from {} to {}",
                    settings.invert,
                    settings.invert
                );
            },
            Property("shift", ..) => {
                let mut settings = self.settings.lock().unwrap();
                settings.shift = value.get().unwrap().unwrap();
                gst_info!(
                    self.cat,
                    obj: element,
                    "Changing shift from {} to {}",
                    settings.shift,
                    settings.shift
                );
            },
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            Property("invert", ..) => {
                let settings = self.settings.lock().unwrap();
                Ok(settings.invert.to_value())
            },
            Property("shift", ..) => {
                let settings = self.settings.lock().unwrap();
                Ok(settings.shift.to_value())
            },
            _ => unimplemented!(),
        }
    }
}

impl ElementImpl for Rgb2Gray {}

impl BaseTransformImpl for Rgb2Gray {
    fn set_caps(
        &self,
        element: &BaseTransform,
        incaps: &Caps,
        outcaps: &Caps,
    ) -> bool {
        let in_info = match VideoInfo::from_caps(incaps) {
            None => return false,
            Some(info) => info,
        };
        let out_info = match VideoInfo::from_caps(outcaps) {
            None => return false,
            Some(info) => info,
        };

        gstreamer::gst_debug!(
            self.cat,
            obj: element,
            "Configured for caps {} to {}",
            incaps,
            outcaps
        );

        *self.state.lock().unwrap() = Some(State { in_info, out_info });

        true
    }

    fn stop(&self, element: &BaseTransform) -> Result<(), ErrorMessage> {
        // Drop state
        let _ = self.state.lock().unwrap().take();

        gstreamer::gst_info!(self.cat, obj: element, "Stopped");

        Ok(())
    }

    fn get_unit_size(
        &self,
        _element: &BaseTransform,
        caps: &Caps,
    ) -> Option<usize> {
        VideoInfo::from_caps(caps).map(|info| info.size())
    }

    fn transform_caps(
        &self,
        element: &BaseTransform,
        direction: PadDirection,
        caps: &Caps,
        filter: Option<&Caps>,
    ) -> Option<Caps> {
        let other_caps = if direction == PadDirection::Src {
            let mut caps = caps.clone();

            for s in caps.make_mut().iter_mut() {
                s.set("format", &VideoFormat::Bgrx.to_string());
            }

            caps
        } else {
            let mut gray_caps = Caps::new_empty();

            {
                let gray_caps = gray_caps.get_mut().unwrap();

                for s in caps.iter() {
                    let mut s_gray = s.to_owned();
                    s_gray.set("format", &VideoFormat::Gray8.to_string());
                    gray_caps.append_structure(s_gray);
                }
                gray_caps.append(caps.clone());
            }

            gray_caps
        };

        gst_debug!(
            self.cat,
            obj: element,
            "Transformed caps from {} to {} in direction {:?}",
            caps,
            other_caps,
            direction
        );

        if let Some(filter) = filter {
            Some(
                filter
                    .intersect_with_mode(&other_caps, CapsIntersectMode::First),
            )
        } else {
            Some(other_caps)
        }
    }

    fn transform(
        &self,
        element: &BaseTransform,
        inbuf: &Buffer,
        outbuf: &mut BufferRef,
    ) -> Result<FlowSuccess, FlowError> {
        // lock the state and make sure we've started
        let mut state_guard = self.state.lock().unwrap();
        let state = state_guard.as_mut().ok_or_else(|| {
            gst_element_error!(
                element,
                CoreError::Negotiation,
                ["Have no state yet"]
            );
            FlowError::NotNegotiated
        })?;
        let Settings { shift, invert } = *self.settings.lock().unwrap();
        let shift = u8::try_from(shift).unwrap();

        // make sure the incoming buffer is readable
        let in_frame = VideoFrameRef::from_buffer_ref_readable(
            inbuf.as_ref(),
            &state.in_info,
        )
        .ok_or_else(|| {
            gst_element_error!(
                element,
                CoreError::Failed,
                ["Failed to map input buffer readable"]
            );
            FlowError::Error
        })?;

        // make sure the outgoing buffer is writeable
        let mut out_frame =
            VideoFrameRef::from_buffer_ref_writable(outbuf, &state.out_info)
                .ok_or_else(|| {
                    gst_element_error!(
                        element,
                        CoreError::Failed,
                        ["Failed to map output buffer writable"]
                    );
                    FlowError::Error
                })?;

        let width = in_frame.width() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_format = out_frame.format();
        let out_data = out_frame.plane_data_mut(0).unwrap();

        if out_format == VideoFormat::Bgrx {
            // the sink asked for Bgrx (it doesn't support Gray8 :/)

            // sanity checks
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width * 4;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            // walk over the pixel rows and columns
            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].chunks_exact_mut(4))
                {
                    assert_eq!(out_p.len(), 4);

                    // copy the grayscale version to the output buffer
                    let gray = bgrx_to_gray(in_p, shift, invert);
                    out_p[0] = gray;
                    out_p[1] = gray;
                    out_p[2] = gray;
                }
            }
        } else if out_format == VideoFormat::Gray8 {
            // sanity checks
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            // walk over the pixel rows and columns, setting the new pixel value
            // to its grayscale version
            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].iter_mut())
                {
                    let gray = bgrx_to_gray(in_p, shift, invert);
                    *out_p = gray;
                }
            }
        } else {
            // what output format is this?!
            unimplemented!();
        }

        Ok(FlowSuccess::Ok)
    }
}

struct State {
    in_info: VideoInfo,
    out_info: VideoInfo,
}

const DEFAULT_INVERT: bool = false;
const DEFAULT_SHIFT: u32 = 0;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    invert: bool,
    shift: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            invert: DEFAULT_INVERT,
            shift: DEFAULT_SHIFT,
        }
    }
}

pub static PROPERTIES: [Property; 2] = [
    Property("invert", |name| {
        ParamSpec::boolean(
            name,
            "Invert",
            "Invert grayscale output",
            DEFAULT_INVERT,
            ParamFlags::READWRITE,
        )
    }),
    Property("shift", |name| {
        ParamSpec::uint(
            name,
            "Shift",
            "Shift grayscale output (wrapping around)",
            0,
            255,
            DEFAULT_SHIFT,
            ParamFlags::READWRITE,
        )
    }),
];
