//! The software mixer (ADR-030, ADR-032): pure code, run on the audio thread.
//!
//! Every playing sound is a voice with an id. `apply` handles commands from
//! the game (play, stop, change volume, master volume); `mix` adds all voices
//! into the output buffer, converting sample rate (linear interpolation) and
//! channel count (mono to every channel, stereo to left/right, stereo to mono
//! by averaging), applies the master volume and clamps to −1..1. Finished
//! one-shot voices are removed; looping voices wrap around seamlessly.

use std::sync::Arc;

use super::sound::SoundData;

/// Most sounds playing at once; starting another stops the oldest one-shot
/// sound (or the oldest loop if every voice is looping).
pub(crate) const MAX_VOICES: usize = 32;

/// Loudest per-sound or master volume accepted (4 × the original level).
pub(crate) const MAX_VOLUME: f32 = 4.0;

/// A request from the game thread to the audio thread.
#[derive(Debug, Clone)]
pub(crate) enum Command {
    /// Start `sound` as voice `id`.
    Play {
        id: u64,
        sound: Arc<SoundData>,
        volume: f32,
        looping: bool,
    },
    /// Stop voice `id` (nothing if it already ended).
    Stop { id: u64 },
    /// Change voice `id`'s volume.
    SetVolume { id: u64, volume: f32 },
    /// Change the volume applied to everything.
    SetMaster { volume: f32 },
    /// Stop every voice.
    StopAll,
}

/// One playing sound.
struct Voice {
    id: u64,
    sound: Arc<SoundData>,
    /// Position in source frames (fractional when resampling).
    position: f64,
    volume: f32,
    looping: bool,
}

/// Mixes playing sounds into an interleaved `f32` output stream.
pub(crate) struct Mixer {
    channels: usize,
    sample_rate: u32,
    voices: Vec<Voice>,
    master: f32,
}

/// A usable volume: finite, clamped to `0..=MAX_VOLUME`.
pub(crate) fn sanitize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, MAX_VOLUME)
    } else {
        0.0
    }
}

impl Mixer {
    /// A mixer for an output with `channels` channels at `sample_rate` frames per second.
    pub(crate) fn new(channels: usize, sample_rate: u32) -> Self {
        Self {
            channels: channels.max(1),
            sample_rate: sample_rate.max(1),
            voices: Vec::with_capacity(MAX_VOICES),
            master: 1.0,
        }
    }

