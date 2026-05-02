use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;

/// Note frequencies (Hz). Named as NOTE + OCTAVE, S = sharp.
#[allow(dead_code)]
mod note {
    pub const REST: f32 = 0.0;
    // Octave 2
    pub const A2: f32 = 110.00;
    pub const B2: f32 = 123.47;
    // Octave 3
    pub const C3: f32 = 130.81;
    pub const D3: f32 = 146.83;
    pub const DS3: f32 = 155.56;
    pub const E3: f32 = 164.81;
    pub const F3: f32 = 174.61;
    pub const FS3: f32 = 185.00;
    pub const G3: f32 = 196.00;
    pub const GS3: f32 = 207.65;
    pub const A3: f32 = 220.00;
    pub const AS3: f32 = 233.08;
    pub const B3: f32 = 246.94;
    // Octave 4
    pub const C4: f32 = 261.63;
    pub const CS4: f32 = 277.18;
    pub const D4: f32 = 293.66;
    pub const DS4: f32 = 311.13;
    pub const E4: f32 = 329.63;
    pub const F4: f32 = 349.23;
    pub const FS4: f32 = 369.99;
    pub const G4: f32 = 392.00;
    pub const GS4: f32 = 415.30;
    pub const A4: f32 = 440.00;
    pub const AS4: f32 = 466.16;
    pub const B4: f32 = 493.88;
    // Octave 5
    pub const C5: f32 = 523.25;
    pub const D5: f32 = 587.33;
    pub const DS5: f32 = 622.25;
    pub const E5: f32 = 659.25;
    pub const F5: f32 = 698.46;
    pub const G5: f32 = 783.99;
    pub const A5: f32 = 880.00;
}
use note::*;

/// A note: frequency + duration in beats.
#[derive(Clone, Copy)]
struct Note {
    freq: f32,
    beats: f32,
}

fn n(freq: f32, beats: f32) -> Note {
    Note { freq, beats }
}

/// Manages audio playback for the game.
pub struct SoundManager {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    music_sink: Sink,
    current_track: Track,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Track {
    None,
    Menu,
    Flight,
}

impl SoundManager {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&handle).ok()?;
        sink.set_volume(1.0);

        Some(Self {
            _stream: stream,
            _handle: handle,
            music_sink: sink,
            current_track: Track::None,
        })
    }

    /// Switch to a different music track. Does nothing if already playing that track.
    pub fn play(&mut self, track: Track) {
        if track == self.current_track && !self.music_sink.empty() {
            return;
        }

        self.music_sink.stop();
        self.music_sink = Sink::try_new(&self._handle).unwrap();
        self.music_sink.set_volume(0.15);
        self.current_track = track;

        match track {
            Track::None => {}
            Track::Menu => {
                let source = chiptune_sequence(&menu_melody(), 110.0, Waveform::Square)
                    .repeat_infinite()
                    .amplify(1.0);
                self.music_sink.append(source);
            }
            Track::Flight => {
                let source = chiptune_sequence(&flight_melody(), 150.0, Waveform::Square)
                    .repeat_infinite()
                    .amplify(1.0);
                self.music_sink.append(source);
            }
        }
        self.music_sink.play();
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        self.music_sink.clear();
        self.current_track = Track::None;
    }
}

