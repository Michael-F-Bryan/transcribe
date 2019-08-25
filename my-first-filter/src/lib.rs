#[macro_use]
extern crate glib;
#[macro_use]
extern crate gstreamer;

mod rgb_2_gray;

pub use rgb_2_gray::Rgb2Gray;

use glib::BoolError;
use gstreamer::Plugin;

gstreamer::gst_plugin_define!(
    my_first_filter,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MIT/X11",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

fn plugin_init(plugin: &Plugin) -> Result<(), BoolError> {
    rgb_2_gray::register(plugin)?;
    Ok(())
}
