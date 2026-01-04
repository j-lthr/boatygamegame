use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;

use bevy::audio::Source;

#[derive(Debug, Clone, Copy)]
pub struct FMSoundConfig {
    pub carrier_freq: f32,
    pub modulator_freq: f32,
    pub modulation_depth: f32,
    pub attack: f32,
    pub decay: f32,
    pub fm_level: f32, // Level of FM synthesis
    pub unison_voices: u32,
    pub detune: f32,      // Detune for unison voices
    pub noise_level: f32, // white noise level
    pub noise_decay: f32, // Decay for noise
}

// Custom audio asset for FM synthesis shooting sound
#[derive(Asset, TypePath)]
pub struct FMSound {
    pub config: FMSoundConfig,
    pub duration: Duration,
}

// Decoder for FM synthesis
pub struct FMDecoder {
    config: FMSoundConfig,
    sample_rate: u32,
    current_frame: usize,
    total_frames: usize,
}

impl FMDecoder {
    pub fn new(config: FMSoundConfig, duration: Duration) -> Self {
        let sample_rate = 44100;
        let total_frames = (duration.as_secs_f32() * sample_rate as f32) as usize;

        Self {
            config,
            sample_rate,
            current_frame: 0,
            total_frames,
        }
    }
}

struct ADEnvelope {
    pub attack: f32,
    pub decay: f32,
    pub t: f32, // Current time in seconds
}

impl ADEnvelope {
    pub fn apply(&self, audio: f32) -> f32 {
        let envelope = if self.t < self.attack {
            self.t / self.attack // Quick attack
        } else {
            (1.0 - self.t / self.decay).max(0.0) // Quick decay
        };
        audio * envelope // Apply envelope to the audio signal
    }
}

impl Iterator for FMDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame >= self.total_frames {
            return None;
        }

        let t = self.current_frame as f32 / self.sample_rate as f32;

        let mut out: f32 = 0.0;

        for j in 0..self.config.unison_voices {
            let phi = t + j as f32 * 0.01; // Slight phase offset for unison voices

            // detune between -detune / 2 and + detune / 2 semitones
            let detune_ratio = 2.0f32.powf(
                (j as f32 - (self.config.unison_voices as f32 - 1.0) / 2.0) * self.config.detune
                    / 12.0,
            );

            let carrier_freq = self.config.carrier_freq * detune_ratio;
            let modulator_freq = self.config.modulator_freq * detune_ratio;

            // FM synthesis: carrier frequency is modulated by modulator
            let modulator = (2.0 * PI * modulator_freq * phi).sin();
            let carrier =
                (2.0 * PI * carrier_freq * phi + modulator * self.config.modulation_depth).sin();

            out += carrier / self.config.unison_voices as f32; // Lower volume per voice
        }

        let fm_envelope = ADEnvelope {
            attack: self.config.attack,
            decay: self.config.decay,
            t,
        };

        let noise_envelope = ADEnvelope {
            attack: 0.0,
            decay: self.config.noise_decay,
            t, // Slightly offset for noise
        };

        out = fm_envelope.apply(out) * self.config.fm_level;

        out += self.config.noise_level * noise_envelope.apply(fastrand::f32() * 2.0 - 1.0); // Add white noise

        // add saturation
        out = 0.8 * out.tanh(); // Apply tanh for soft saturation

        // out = out.clamp(-1.0, 1.0); // Clamp to avoid clipping
        //out *= 0.5; // Lower overall volume

        self.current_frame += 1;

        Some(out) // Lower volume
    }
}

impl Source for FMDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.total_frames as f32 / self.sample_rate as f32,
        ))
    }
}

impl Decodable for FMSound {
    type DecoderItem = <FMDecoder as Iterator>::Item;
    type Decoder = FMDecoder;

    fn decoder(&self) -> Self::Decoder {
        FMDecoder::new(self.config, self.duration)
    }
}

/*
pub const GUN_SOUND: FMSoundConfig = FMSoundConfig {
    carrier_freq: 440.0,   // A2 note
    modulator_freq: 220.0, // A3 note
    modulation_depth: 0.0, // Modulation depth
    attack: 0.0,           // Quick attack
    decay: 0.2,            // Quick decay
    unison_voices: 1,      // Number of unison voices
    detune: 0.0,           // Detune for unison voices
    fm_level: 0.0,
    noise_level: 0.05, // White noise level
    noise_decay: 0.1,  // Decay for noise
};

pub const HIT_SOUND: FMSoundConfig = FMSoundConfig {
    carrier_freq: 110.0,   // A2 note
    modulator_freq: 220.0, // A3 note
    modulation_depth: 0.0, // Modulation depth
    attack: 0.0,           // Quick attack
    decay: 0.2,            // Quick decay
    unison_voices: 1,      // Number of unison voices
    detune: 0.0,           // Detune for unison voices
    fm_level: 1.0,
    noise_level: 0.05, // White noise level
    noise_decay: 0.1,  // Decay for noise
};

pub const DEATH_SOUND: FMSoundConfig = FMSoundConfig {
    carrier_freq: 55.0,    // A3 note
    modulator_freq: 440.0, // A4 note
    modulation_depth: 0.0, // Modulation depth
    attack: 0.5,           // Quick attack
    decay: 0.5,            // Quick decay
    unison_voices: 1,      // Number of unison voices
    detune: 0.0,           // Detune for unison voices
    fm_level: 1.0,
    noise_level: 0.00, // White noise level
    noise_decay: 0.1,  // Decay for noise
};

pub const SLAM: FMSoundConfig = FMSoundConfig {
    carrier_freq: 440.0,   // A3 note
    modulator_freq: 880.0, // A4 note
    modulation_depth: 4.0, // Modulation depth
    attack: 0.0,           // Quick attack
    decay: 0.1,            // Quick decay
    unison_voices: 3,      // Number of unison voices
    detune: 0.2,           // Detune for unison voices
    fm_level: 1.0,
    noise_level: 0.05, // White noise level
    noise_decay: 0.1,  // Decay for noise
};

pub const DASH_SOUND: FMSoundConfig = FMSoundConfig {
    carrier_freq: 660.0,    // Higher pitch for speed feeling
    modulator_freq: 1320.0, // Lower modulator
    modulation_depth: 2.0,  // Some FM for texture
    attack: 0.0,            // Instant attack
    decay: 0.15,            // Quick decay
    unison_voices: 2,       // Slight chorus
    detune: 0.1,            // Light detune
    fm_level: 0.8,
    noise_level: 0.1,  // More noise for "whoosh" effect
    noise_decay: 0.12, // Noise fades slightly slower
};

*/