/// Menu music — D minor atmospheric SID theme. ~110 BPM.
/// Inspired by C64 title screen music: arpeggiated chords, bass octaves,
/// a clear melodic hook, and space to breathe.
fn menu_melody() -> Vec<Note> {
    vec![
        // === Intro: bass octave pulse (Dm) ===
        n(D3, 0.5), n(D4, 0.5), n(D3, 0.5), n(D4, 0.5),
        n(A2, 0.5), n(A3, 0.5), n(A2, 0.5), n(A3, 0.5),

        // === A section: melody over arpeggiated Dm ===
        // Bar 1: Dm arpeggio rising into melody
        n(D3, 0.25), n(A3, 0.25), n(D4, 0.25), n(F4, 0.25),
        n(A4, 0.75), n(G4, 0.25),
        n(F4, 0.5), n(E4, 0.5),
        // Bar 2: resolve to Dm
        n(D4, 0.75), n(REST, 0.25),
        n(D3, 0.25), n(A3, 0.25), n(D4, 0.25), n(F4, 0.25),
        // Bar 3: Bb arpeggio, melody climbs
        n(AS3, 0.25), n(D4, 0.25), n(F4, 0.25), n(AS4, 0.25),
        n(A4, 0.5), n(G4, 0.25), n(F4, 0.25),
        n(G4, 0.5), n(A4, 0.5),
        // Bar 4: C arpeggio, melody descends
        n(C3, 0.25), n(G3, 0.25), n(C4, 0.25), n(E4, 0.25),
        n(G4, 0.5), n(F4, 0.25), n(E4, 0.25),
        n(D4, 1.0),

        // === B section: darker, lower ===
        // Bar 5: Gm bass pulse
        n(G3, 0.25), n(AS3, 0.25), n(D4, 0.25), n(G4, 0.25),
        n(F4, 0.25), n(D4, 0.25), n(AS3, 0.25), n(D4, 0.25),
        // Bar 6: Am bass
        n(A2, 0.25), n(E3, 0.25), n(A3, 0.25), n(C4, 0.25),
        n(E4, 0.75), n(D4, 0.25),
        // Bar 7: Dm returning
        n(D3, 0.25), n(F3, 0.25), n(A3, 0.25), n(D4, 0.25),
        n(F4, 0.5), n(E4, 0.5),
        // Bar 8: final phrase, hold and breathe
        n(D4, 0.5), n(A3, 0.5),
        n(D3, 1.0),
        n(REST, 1.0),
    ]
}

/// Flight music — E minor high-energy driving SID tune. ~150 BPM.
/// Fast alternating bass/lead (SID arpeggiation trick), relentless drive,
/// minor key tension with soaring phrases.
fn flight_melody() -> Vec<Note> {
    vec![
        // === Driving intro: E minor bass pulse ===
        n(E3, 0.25), n(E4, 0.25), n(B3, 0.25), n(E4, 0.25),
        n(E3, 0.25), n(G4, 0.25), n(B3, 0.25), n(E4, 0.25),
        n(E3, 0.25), n(E4, 0.25), n(B3, 0.25), n(G4, 0.25),
        n(FS4, 0.25), n(E4, 0.25), n(D4, 0.25), n(B3, 0.25),

        // === A section: melody bursts over pumping bass ===
        // Bar 1: Em — lead soars up
        n(E3, 0.25), n(B4, 0.25), n(E3, 0.25), n(G4, 0.25),
        n(E3, 0.25), n(A4, 0.25), n(E3, 0.25), n(B4, 0.25),
        // Bar 2: C — shift feel
        n(C3, 0.25), n(E5, 0.25), n(C3, 0.25), n(D5, 0.25),
        n(C3, 0.25), n(C5, 0.25), n(C3, 0.25), n(B4, 0.25),
        // Bar 3: D — ascending run
        n(D3, 0.25), n(A4, 0.25), n(D3, 0.25), n(B4, 0.25),
        n(D3, 0.25), n(D5, 0.25), n(D3, 0.25), n(E5, 0.25),
        // Bar 4: Em — resolve and breathe
        n(E3, 0.25), n(E5, 0.5), n(D5, 0.25),
        n(B4, 0.25), n(G4, 0.25), n(E4, 0.25), n(REST, 0.25),

        // === B section: double-time lead runs ===
        // Bar 5: fast Em arpeggio
        n(E4, 0.125), n(G4, 0.125), n(B4, 0.125), n(E5, 0.125),
        n(B4, 0.125), n(G4, 0.125), n(E4, 0.125), n(B3, 0.125),
        n(E3, 0.25), n(E4, 0.25), n(E3, 0.25), n(G4, 0.25),
        // Bar 6: Am fast arpeggio
        n(A3, 0.125), n(C4, 0.125), n(E4, 0.125), n(A4, 0.125),
        n(E4, 0.125), n(C4, 0.125), n(A3, 0.125), n(E3, 0.125),
        n(A3, 0.25), n(C4, 0.25), n(E4, 0.25), n(A4, 0.25),
        // Bar 7: B — tension peak
        n(B3, 0.25), n(DS4, 0.25), n(FS4, 0.25), n(B4, 0.25),
        n(B4, 0.25), n(A4, 0.25), n(G4, 0.25), n(FS4, 0.25),
        // Bar 8: resolve to Em
        n(E4, 0.25), n(G4, 0.25), n(B4, 0.5),
        n(E3, 0.25), n(B3, 0.25), n(E4, 0.25), n(REST, 0.25),

        // === C section: big melody with octave bass ===
        // Bar 9: soaring lead
        n(E3, 0.25), n(E5, 0.5), n(D5, 0.25),
        n(B3, 0.25), n(B4, 0.5), n(A4, 0.25),
        // Bar 10: descending
        n(A3, 0.25), n(G4, 0.25), n(FS4, 0.25), n(E4, 0.25),
        n(D4, 0.25), n(E4, 0.25), n(G4, 0.25), n(A4, 0.25),
        // Bar 11: rising again
        n(C3, 0.25), n(B4, 0.25), n(C5, 0.25), n(D5, 0.25),
        n(D3, 0.25), n(E5, 0.5), n(D5, 0.25),
        // Bar 12: final resolve
        n(B4, 0.25), n(G4, 0.25), n(E4, 0.25), n(B3, 0.25),
        n(E3, 0.5), n(REST, 0.5),
    ]
}

