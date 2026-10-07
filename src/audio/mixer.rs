//! The software mixer (ADR-030): pure code, run on the audio thread.
//!
//! Every playing sound is a voice. `mix` adds all voices into the output
//! buffer, converting sample rate (linear interpolation) and channel count
//! (mono to every channel, stereo to left/right, stereo to mono by averaging),
//! then clamps to −1..1. Finished voices are removed.

use std::sync::Arc;

use super::sound::SoundData;

/// Most sounds playing at once; starting another stops the oldest.
pub(crate) const MAX_VOICES: usize = 32;

/// Loudest per-sound volume accepted (4 × the original level).
pub(crate) const MAX_VOLUME: f32 = 4.0;

/// One playing sound.
struct Voice {
    sound: Arc<SoundData>,
    /// Position in source frames (fractional when resampling).
    position: f64,
    volume: f32,
}

/// Mixes playing sounds into an interleaved `f32` output stream.
pub(crate) struct Mixer {
    channels: usize,
    sample_rate: u32,
    voices: Vec<Voice>,
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
        }
    }

    /// Starts `sound` from its beginning at `volume` (1.0 = as recorded).
    pub(crate) fn play(&mut self, sound: Arc<SoundData>, volume: f32) {
        let volume = sanitize_volume(volume);
        if volume == 0.0 || sound.frames() == 0 {
            return;
        }
        if self.voices.len() >= MAX_VOICES {
            self.voices.remove(0);
        }
        self.voices.push(Voice {
            sound,
            position: 0.0,
            volume,
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
            let step = f64::from(sound.sample_rate) / output_rate;
            for frame in out.chunks_exact_mut(channels) {
                if voice.position >= frames as f64 {
                    break;
                }
                let index = voice.position as usize;
                let fraction = (voice.position - index as f64) as f32;
                let next = (index + 1).min(frames - 1);
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
            .retain(|voice| voice.position < voice.sound.frames() as f64);
        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        mixer.play(sound(1, 100, &[0.2, 0.4, -0.6]), 0.5);
        let mut out = [9.0; 10];
        mixer.mix(&mut out);
        assert_eq!(out, [0.1, 0.1, 0.2, 0.2, -0.3, -0.3, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(mixer.playing(), 0, "finished voices are removed");
    }

    #[test]
    fn playback_continues_across_buffers() {
        let mut mixer = Mixer::new(1, 100);
        mixer.play(sound(1, 100, &[0.1, 0.2, 0.3, 0.4, 0.5]), 1.0);
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
        mixer.play(sound(1, 100, &[0.25, 0.75]), 1.0);
        mixer.play(sound(1, 100, &[0.25, 0.75]), 1.0);
        let mut out = [0.0; 2];
        mixer.mix(&mut out);
        assert_eq!(out, [0.5, 1.0], "0.75 + 0.75 is clamped to 1");
    }

    #[test]
    fn stereo_maps_to_left_right_or_is_averaged_for_mono() {
        let mut stereo_out = Mixer::new(2, 100);
        stereo_out.play(sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 2];
        stereo_out.mix(&mut out);
        assert_eq!(out, [0.2, -0.4]);
        let mut mono_out = Mixer::new(1, 100);
        mono_out.play(sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 1];
        mono_out.mix(&mut out);
        assert!((out[0] + 0.1).abs() < 1e-6);
        let mut surround = Mixer::new(4, 100);
        surround.play(sound(2, 100, &[0.2, -0.4]), 1.0);
        let mut out = [0.0; 4];
        surround.mix(&mut out);
        assert_eq!(out, [0.2, -0.4, 0.0, 0.0]);
    }

    #[test]
    fn other_sample_rates_are_resampled_linearly() {
        // 50 Hz source on a 100 Hz output: twice as many frames, midpoints interpolated.
        let mut mixer = Mixer::new(1, 100);
        mixer.play(sound(1, 50, &[0.0, 0.4, 0.8]), 1.0);
        let mut out = [0.0; 7];
        mixer.mix(&mut out);
        let expected = [0.0, 0.2, 0.4, 0.6, 0.8, 0.8, 0.0];
        for (got, want) in out.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "{out:?}");
        }
        // 200 Hz source on 100 Hz: every other frame, half the duration.
        let mut fast = Mixer::new(1, 100);
        fast.play(sound(1, 200, &[0.1, 0.2, 0.3, 0.4]), 1.0);
        let mut out = [0.0; 3];
        fast.mix(&mut out);
        assert_eq!(out, [0.1, 0.3, 0.0]);
    }

    #[test]
    fn invalid_volumes_and_empty_sounds_play_nothing() {
        let mut mixer = Mixer::new(1, 100);
        for volume in [0.0, -1.0, f32::NAN, f32::NEG_INFINITY] {
            mixer.play(sound(1, 100, &[0.5]), volume);
        }
        mixer.play(sound(1, 100, &[]), 1.0);
        assert_eq!(mixer.playing(), 0);
        assert_eq!(sanitize_volume(f32::INFINITY), 0.0);
        assert_eq!(sanitize_volume(10.0), MAX_VOLUME);
    }

    #[test]
    fn the_oldest_voice_is_dropped_past_the_limit() {
        let mut mixer = Mixer::new(1, 100);
        mixer.play(sound(1, 100, &[0.5, 0.5]), 1.0);
        for _ in 0..MAX_VOICES {
            mixer.play(sound(1, 100, &[0.0, 0.0]), 1.0);
        }
        assert_eq!(mixer.playing(), MAX_VOICES);
        let mut out = [0.0; 1];
        mixer.mix(&mut out);
        assert_eq!(out, [0.0], "the first (loud) voice was stopped");
    }
}
