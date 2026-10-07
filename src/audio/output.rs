//! The audio device (ADR-030). The only code that touches `cpal`.
//!
//! `AudioOutput::open` asks the default host for its default output device
//! and starts a stream whose callback owns a [`Mixer`]. The game side sends
//! play commands over a channel, so the audio thread never waits on a lock.
//! Any failure (no device, unsupported format, stream error) leaves a silent
//! output: playing sounds then does nothing, and the game keeps running.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::mixer::Mixer;
use super::sound::SoundData;
use crate::error::BoxError;

/// A request from the game thread to the audio thread.
enum Command {
    Play { sound: Arc<SoundData>, volume: f32 },
}

/// Where sounds go: a running device stream, or nowhere.
pub(crate) struct AudioOutput {
    commands: Option<Sender<Command>>,
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
            _stream: Some(stream),
        })
    }

    /// `true` if sounds reach a device.
    pub(crate) fn is_active(&self) -> bool {
        self.commands.is_some()
    }

    /// Starts `sound` at `volume` (ignored when silent).
    pub(crate) fn play(&self, sound: Arc<SoundData>, volume: f32) {
        if let Some(commands) = &self.commands {
            // A closed channel means the stream is gone; stay silent.
            let _ = commands.send(Command::Play { sound, volume });
        }
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
            while let Ok(Command::Play { sound, volume }) = commands.try_recv() {
                mixer.play(sound, volume);
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