#[derive(Clone, Copy)]
enum Waveform {
    Square,
}

/// Build a rodio Source from a sequence of notes.
fn chiptune_sequence(
    notes: &[Note],
    bpm: f32,
    _waveform: Waveform,
) -> ChiptuneSource {
    let beat_duration = 60.0 / bpm;
    let sample_rate = 44100u32;

    let mut samples = Vec::new();

    for note in notes {
        let duration_secs = note.beats * beat_duration;
        let num_samples = (duration_secs * sample_rate as f32) as usize;

        if note.freq <= 0.0 {
            // Rest
            samples.extend(std::iter::repeat(0.0f32).take(num_samples));
        } else {
            // Square wave with simple ADSR envelope
            let attack = (0.005 * sample_rate as f32) as usize;
            let decay = (0.02 * sample_rate as f32) as usize;
            let release = (0.01 * sample_rate as f32) as usize;
            let sustain_level = 0.7f32;

            for i in 0..num_samples {
                // Square wave oscillator
                let phase = (i as f32 * note.freq / sample_rate as f32) % 1.0;
                let osc = if phase < 0.5 { 1.0f32 } else { -1.0 };

                // Envelope
                let env = if i < attack {
                    i as f32 / attack as f32
                } else if i < attack + decay {
                    let t = (i - attack) as f32 / decay as f32;
                    1.0 - t * (1.0 - sustain_level)
                } else if i >= num_samples - release {
                    let t = (num_samples - i) as f32 / release as f32;
                    sustain_level * t
                } else {
                    sustain_level
                };

                samples.push(osc * env);
            }
        }
    }

    ChiptuneSource {
        samples,
        position: 0,
        sample_rate,
    }
}

/// A rodio Source backed by pre-rendered samples.
struct ChiptuneSource {
    samples: Vec<f32>,
    position: usize,
    sample_rate: u32,
}

impl Iterator for ChiptuneSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.position >= self.samples.len() {
            return None;
        }
        let sample = self.samples[self.position];
        self.position += 1;
        Some(sample)
    }
}

impl Source for ChiptuneSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        1 // mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let secs = self.samples.len() as f64 / self.sample_rate as f64;
        Some(Duration::from_secs_f64(secs))
    }
}