    /// Carries out one command from the game.
    pub(crate) fn apply(&mut self, command: Command) {
        match command {
            Command::Play {
                id,
                sound,
                volume,
                looping,
            } => self.play(id, sound, volume, looping),
            Command::Stop { id } => self.voices.retain(|v| v.id != id),
            Command::SetVolume { id, volume } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == id) {
                    voice.volume = sanitize_volume(volume);
                }
            }
            Command::SetMaster { volume } => self.master = sanitize_volume(volume),
            Command::StopAll => self.voices.clear(),
        }
    }

    /// Starts `sound` from its beginning as voice `id` at `volume`.
    /// A silent volume still starts the voice (its volume can be raised later).
    fn play(&mut self, id: u64, sound: Arc<SoundData>, volume: f32, looping: bool) {
        if sound.frames() == 0 {
            return;
        }
        if self.voices.len() >= MAX_VOICES {
            // Make room: the oldest one-shot, else the oldest loop.
            let victim = self.voices.iter().position(|v| !v.looping).unwrap_or(0);
            self.voices.remove(victim);
        }
        self.voices.push(Voice {
            id,
            sound,
            position: 0.0,
            volume: sanitize_volume(volume),
            looping,
        });
    }

    /// Number of sounds still playing.
    #[cfg(test)]
    pub(crate) fn playing(&self) -> usize {
        self.voices.len()
    }

    /// Fills `out` (interleaved, `channels` per frame) with the next frames
    /// of every playing sound.
    pub(crate) fn mix(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        let channels = self.channels;
        let output_rate = f64::from(self.sample_rate);
        for voice in &mut self.voices {
            let sound = &voice.sound;
            let source_channels = usize::from(sound.channels);
            let frames = sound.frames();
            let length = frames as f64;
            let step = f64::from(sound.sample_rate) / output_rate;
            for frame in out.chunks_exact_mut(channels) {
                if voice.position >= length {
                    if !voice.looping {
                        break;
                    }
                    voice.position -= length * (voice.position / length).floor();
                }
                let index = (voice.position as usize).min(frames - 1);
                let fraction = (voice.position - index as f64) as f32;
                // The frame after the last one is the first again when looping.
                let next = if index + 1 < frames {
                    index + 1
                } else if voice.looping {
                    0
                } else {
                    index
                };
                // Linear interpolation between two source frames, one channel.
                let sample = |channel: usize| {
                    let a = sound.samples[index * source_channels + channel];
                    let b = sound.samples[next * source_channels + channel];
                    a + (b - a) * fraction
                };
                for (c, out_sample) in frame.iter_mut().enumerate() {
                    let value = match (source_channels, channels) {
                        (1, _) => sample(0),
                        (_, 1) => (sample(0) + sample(1)) * 0.5,
                        _ if c < 2 => sample(c),
                        _ => 0.0, // stereo sources stay out of surround channels
                    };
                    *out_sample += value * voice.volume;
                }
                voice.position += step;
            }
        }
        self.voices
            .retain(|voice| voice.looping || voice.position < voice.sound.frames() as f64);
        let master = self.master;
        for sample in out.iter_mut() {
            *sample = (*sample * master).clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Starts a one-shot voice with the next id.
    fn one_shot(mixer: &mut Mixer, sound: Arc<SoundData>, volume: f32) {
        let id = mixer.voices.iter().map(|v| v.id + 1).max().unwrap_or(0);
        mixer.apply(Command::Play {
            id,
            sound,
            volume,
            looping: false,
        });
    }

    fn sound(channels: u16, sample_rate: u32, samples: &[f32]) -> Arc<SoundData> {
        Arc::new(SoundData {
            channels,
            sample_rate,
            samples: samples.to_vec(),
        })
    }

    #[test]
    fn a_mono_sound_plays_on_every_channel_at_its_volume_then_stops() {
        let mut mixer = Mixer::new(2, 100);
        one_shot(&mut mixer, sound(1, 100, &[0.2, 0.4, -0.6]), 0.5);
        let mut out = [9.0; 10];
        mixer.mix(&mut out);
        assert_eq!(out, [0.1, 0.1, 0.2, 0.2, -0.3, -0.3, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(mixer.playing(), 0, "finished voices are removed");
    }

    #[test]
    fn playback_continues_across_buffers() {
        let mut mixer = Mixer::new(1, 100);
        one_shot(&mut mixer, sound(1, 100, &[0.1, 0.2, 0.3, 0.4, 0.5]), 1.0);
        let mut first = [0.0; 2];
        let mut second = [0.0; 4];
        mixer.mix(&mut first);
        mixer.mix(&mut second);
        assert_eq!(first, [0.1, 0.2]);
        assert_eq!(second, [0.3, 0.4, 0.5, 0.0]);
    }

    #[test]
    fn overlapping_sounds_add_up_and_the_sum_is_clamped() {
        let mut mixer = Mixer::new(1, 100);
        one_shot(&mut mixer, sound(1, 100, &[0.25, 0.75]), 1.0);
        one_shot(&mut mixer, sound(1, 100, &[0.25, 0.75]), 1.0);
        let mut out = [0.0; 2];
        mixer.mix(&mut out);
        assert_eq!(out, [0.5, 1.0], "0.75 + 0.75 is clamped to 1");
    }

    #[test]
    fn stereo_maps_to_left_right_or_is_averaged_for_mono() {
        let mut stereo_out = Mixer::new(2, 100);
        one_shot(&mut stereo_out, sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 2];
        stereo_out.mix(&mut out);
        assert_eq!(out, [0.2, -0.4]);
        let mut mono_out = Mixer::new(1, 100);
        one_shot(&mut mono_out, sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 1];
        mono_out.mix(&mut out);
        assert!((out[0] + 0.1).abs() < 1e-6);
        let mut surround = Mixer::new(4, 100);
        one_shot(&mut surround, sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 4];
        surround.mix(&mut out);
        assert_eq!(out, [0.2, -0.4, 0.0, 0.0]);
    }

    #[test]
    fn other_sample_rates_are_resampled_linearly() {
        // 50 Hz source on a 100 Hz output: twice as many frames, midpoints interpolated.
        let mut mixer = Mixer::new(1, 100);
        one_shot(&mut mixer, sound(1, 50, &[0.0, 0.4, 0.8]), 1.0);
        let mut out = [0.0; 7];
        mixer.mix(&mut out);
        let expected = [0.0, 0.2, 0.4, 0.6, 0.8, 0.8, 0.0];
        for (got, want) in out.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "{out:?}");
        }
        // 200 Hz source on 100 Hz: every other frame, half the duration.
        let mut fast = Mixer::new(1, 100);
        one_shot(&mut fast, sound(1, 200, &[0.1, 0.2, 0.3, 0.4]), 1.0);
        let mut out = [0.0; 3];
        fast.mix(&mut out);
        assert_eq!(out, [0.1, 0.3, 0.0]);
    }

    #[test]
    fn invalid_volumes_are_silent_and_empty_sounds_do_not_start() {
        let mut mixer = Mixer::new(1, 100);
        for volume in [0.0, -1.0, f32::NAN, f32::NEG_INFINITY] {
            one_shot(&mut mixer, sound(1, 100, &[0.5]), volume);
        }
        one_shot(&mut mixer, sound(1, 100, &[]), 1.0);
        assert_eq!(
            mixer.playing(),
            4,
            "silent voices still run (volume can rise later)"
        );
        let mut out = [9.0; 2];
        mixer.mix(&mut out);
        assert_eq!(out, [0.0, 0.0]);
        assert_eq!(mixer.playing(), 0);
        assert_eq!(sanitize_volume(f32::INFINITY), 0.0);
        assert_eq!(sanitize_volume(10.0), MAX_VOLUME);
    }

    fn looped(mixer: &mut Mixer, id: u64, sound: Arc<SoundData>, volume: f32) {
        mixer.apply(Command::Play {
            id,
            sound,
            volume,
            looping: true,
        });
    }

    #[test]
    fn a_loop_repeats_seamlessly_across_buffers_until_stopped() {
        let mut mixer = Mixer::new(1, 100);
        looped(&mut mixer, 7, sound(1, 100, &[0.1, 0.2, 0.3]), 1.0);
        let mut a = [0.0; 4];
        let mut b = [0.0; 4];
        mixer.mix(&mut a);
        mixer.mix(&mut b);
        assert_eq!(a, [0.1, 0.2, 0.3, 0.1]);
        assert_eq!(b, [0.2, 0.3, 0.1, 0.2]);
        mixer.apply(Command::Stop { id: 7 });
        mixer.mix(&mut a);
        assert_eq!(a, [0.0; 4]);
        assert_eq!(mixer.playing(), 0);
    }

    #[test]
    fn a_resampled_loop_interpolates_across_the_seam() {
        // 50 Hz source on 100 Hz output: between the last and first frame the
        // midpoint is the average of both, not the last frame held.
        let mut mixer = Mixer::new(1, 100);
        looped(&mut mixer, 1, sound(1, 50, &[0.0, 0.4]), 1.0);
        let mut out = [0.0; 6];
        mixer.mix(&mut out);
        let expected = [0.0, 0.2, 0.4, 0.2, 0.0, 0.2];
        for (got, want) in out.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "{out:?}");
        }
    }

    #[test]
    fn volumes_change_while_playing_and_the_master_scales_everything() {
        let mut mixer = Mixer::new(1, 100);
        looped(&mut mixer, 1, sound(1, 100, &[0.5]), 1.0);
        looped(&mut mixer, 2, sound(1, 100, &[0.25]), 1.0);
        let mut out = [0.0; 1];
        mixer.mix(&mut out);
        assert_eq!(out, [0.75]);
        mixer.apply(Command::SetVolume { id: 2, volume: 0.0 });
        mixer.mix(&mut out);
        assert_eq!(out, [0.5]);
        mixer.apply(Command::SetMaster { volume: 0.5 });
        mixer.mix(&mut out);
        assert_eq!(out, [0.25]);
        mixer.apply(Command::SetVolume { id: 2, volume: 2.0 });
        mixer.mix(&mut out);
        assert_eq!(out, [0.5], "(0.5 + 0.5) × 0.5");
        mixer.apply(Command::SetVolume {
            id: 99,
            volume: 1.0,
        }); // unknown id: ignored
        mixer.apply(Command::Stop { id: 99 });
        mixer.apply(Command::StopAll);
        assert_eq!(mixer.playing(), 0);
    }

    #[test]
    fn the_voice_limit_drops_one_shots_before_loops() {
        let mut mixer = Mixer::new(1, 100);
        looped(&mut mixer, 1000, sound(1, 100, &[0.5]), 1.0);
        for _ in 0..MAX_VOICES {
            one_shot(&mut mixer, sound(1, 100, &[0.0, 0.0]), 1.0);
        }
        assert_eq!(mixer.playing(), MAX_VOICES);
        let mut out = [0.0; 1];
        mixer.mix(&mut out);
        assert_eq!(out, [0.5], "the music loop survived");
    }

    #[test]
    fn the_oldest_voice_is_dropped_past_the_limit() {
        let mut mixer = Mixer::new(1, 100);
        one_shot(&mut mixer, sound(1, 100, &[0.5, 0.5]), 1.0);
        for _ in 0..MAX_VOICES {
            one_shot(&mut mixer, sound(1, 100, &[0.0, 0.0]), 1.0);
        }
        assert_eq!(mixer.playing(), MAX_VOICES);
        let mut out = [0.0; 1];
        mixer.mix(&mut out);
        assert_eq!(out, [0.0], "the first (loud) voice was stopped");
    }
}
