//! Audio: loading sounds and playing them (ADR-030).
//!
//! Public: [`SoundId`], a plain handle from
//! [`Context::load_sound`](crate::Context::load_sound), played with
//! [`Context::play_sound`](crate::Context::play_sound) or
//! [`Context::loop_sound`](crate::Context::loop_sound), which return a
//! [`PlaybackId`] to stop or adjust that playback (ADR-032). Crate-private: the
//! sound store (WAV decoding with `hound`, OGG Vorbis with `lewton`, ADR-033;
//! both only in `sound.rs`), the mixer (pure code, unit-tested)
//! and the device output (`cpal`, only in `output.rs`). Game code never sees
//! an audio-device type, and a missing or failing device means silence, not
//! an error.

mod mixer;
mod output;
mod sound;

pub(crate) use output::AudioOutput;
pub use output::PlaybackId;
pub use sound::SoundId;
pub(crate) use sound::Sounds;
