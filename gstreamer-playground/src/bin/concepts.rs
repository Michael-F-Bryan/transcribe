use gstreamer::prelude::*;
use gstreamer::{ElementFactory, MessageView, Pipeline, State};

fn main() {
    gstreamer::init().unwrap();

    let source = ElementFactory::make("videotestsrc", Some("source")).unwrap();
    let sink = ElementFactory::make("ximagesink", Some("sink")).unwrap();

    let pipeline = Pipeline::new(Some("test-pipeline"));

    pipeline.add(&source).unwrap();
    pipeline.add(&sink).unwrap();
    source.link(&sink).unwrap();

    source.set_property_from_str("pattern", "smpte");

    pipeline.set_state(State::Playing).unwrap();

    let bus = pipeline.get_bus().unwrap();

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
            _ => (),
        }
    }

    pipeline.set_state(State::Null).unwrap();
}
