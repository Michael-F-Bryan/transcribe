use gstreamer::prelude::*;
use gstreamer::{ClockTime, State};

fn main() {
    gstreamer::init().unwrap();

    let pipeline = gstreamer::parse_launch("playbin uri=https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm").unwrap();
    let bus = pipeline.get_bus().unwrap();

    pipeline.set_state(State::Playing).unwrap();

    for msg in bus.iter_timed(ClockTime::none()) {
        println!("{:?}", msg);
    }
}
