# Transcribe

An application for receiving radio transmissions in real time and converting it
to text. Originally intended for use with emergency services.

## Getting Started

This project uses [gstreamer][gst] for realtime audio processing. Before
starting, make sure you have the following installed:

- `libgstreamer1.0-dev` - shared libraries and files used for developing 
  aplications that use `gstreamer`
- `libgstreamer-plugins-base1.0-dev` - development files for libraries from
  the "base" set of `gstreamer` plugins

> **Note:** These are the names of the corresponding *Ubuntu* packages. Your
> distro may refer to them under different names.

## Project Structure

This project has three main parts.

- *Radio Receiver* - This receives a stream of audio (typically from a radio) 
  and breaks it up into individual transmissions. Attaching timestamps and other
  useful metadata, and transcribing the audio into text
- *Frontend* - A web UI which lets users browse received transmissions or play
  back audio
- *Server* - Serves up the *Frontend* and feeds it information from the *Radio 
  Receiver*.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[gst]: https://gstreamer.freedesktop.org
