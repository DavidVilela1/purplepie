//! Sound effects and music: load WAV or OGG Vorbis files, then play them.
//!
//! A game loads a sound once with [`Context::load_sound`](crate::Context::load_sound)
//! and gets a [`SoundId`]. [`Context::play_sound`](crate::Context::play_sound)
//! plays it from the start (sounds overlap freely), and
//! [`Context::loop_sound`](crate::Context::loop_sound) repeats it seamlessly, for music.
//! Both return a [`PlaybackId`] that can stop it or change its volume. Without an
//! audio device, games run silently instead of failing. Guide: section 10
//! (`docs/GUIDE.md`).
//!
//! ```
//! use purplepie::Context;
//! use purplepie::audio::{PlaybackId, SoundId};
//!
//! struct Audio {
//!     jump: SoundId,
//!     music: PlaybackId,
//! }
//!
//! fn start_audio(ctx: &mut Context<'_>) -> purplepie::Result<Audio> {
//!     let jump = ctx.load_sound("sounds/jump.wav")?; // relative to the assets folder
//!     let theme = ctx.load_sound("sounds/theme.ogg")?;
//!     let music = ctx.loop_sound(theme, 0.6); // 1.0 = as recorded
//!     Ok(Audio { jump, music })
//! }
//!
//! fn on_jump(ctx: &mut Context<'_>, audio: &Audio) {
//!     ctx.play_sound(audio.jump, 1.0); // fire and forget
//! }
//!
//! fn on_game_over(ctx: &mut Context<'_>, audio: &Audio) {
//!     ctx.stop_sound(audio.music);
//! }
//! ```
//!
//! # Engine notes
//!
//! Public: the two handles above (ADR-030, ADR-032). Crate-private: the sound store
//! (WAV decoding with `hound`, OGG Vorbis with `lewton`, ADR-033; both only in
//! `sound.rs`), the mixer (pure code, unit-tested) and the device output (`cpal`,
//! only in `output.rs`). Game code never sees an audio-device type, and a missing
//! or failing device means silence, not an error.

mod mixer;
mod output;
mod sound;

pub(crate) use output::AudioOutput;
pub use output::PlaybackId;
pub use sound::SoundId;
pub(crate) use sound::Sounds;
