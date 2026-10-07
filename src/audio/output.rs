//! The audio device (ADR-030). The only code that touches `cpal`.
//!
//! `AudioOutput::open` asks the default host for its default output device
//! and starts a stream whose callback owns a [`Mixer`]. The game side sends
//! commands (play, stop, volume) over a channel, so the audio thread never
//! waits on a lock (ADR-030, ADR-032).
//! Any failure (no device, unsupported format, stream error) leaves a silent
//! output: playing sounds then does nothing, and the game keeps running.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::mixer::{Command, Mixer, sanitize_volume};
use super::sound::SoundData;
use crate::error::BoxError;

/// Identifies one playback started by
/// [`Context::play_sound`](crate::Context::play_sound) or
/// [`Context::loop_sound`](crate::Context::loop_sound), to stop it or change its
/// volume later. Every playback gets a new id, also without an audio device;
/// using an id whose sound has ended does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlaybackId(u64);

/// Where sounds go: a running device stream, or nowhere.
pub(crate) struct AudioOutput {
    commands: Option<Sender<Command>>,
    /// The id the next playback gets.
    next_id: u64,
    /// The master volume last requested (kept on the game side to report it).
    master: f32,
    /// Kept alive while the engine runs; dropping it stops the device.
    _stream: Option<cpal::Stream>,
}

impl std::fmt::Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioOutput")
            .field("active", &self.is_active())
            .finish()
    }
}

impl AudioOutput {
    /// No device: every sound is silently dropped.
    pub(crate) fn silent() -> Self {
        Self {
            commands: None,
            next_id: 0,
            master: 1.0,
            _stream: None,
        }
    }

    /// Opens the default output device, or returns a silent output (logged)
    /// if that is not possible. Never fails and never panics.
    pub(crate) fn open() -> Self {
        match Self::try_open() {
            Ok(output) => output,
            Err(error) => {
                log::warn!("audio disabled: {error}");
                Self::silent()
            }
        }
    }

    fn try_open() -> Result<Self, BoxError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no default audio output device")?;
        let supported = device.default_output_config()?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let mixer = Mixer::new(usize::from(config.channels), config.sample_rate);
        let (sender, receiver) = channel();
        let stream = match format {
            SampleFormat::F32 => build::<f32>(&device, config, mixer, receiver)?,
            SampleFormat::I16 => build::<i16>(&device, config, mixer, receiver)?,
            SampleFormat::U16 => build::<u16>(&device, config, mixer, receiver)?,
            SampleFormat::I32 => build::<i32>(&device, config, mixer, receiver)?,
            other => return Err(format!("unsupported device sample format {other}").into()),
        };
        stream.play()?;
        log::info!(
            "audio: {} channels at {} Hz ({format})",
            config.channels,
            config.sample_rate
        );
        Ok(Self {
            commands: Some(sender),
            next_id: 0,
            master: 1.0,
            _stream: Some(stream),
        })
    }

    /// `true` if sounds reach a device.
    pub(crate) fn is_active(&self) -> bool {
        self.commands.is_some()
    }

    /// Sends `command` to the audio thread (dropped when silent).
    fn send(&self, command: Command) {
        if let Some(commands) = &self.commands {
            // A closed channel means the stream is gone; stay silent.
            let _ = commands.send(command);
        }
    }

    /// A fresh id that no playback uses (for requests that play nothing).
    pub(crate) fn unused_id(&mut self) -> PlaybackId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        PlaybackId(id)
    }

    /// Starts `sound` at `volume`, once or looping, and returns its id.
    pub(crate) fn play(&mut self, sound: Arc<SoundData>, volume: f32, looping: bool) -> PlaybackId {
        let id = self.unused_id();
        self.send(Command::Play {
            id: id.0,
            sound,
            volume,
            looping,
        });
        id
    }

    /// Stops a playback (nothing if it already ended).
    pub(crate) fn stop(&self, playback: PlaybackId) {
        self.send(Command::Stop { id: playback.0 });
    }

    /// Changes a playback's volume.
    pub(crate) fn set_volume(&self, playback: PlaybackId, volume: f32) {
        self.send(Command::SetVolume {
            id: playback.0,
            volume,
        });
    }

    /// Stops every playback.
    pub(crate) fn stop_all(&self) {
        self.send(Command::StopAll);
    }

    /// Changes the volume applied to all sounds.
    pub(crate) fn set_master_volume(&mut self, volume: f32) {
        self.master = sanitize_volume(volume);
        self.send(Command::SetMaster {
            volume: self.master,
        });
    }

    /// The master volume last set (1.0 at start).
    pub(crate) fn master_volume(&self) -> f32 {
        self.master
    }
}

/// Builds the device stream for sample type `T`. The callback drains play
/// commands, mixes in `f32` and converts to `T`.
fn build<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut mixer: Mixer,
    commands: Receiver<Command>,
) -> Result<cpal::Stream, BoxError>
where
    T: SizedSample + FromSample<f32>,
{
    let mut scratch: Vec<f32> = Vec::new();
    let stream = device.build_output_stream(
        config,
        move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(command) = commands.try_recv() {
                mixer.apply(command);
            }
            if scratch.len() < out.len() {
                scratch.resize(out.len(), 0.0);
            }
            let mixed = &mut scratch[..out.len()];
            mixer.mix(mixed);
            for (o, &s) in out.iter_mut().zip(mixed.iter()) {
                *o = T::from_sample(s);
            }
        },
        |error| log::warn!("audio stream error: {error}"),
        None,
    )?;
    Ok(stream)
}
