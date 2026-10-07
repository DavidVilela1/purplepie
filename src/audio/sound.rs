//! Sound handles and the CPU-side sound store (ADR-030).
//!
//! Files are decoded completely when they load (WAV with `hound`, OGG Vorbis
//! with `lewton`, ADR-033); the format is chosen by the file's first bytes,
//! not its extension.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{BoxError, Error, Result};

/// Identifies a sound loaded with [`Context::load_sound`](crate::Context::load_sound).
///
/// A small `Copy` value: keep it in your game struct or in components and
/// pass it to [`Context::play_sound`](crate::Context::play_sound). Sounds stay
/// loaded until the engine stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SoundId(u32);

impl SoundId {
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Decoded audio: interleaved `f32` samples in −1..1.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SoundData {
    /// 1 (mono) or 2 (stereo, left first).
    pub(crate) channels: u16,
    /// Frames per second.
    pub(crate) sample_rate: u32,
    pub(crate) samples: Vec<f32>,
}

impl SoundData {
    /// Number of frames (one sample per channel).
    pub(crate) fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }
}

/// Decodes a sound file in any supported format, chosen by its first bytes:
/// `RIFF` → WAV, `OggS` → OGG Vorbis.
pub(crate) fn decode(bytes: Vec<u8>) -> std::result::Result<SoundData, BoxError> {
    if bytes.starts_with(b"OggS") {
        decode_ogg(bytes)
    } else if bytes.starts_with(b"RIFF") {
        decode_wav(bytes)
    } else {
        Err("not a WAV or OGG Vorbis file".into())
    }
}

/// Checks the format limits shared by every decoder.
fn check_format(channels: usize, sample_rate: u32) -> std::result::Result<u16, BoxError> {
    if !(1..=2).contains(&channels) {
        return Err(format!(
            "the sound has {channels} channels; only mono and stereo are supported"
        )
        .into());
    }
    if sample_rate == 0 {
        return Err("the sound has a sample rate of 0".into());
    }
    Ok(channels as u16)
}

/// Decodes a WAV file (PCM 8/16/24/32-bit integer or 32-bit float, mono or stereo).
pub(crate) fn decode_wav(bytes: Vec<u8>) -> std::result::Result<SoundData, BoxError> {
    let mut reader = hound::WavReader::new(Cursor::new(bytes))?;
    let spec = reader.spec();
    check_format(usize::from(spec.channels), spec.sample_rate)?;
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            if !(1..=32).contains(&spec.bits_per_sample) {
                return Err(
                    format!("unsupported sample size: {} bits", spec.bits_per_sample).into(),
                );
            }
            let scale = 1.0 / (1_u64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|s| s as f32 * scale))
                .collect::<std::result::Result<Vec<_>, _>>()?
        }
    };
    Ok(SoundData {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        samples,
    })
}

/// Decodes an OGG Vorbis file (mono or stereo, any sample rate) to `f32`
/// samples. Chained streams are joined if they all share the first stream's
/// channel count and sample rate.
pub(crate) fn decode_ogg(bytes: Vec<u8>) -> std::result::Result<SoundData, BoxError> {
    let mut reader = lewton::inside_ogg::OggStreamReader::new(Cursor::new(bytes))?;
    let format = |r: &lewton::inside_ogg::OggStreamReader<_>| {
        (
            usize::from(r.ident_hdr.audio_channels),
            r.ident_hdr.audio_sample_rate,
        )
    };
    let (channels, sample_rate) = format(&reader);
    let channel_count = check_format(channels, sample_rate)?;
    let mut samples = Vec::new();
    while let Some(packet) =
        reader.read_dec_packet_generic::<lewton::samples::InterleavedSamples<f32>>()?
    {
        if format(&reader) != (channels, sample_rate) || packet.channel_count != channels {
            return Err("a chained stream changes the channel count or sample rate".into());
        }
        samples.extend(packet.samples.iter().map(|s| s.clamp(-1.0, 1.0)));
    }
    if samples.is_empty() {
        return Err("the OGG stream has no audio".into());
    }
    Ok(SoundData {
        channels: channel_count,
        sample_rate,
        samples,
    })
}

