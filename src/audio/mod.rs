//! Audio: loading sounds and playing them (ADR-030).
//!
//! Public: [`SoundId`], a plain handle from
//! [`Context::load_sound`](crate::Context::load_sound), played with
//! [`Context::play_sound`](crate::Context::play_sound). Crate-private: the
//! sound store (WAV decoding, `hound`), the mixer (pure code, unit-tested)
//! and the device output (`cpal`, only in `output.rs`). Game code never sees
//! an audio-device type, and a missing or failing device means silence, not
//! an error.

mod mixer;
mod output;
mod sound;

pub(crate) use output::AudioOutput;
pub use sound::SoundId;
pub(crate) use sound::Sounds;
