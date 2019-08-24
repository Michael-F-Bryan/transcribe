use gstreamer::prelude::*;
use gstreamer::{Bin, Element, ElementFactory, MessageView, Pad, Pipeline, State};

fn main() {
    gstreamer::init().unwrap();

    // create our pipeline elements
    let data = Data {
        source: ElementFactory::make("uridecodebin", Some("source")).unwrap(),
        convert: ElementFactory::make("audioconvert", Some("convert")).unwrap(),
        sink: ElementFactory::make("autoaudiosink", Some("sink")).unwrap(),
        pipeline: Pipeline::new(Some("test-pipeline")),
    };

    // wire up everything but the source
    let bin = data.pipeline.upcast_ref::<Bin>();
    bin.add(&data.source).unwrap();
    bin.add(&data.convert).unwrap();
    bin.add(&data.sink).unwrap();
    data.convert.link(&data.sink).unwrap();

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
    convert: Element,
    sink: Element,
}

impl Data {
    fn on_pad_added(&self, source: &Element, pad: &Pad) {
        println!(
            "Received new pad \"{}\" from \"{}\"",
            pad.get_name(),
            source.get_name()
        );

        let sink_pad = self.convert.get_static_pad("sink").unwrap();
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring...");
            return;
        }

        // check the new pad's type
        let caps = pad.get_current_caps().unwrap();
        let caps_struct = caps.get_structure(0).unwrap();
        let pad_type = caps_struct.get_name();

        if !pad_type.starts_with("audio/x-raw") {
            println!(
                "It has type \"{}\" which is not raw audio. Ignoring...",
                pad_type
            );
            return;
        }

        // attempt to link
        match pad.link(&sink_pad) {
            Ok(_) => println!("Link successful (type: \"{}\")", pad_type),
            Err(e) => println!("Type is \"{}\" but link failed: {}", pad_type, e),
        }
    }
}