/// Every sound loaded during this run, in load order. Owned by the engine's
/// runner. Entries are shared (`Arc`) with the audio thread while they play.
#[derive(Debug, Default)]
pub(crate) struct Sounds {
    entries: Vec<Arc<SoundData>>,
    /// Path → id, so loading the same path twice returns the same sound.
    by_path: HashMap<PathBuf, SoundId>,
}

impl Sounds {
    /// Loads and decodes the WAV or OGG Vorbis file at `path` (a full path from the asset
    /// root; `Context::load_sound` resolves it). A path that is already loaded
    /// returns the existing id without reading the file.
    pub(crate) fn load(&mut self, path: &Path) -> Result<SoundId> {
        if let Some(&id) = self.by_path.get(path) {
            return Ok(id);
        }
        let asset_error = |source: BoxError| Error::Asset {
            path: path.to_path_buf(),
            source,
        };
        let bytes = std::fs::read(path).map_err(|e| asset_error(Box::new(e)))?;
        let data = decode(bytes).map_err(asset_error)?;
        let id = self.push(data);
        self.by_path.insert(path.to_path_buf(), id);
        log::debug!("loaded sound {} as {id:?}", path.display());
        Ok(id)
    }

    pub(crate) fn push(&mut self, data: SoundData) -> SoundId {
        let index = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        self.entries.push(Arc::new(data));
        SoundId(index)
    }

    /// Number of sounds loaded so far.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn get(&self, id: SoundId) -> Option<&Arc<SoundData>> {
        self.entries.get(id.index())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::error::Error as _;

