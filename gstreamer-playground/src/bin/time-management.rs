use gstreamer::{
    prelude::*, ClockTime, Element, ElementFactory, Format, Message,
    MessageType, MessageView, Query, SeekFlags, State,
};
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

fn main() {
    gstreamer::init().unwrap();
    let data = Arc::new(Data::default());

    data.bin.set_property_from_str(
        "uri",
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm",
    );
    data.bin.set_state(State::Playing).unwrap();

    let bus = data.bin.get_bus().unwrap();
    let threshold = ClockTime::from_seconds(30);

    while !data.terminate.load(Ordering::SeqCst) {
        let msg = bus.timed_pop_filtered(
            threshold,
            &[
                MessageType::Error,
                MessageType::Eos,
                MessageType::DurationChanged,
                MessageType::StateChanged,
            ],
        );

        if let Some(msg) = msg {
            data.handle_msg(msg);
            continue;
        }

        // no message recieved, that means we got a timeout

        if !data.playing.load(Ordering::SeqCst) {
            continue;
        }

        // make sure our duration is up to date
        let duration = data.update_duration();
        let current = data.bin.query_position::<ClockTime>().unwrap();

        println!("Currently at {:?} / {:?}", current, duration);

        // If seeking is enabled, we have not done it yet, and the time is
        // right, seek
        if data.seek_enabled.load(Ordering::SeqCst)
            && !data.seek_done.load(Ordering::SeqCst)
            && current > threshold
        {
            println!("Reached 10s, performing seek...");
            data.bin
                .seek_simple(SeekFlags::FLUSH | SeekFlags::KEY_UNIT, threshold)
                .unwrap();
            data.seek_done.store(true, Ordering::SeqCst);
        }
    }
}

#[derive(Debug)]
struct Data {
    bin: Element,
    playing: AtomicBool,
    terminate: AtomicBool,
    seek_enabled: AtomicBool,
    seek_done: AtomicBool,
    duration: Mutex<ClockTime>,
}

impl Data {
    fn handle_msg(&self, msg: Message) {
        match msg.view() {
            MessageView::Error(e) => {
                let err = e.get_error();
                println!("Error: {}", err);
                if let Some(debug_info) = e.get_debug() {
                    println!("Debug Info: {:?}", debug_info);
                }
                self.terminate.store(true, Ordering::SeqCst);
            },
            MessageView::DurationChanged(d) => {
                println!("{:?}", d);
                *self.duration.lock().unwrap() = ClockTime::none();
            },
            MessageView::StateChanged(ref change)
                if change.get_src() != Some(self.bin.clone().upcast()) =>
            {
                println!(
                    "Pipeline state changed from {:?} to {:?}",
                    change.get_old(),
                    change.get_current()
                );

                let playing = change.get_current() == State::Playing;
                self.playing.store(playing, Ordering::SeqCst);

                if playing {
                    // we've just switched to playing. Check if seek is possible
                    let mut query = Query::new_seeking(Format::Time);
                    if self.bin.query(&mut query) {
                        let (enabled, start, end) = query.get_result();
                        self.seek_enabled.store(enabled, Ordering::SeqCst);

                        if enabled {
                            println!(
                                "Seeking is enabled from {} to {}",
                                ClockTime::try_from(start).unwrap(),
                                ClockTime::try_from(end).unwrap()
                            );
                        } else {
                            println!("Seeking is not supported");
                        }
                    } else {
                        println!("Seek query failed");
                    }
                }
            }
            _ => {},
        }
    }

    fn update_duration(&self) -> ClockTime {
        let mut d = self.duration.lock().unwrap();

        if d.is_none() {
            *d = self.bin.query_duration().unwrap();
        }

        *d
    }
}

impl Default for Data {
    fn default() -> Data {
        Data {
            bin: ElementFactory::make("playbin", Some("playbin")).unwrap(),
            playing: AtomicBool::new(false),
            terminate: AtomicBool::new(false),
            seek_enabled: AtomicBool::new(false),
            seek_done: AtomicBool::new(false),
            duration: Mutex::new(ClockTime::none()),
        }
    }
}
