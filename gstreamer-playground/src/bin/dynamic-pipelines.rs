use gstreamer::prelude::*;
use gstreamer::{Bin, Element, ElementFactory, MessageView, Pad, Pipeline, State};

fn main() {
    gstreamer::init().unwrap();

    // create our pipeline elements
    let data = Data {
        source: ElementFactory::make("uridecodebin", Some("source")).unwrap(),
        video_convert: ElementFactory::make("videoconvert", Some("video_convert")).unwrap(),
        video_sink: ElementFactory::make("ximagesink", Some("video_sink")).unwrap(),
        audio_convert: ElementFactory::make("audioconvert", Some("audio_convert")).unwrap(),
        audio_sink: ElementFactory::make("autoaudiosink", Some("audio_sink")).unwrap(),
        pipeline: Pipeline::new(Some("test-pipeline")),
    };

    // wire up everything but the source
    let bin = data.pipeline.upcast_ref::<Bin>();
    bin.add(&data.source).unwrap();
    bin.add(&data.audio_convert).unwrap();
    bin.add(&data.audio_sink).unwrap();
    bin.add(&data.video_convert).unwrap();
    bin.add(&data.video_sink).unwrap();
    data.video_convert.link(&data.video_sink).unwrap();
    data.audio_convert.link(&data.audio_sink).unwrap();

    // set the source URI
    data.source.set_property_from_str(
        "uri",
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm",
    );

    // make sure we're notified when the source adds a new pad
    let d2 = data.clone();
    data.source
        .connect_pad_added(move |source, pad| d2.on_pad_added(source, pad));

    // start the pipeline
    data.pipeline.set_state(State::Playing).unwrap();

    let bus = data.pipeline.get_bus().unwrap();

    for msg in bus.iter_timed(gstreamer::CLOCK_TIME_NONE) {
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error()
                );
                eprintln!("Debugging information: {:?}", err.get_debug());
                break;
            }
            MessageView::Eos(..) => break,
            MessageView::StateChanged(change) => {
                if change.get_src() != Some(data.pipeline.clone().upcast()) {
                    // we only care about changes from the pipeline
                    continue;
                }

                println!(
                    "Pipeline state changed from {:?} to {:?}",
                    change.get_old(),
                    change.get_current()
                );
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
struct Data {
    pipeline: Pipeline,
    source: Element,
    audio_convert: Element,
    video_convert: Element,
    audio_sink: Element,
    video_sink: Element,
}

impl Data {
    fn on_pad_added(&self, source: &Element, pad: &Pad) {
        println!(
            "Received new pad \"{}\" from \"{}\"",
            pad.get_name(),
            source.get_name()
        );

        // check the new pad's type
        let caps = pad.get_current_caps().unwrap();
        let caps_struct = caps.get_structure(0).unwrap();
        let pad_type = caps_struct.get_name();

        if pad_type.starts_with("audio/x-raw") {
            self.wire_up_audio(pad, pad_type);
        } else if pad_type.starts_with("video/x-raw") {
            self.wire_up_video(pad, pad_type);
        } else {
            println!(
                "It has type \"{}\" which we can't handle. Ignoring...",
                pad_type
            );
        }
    }

    fn wire_up_audio(&self, pad: &Pad, pad_type: &str) {
        let sink_pad = self.audio_convert.get_static_pad("sink").unwrap();
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring...");
            return;
        }

        // attempt to link
        match pad.link(&sink_pad) {
            Ok(_) => println!("Link successful (type: \"{}\")", pad_type),
            Err(e) => println!("Type is \"{}\" but link failed: {}", pad_type, e),
        }
    }

    fn wire_up_video(&self, pad: &Pad, pad_type: &str) {
        let sink_pad = self.video_convert.get_static_pad("sink").unwrap();
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring...");
            return;
        }

        // attempt to link
        match pad.link(&sink_pad) {
            Ok(_) => println!("Link successful (type: \"{}\")", pad_type),
            Err(e) => println!("Type is \"{}\" but link failed: {}", pad_type, e),
        }
    }
}
