//! Universal WAV encoder
//!
//! Encodes f32 samples as WAV PCM 16-bit mono.
//! No external crate needed — works on both native and WASM.

use bevy::prelude::*;
use std::sync::Arc;

/// Encode f32 samples as a WAV file in memory.
/// Returns an `AudioSource` suitable for Bevy's audio system.
pub fn create_audio_source(samples: &[f32], sample_rate: u32) -> Option<AudioSource> {
    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let s = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        buf.extend_from_slice(&s.to_le_bytes());
    }

    Some(AudioSource {
        bytes: Arc::from(buf.into_boxed_slice()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_encoder_returns_some() {
        let samples = vec![0.0f32; 100];
        let result = create_audio_source(&samples, 44100);
        assert!(result.is_some());
    }

    #[test]
    fn wav_encoder_correct_header_size() {
        let samples = vec![0.0f32; 100];
        let source = create_audio_source(&samples, 44100).unwrap();
        // 44 byte header + 100 samples * 2 bytes = 244
        assert_eq!(source.bytes.len(), 244);
    }

    #[test]
    fn wav_encoder_riff_magic() {
        let samples = vec![0.5f32; 10];
        let source = create_audio_source(&samples, 22050).unwrap();
        assert_eq!(&source.bytes[0..4], b"RIFF");
        assert_eq!(&source.bytes[8..12], b"WAVE");
    }

    #[test]
    fn wav_encoder_fmt_chunk() {
        let samples = vec![0.0f32; 10];
        let source = create_audio_source(&samples, 44100).unwrap();
        assert_eq!(&source.bytes[12..16], b"fmt ");
        // PCM format = 1
        assert_eq!(u16::from_le_bytes([source.bytes[20], source.bytes[21]]), 1);
        // Mono = 1 channel
        assert_eq!(u16::from_le_bytes([source.bytes[22], source.bytes[23]]), 1);
        // Sample rate
        assert_eq!(
            u32::from_le_bytes([
                source.bytes[24],
                source.bytes[25],
                source.bytes[26],
                source.bytes[27]
            ]),
            44100
        );
        // Bits per sample = 16
        assert_eq!(u16::from_le_bytes([source.bytes[34], source.bytes[35]]), 16);
    }

    #[test]
    fn wav_encoder_data_chunk() {
        let samples = vec![0.0f32; 10];
        let source = create_audio_source(&samples, 44100).unwrap();
        assert_eq!(&source.bytes[36..40], b"data");
        // Data size = 10 samples * 2 bytes
        assert_eq!(
            u32::from_le_bytes([
                source.bytes[40],
                source.bytes[41],
                source.bytes[42],
                source.bytes[43]
            ]),
            20
        );
    }

    #[test]
    fn wav_encoder_clamps_samples() {
        let samples = vec![-2.0, 2.0, 0.0];
        let source = create_audio_source(&samples, 44100).unwrap();
        // First sample should be clamped to -1.0 = -32767
        let s0 = i16::from_le_bytes([source.bytes[44], source.bytes[45]]);
        assert_eq!(s0, -32767);
        // Second sample should be clamped to 1.0 = 32767
        let s1 = i16::from_le_bytes([source.bytes[46], source.bytes[47]]);
        assert_eq!(s1, 32767);
        // Third sample = 0
        let s2 = i16::from_le_bytes([source.bytes[48], source.bytes[49]]);
        assert_eq!(s2, 0);
    }

    #[test]
    fn wav_encoder_empty_samples() {
        let source = create_audio_source(&[], 44100).unwrap();
        assert_eq!(source.bytes.len(), 44); // Header only
    }
}