    /// Encodes `samples` as a WAV file in memory.
    pub(crate) fn encode_wav(spec: hound::WavSpec, samples: &[i32]) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut out, spec).expect("writer");
        for &s in samples {
            writer.write_sample(s).expect("sample");
        }
        writer.finalize().expect("finalize");
        out.into_inner()
    }

    fn spec(channels: u16, bits: u16) -> hound::WavSpec {
        hound::WavSpec {
            channels,
            sample_rate: 22_050,
            bits_per_sample: bits,
            sample_format: hound::SampleFormat::Int,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("purplepie-{}-{name}", std::process::id()))
    }

    #[test]
    fn sixteen_bit_mono_decodes_to_unit_range_floats() {
        let data =
            decode_wav(encode_wav(spec(1, 16), &[0, 16_384, -32_768, 32_767])).expect("decode");
        assert_eq!(
            (data.channels, data.sample_rate, data.frames()),
            (1, 22_050, 4)
        );
        assert_eq!(data.samples[..3], [0.0, 0.5, -1.0]);
        assert!((data.samples[3] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn stereo_24_bit_and_float_wavs_decode() {
        let stereo = decode_wav(encode_wav(spec(2, 24), &[4_194_304, -4_194_304])).expect("decode");
        assert_eq!((stereo.channels, stereo.frames()), (2, 1));
        assert_eq!(stereo.samples, [0.5, -0.5]);
        let mut out = Cursor::new(Vec::new());
        let float_spec = hound::WavSpec {
            sample_format: hound::SampleFormat::Float,
            bits_per_sample: 32,
            ..spec(1, 32)
        };
        let mut writer = hound::WavWriter::new(&mut out, float_spec).expect("writer");
        writer.write_sample(0.25_f32).expect("sample");
        writer.finalize().expect("finalize");
        assert_eq!(
            decode_wav(out.into_inner()).expect("decode").samples,
            [0.25]
        );
    }

    #[test]
    fn unsupported_or_broken_files_are_rejected() {
        assert!(decode_wav(b"not a wav file".to_vec()).is_err());
        assert!(decode_wav(Vec::new()).is_err());
        let err = decode_wav(encode_wav(spec(3, 16), &[0, 0, 0])).expect_err("3 channels");
        assert!(err.to_string().contains("3 channels"), "{err}");
    }

    #[test]
    fn the_shipped_sounds_decode() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sounds");
        for name in ["blip.wav", "hit.wav", "lose.wav", "loop.wav", "loop.ogg"] {
            let data = decode(std::fs::read(dir.join(name)).expect("read")).expect(name);
            assert_eq!((data.channels, data.sample_rate), (1, 22_050), "{name}");
            assert!(
                data.frames() > 0 && data.samples.iter().all(|s| s.abs() <= 1.0),
                "{name}"
            );
        }
    }

    fn shipped(name: &str) -> Vec<u8> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sounds");
        std::fs::read(dir.join(name)).expect(name)
    }

    #[test]
    fn the_shipped_ogg_loop_matches_its_wav_source() {
        let ogg = decode(shipped("loop.ogg")).expect("loop.ogg");
        let wav = decode(shipped("loop.wav")).expect("loop.wav");
        assert_eq!(
            (ogg.channels, ogg.sample_rate, ogg.frames()),
            (wav.channels, wav.sample_rate, wav.frames()),
            "same format and exactly the same length, so it still loops seamlessly"
        );
        let squared: f64 = ogg
            .samples
            .iter()
            .zip(&wav.samples)
            .map(|(a, b)| f64::from(a - b).powi(2))
            .sum();
        let rms_error = (squared / wav.samples.len() as f64).sqrt();
        // Measured 0.0019 against a source peak of 0.32 (Vorbis quality 4).
        assert!(rms_error < 0.005, "lossy, but close: rms error {rms_error}");
        assert!(ogg.samples.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn the_format_comes_from_the_content_not_the_name() {
        assert!(decode(shipped("loop.ogg")).is_ok());
        assert!(decode(shipped("blip.wav")).is_ok());
        let err = decode(b"ID3 an mp3, say".to_vec()).expect_err("unknown");
        assert!(
            err.to_string().contains("not a WAV or OGG Vorbis file"),
            "{err}"
        );
        assert!(decode(Vec::new()).is_err());
        // An OGG file named .wav still loads, as OGG.
        let path = temp_path("actually-ogg.wav");
        std::fs::write(&path, shipped("loop.ogg")).expect("write");
        let mut sounds = Sounds::default();
        let id = sounds.load(&path).expect("load");
        assert_eq!(sounds.get(id).expect("entry").frames(), 22_050);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn broken_ogg_files_are_rejected() {
        let good = shipped("loop.ogg");
        assert!(decode(b"OggS".to_vec()).is_err(), "only the magic");
        assert!(decode(b"OggS and then garbage".to_vec()).is_err());
        // Cut inside the headers.
        assert!(decode(good[..good.len().min(120)].to_vec()).is_err());
        // Damage the audio: the page checksum no longer matches.
        let mut damaged = good.clone();
        let middle = damaged.len() / 2;
        for byte in &mut damaged[middle..middle + 16] {
            *byte ^= 0xA5;
        }
        assert!(decode(damaged).is_err());
        let path = temp_path("broken.ogg");
        std::fs::write(&path, b"OggS garbage").expect("write");
        let err = Sounds::default().load(&path).expect_err("broken");
        assert!(
            matches!(&err, Error::Asset { path: p, .. } if *p == path),
            "{err}"
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_reuses_ids_and_reports_asset_errors() {
        let path = temp_path("sound.wav");
        std::fs::write(&path, encode_wav(spec(1, 16), &[1, 2, 3])).expect("write");
        let mut sounds = Sounds::default();
        let a = sounds.load(&path).expect("load");
        assert_eq!(sounds.load(&path).expect("again"), a);
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds.get(a).expect("entry").frames(), 3);
        std::fs::write(&path, b"garbage").expect("write");
        let other = temp_path("missing.wav");
        let err = sounds.load(&other).expect_err("missing");
        assert!(matches!(&err, Error::Asset { path: p, .. } if *p == other));
        assert!(
            err.source()
                .and_then(|s| s.downcast_ref::<std::io::Error>())
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        );
        std::fs::remove_file(&path).ok();
        assert_eq!(sounds.len(), 1, "failed loads store nothing");
    }
}
