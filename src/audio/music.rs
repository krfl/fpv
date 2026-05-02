use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;

#[allow(dead_code)]
mod note {
    pub const REST: f32 = 0.0;
    pub const C2: f32 = 65.41;  pub const D2: f32 = 73.42;  pub const E2: f32 = 82.41;
    pub const F2: f32 = 87.31;  pub const G2: f32 = 98.00;  pub const A2: f32 = 110.00;
    pub const AS2: f32 = 116.54; pub const B2: f32 = 123.47;
    pub const C3: f32 = 130.81; pub const CS3: f32 = 138.59; pub const D3: f32 = 146.83;
    pub const DS3: f32 = 155.56; pub const E3: f32 = 164.81; pub const F3: f32 = 174.61;
    pub const FS3: f32 = 185.00; pub const G3: f32 = 196.00; pub const GS3: f32 = 207.65;
    pub const A3: f32 = 220.00; pub const AS3: f32 = 233.08; pub const B3: f32 = 246.94;
    pub const C4: f32 = 261.63; pub const CS4: f32 = 277.18; pub const D4: f32 = 293.66;
    pub const DS4: f32 = 311.13; pub const E4: f32 = 329.63; pub const F4: f32 = 349.23;
    pub const FS4: f32 = 369.99; pub const G4: f32 = 392.00; pub const GS4: f32 = 415.30;
    pub const A4: f32 = 440.00; pub const AS4: f32 = 466.16; pub const B4: f32 = 493.88;
    pub const C5: f32 = 523.25; pub const CS5: f32 = 554.37; pub const D5: f32 = 587.33;
    pub const DS5: f32 = 622.25; pub const E5: f32 = 659.25; pub const F5: f32 = 698.46;
    pub const FS5: f32 = 739.99; pub const G5: f32 = 783.99; pub const GS5: f32 = 830.61;
    pub const A5: f32 = 880.00; pub const AS5: f32 = 932.33; pub const B5: f32 = 987.77;
}
use note::*;

#[derive(Clone, Copy)]
enum Wave { Square, Triangle, Sawtooth, #[allow(dead_code)] Noise }

#[derive(Clone, Copy)]
struct VoiceNote { freq: f32, beats: f32, wave: Wave, volume: f32 }

fn v(freq: f32, beats: f32, wave: Wave, volume: f32) -> VoiceNote {
    VoiceNote { freq, beats, wave, volume }
}
#[allow(dead_code)]
fn tri(freq: f32, beats: f32) -> VoiceNote { v(freq, beats, Wave::Triangle, 0.7) }
fn r(beats: f32) -> VoiceNote { v(REST, beats, Wave::Square, 0.0) }

pub struct SoundManager {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    music_sink: Sink,
    current_track: Track,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Track { None, Menu, Flight }

impl SoundManager {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&handle).ok()?;
        Some(Self { _stream: stream, _handle: handle, music_sink: sink, current_track: Track::None })
    }

    pub fn play(&mut self, track: Track) {
        if track == self.current_track && !self.music_sink.empty() { return; }
        self.music_sink.stop();
        self.music_sink = Sink::try_new(&self._handle).unwrap();
        self.music_sink.set_volume(0.15);
        self.current_track = track;
        match track {
            Track::None => {}
            Track::Menu => {
                self.music_sink.append(render_multivoice(&menu_voices(), 138.0).repeat_infinite());
            }
            Track::Flight => {
                self.music_sink.append(render_multivoice(&flight_voices(), 138.0).repeat_infinite());
            }
        }
        self.music_sink.play();
    }

    pub fn stop(&mut self) {
        self.music_sink.stop();
        self.music_sink = Sink::try_new(&self._handle).unwrap();
        self.current_track = Track::None;
    }
}

// ─── Renderer ───

#[inline]
fn oscillator(phase: f32, wave: Wave, ns: &mut u32) -> f32 {
    match wave {
        Wave::Square => if phase < 0.5 { 1.0 } else { -1.0 },
        Wave::Triangle => if phase < 0.25 { phase * 4.0 } else if phase < 0.75 { 2.0 - phase * 4.0 } else { phase * 4.0 - 4.0 },
        Wave::Sawtooth => 2.0 * phase - 1.0,
        Wave::Noise => { *ns ^= *ns << 13; *ns ^= *ns >> 17; *ns ^= *ns << 5; (*ns as f32 / u32::MAX as f32) * 2.0 - 1.0 }
    }
}

fn render_multivoice(voices: &[Vec<VoiceNote>], bpm: f32) -> ChiptuneSource {
    let beat_dur = 60.0 / bpm;
    let sr = 44100u32;
    let total_beats: f32 = voices.iter().map(|v| v.iter().map(|n| n.beats).sum::<f32>()).fold(0.0f32, |a, b| a.max(b));
    let total_samples = (total_beats * beat_dur * sr as f32) as usize;
    let mut mix = vec![0.0f32; total_samples];

    for voice in voices {
        let mut pos = 0usize;
        let mut ns = 0xACE1u32;
        for note in voice {
            let dur = (note.beats * beat_dur * sr as f32) as usize;
            if note.freq <= 0.0 { pos += dur; continue; }
            let atk = (0.008 * sr as f32) as usize;
            let dec = (0.03 * sr as f32) as usize;
            let rel = (0.015 * sr as f32) as usize;
            let sus = 0.6f32;
            for i in 0..dur {
                if pos + i >= total_samples { break; }
                let phase = (i as f32 * note.freq / sr as f32) % 1.0;
                let osc = oscillator(phase, note.wave, &mut ns);
                let env = if i < atk { i as f32 / atk as f32 }
                    else if i < atk + dec { 1.0 - (i - atk) as f32 / dec as f32 * (1.0 - sus) }
                    else if i >= dur.saturating_sub(rel) { (dur - i) as f32 / rel as f32 * sus }
                    else { sus };
                mix[pos + i] += osc * env * note.volume;
            }
            pos += dur;
        }
    }
    for s in &mut mix { *s = s.tanh(); }
    ChiptuneSource { samples: mix, position: 0, sample_rate: sr }
}

// ═══════════════════════════════════════════════════════════
// EPIC ATMOSPHERIC TRANCE — 138 BPM, D minor
//
// Think: Above & Beyond, Oceanlab, classic Tiësto
// Four-on-the-floor kick, offbeat bass pulse, huge supersaw pads,
// 16th-note trance gate arps, epic soaring melodies.
//
// D minor: Dm - Bb - C - Am (trance anthem progression)
// 16 bars per section = 64 beats. Both tracks loop 16 bars.
// ═══════════════════════════════════════════════════════════

/// Four-on-the-floor kick — THE trance heartbeat
fn trance_kick() -> Vec<VoiceNote> {
    let k = || v(45.0, 0.5, Wave::Triangle, 0.6);
    let mut out = Vec::new();
    for _ in 0..64 { out.push(r(0.5)); out.push(r(0.0)); } // placeholder size
    // Actually: one kick per beat, 64 beats
    out.clear();
    for _ in 0..64 { out.push(k()); }
    // But each beat = 1.0, kick lasts 0.5 then silence 0.5
    out.clear();
    for _ in 0..64 {
        out.push(v(45.0, 0.25, Wave::Triangle, 0.55));
        out.push(r(0.75));
    }
    out
}

/// Offbeat bass — pumps between kicks, classic trance energy
fn trance_bass() -> Vec<VoiceNote> {
    let b = |freq| vec![r(0.5), v(freq, 0.5, Wave::Triangle, 0.6)];
    let mut out = Vec::new();
    // 4 bars Dm, 4 bars Bb, 4 bars C, 4 bars Am (each bar = 4 beats)
    for _ in 0..16 { out.extend(b(D2)); }  // 16 beats Dm
    for _ in 0..16 { out.extend(b(AS2)); } // 16 beats Bb
    for _ in 0..16 { out.extend(b(C3)); }  // 16 beats C
    for _ in 0..16 { out.extend(b(A2)); }  // 16 beats Am
    out
}

/// Trance supersaw pad — huge, wide, emotional
fn trance_pad() -> Vec<VoiceNote> {
    // Each chord held for 16 beats (4 bars)
    vec![
        // Dm (D-F-A)
        v(D4, 16.0, Wave::Sawtooth, 0.2),
        // Bb (Bb-D-F)
        v(AS3, 16.0, Wave::Sawtooth, 0.2),
        // C (C-E-G)
        v(C4, 16.0, Wave::Sawtooth, 0.2),
        // Am (A-C-E)
        v(A3, 16.0, Wave::Sawtooth, 0.2),
    ]
}

/// Pad harmony — third above root for width
fn trance_pad2() -> Vec<VoiceNote> {
    vec![
        v(F4, 16.0, Wave::Sawtooth, 0.12),
        v(D4, 16.0, Wave::Sawtooth, 0.12),
        v(E4, 16.0, Wave::Sawtooth, 0.12),
        v(C4, 16.0, Wave::Sawtooth, 0.12),
    ]
}

/// Pad fifth — top of the chord, very quiet shimmer
fn trance_pad3() -> Vec<VoiceNote> {
    vec![
        v(A4, 16.0, Wave::Triangle, 0.08),
        v(F4, 16.0, Wave::Triangle, 0.08),
        v(G4, 16.0, Wave::Triangle, 0.08),
        v(E4, 16.0, Wave::Triangle, 0.08),
    ]
}

/// Trance gate arp — 8th note rhythmic pattern, the signature trance texture
fn trance_arp() -> Vec<VoiceNote> {
    let a = |freq| v(freq, 0.25, Wave::Square, 0.15);
    let g = || r(0.25); // gap for gated feel
    let mut out = Vec::new();

    // Dm: 16 beats = 32 eighth notes, pattern: note-gap-note-note-gap-note-note-gap
    for _ in 0..4 {
        out.extend([a(D4), g(), a(A4), a(D5), g(), a(A4), a(F4), g()]);
    }
    // Bb
    for _ in 0..4 {
        out.extend([a(AS3), g(), a(F4), a(AS4), g(), a(F4), a(D4), g()]);
    }
    // C
    for _ in 0..4 {
        out.extend([a(C4), g(), a(G4), a(C5), g(), a(G4), a(E4), g()]);
    }
    // Am
    for _ in 0..4 {
        out.extend([a(A3), g(), a(E4), a(A4), g(), a(E4), a(C4), g()]);
    }
    out
}

// ─── Menu: full epic trance with melody ───

fn menu_voices() -> Vec<Vec<VoiceNote>> {
    let m = |freq, beats| v(freq, beats, Wave::Square, 0.5);
    let melody = vec![
        // Dm (bars 1-4): heroic opening
        r(2.0), m(D5, 1.0), m(F5, 1.0),
        m(A5, 2.0), m(G5, 1.0), m(F5, 1.0),
        m(E5, 1.0), m(D5, 1.0), m(F5, 2.0),
        m(D5, 2.0), r(2.0),
        // Bb (bars 5-8): lifting up
        r(1.0), m(D5, 1.0), m(F5, 1.0), m(AS5, 1.0),
        m(A5, 2.0), m(G5, 1.0), m(F5, 1.0),
        m(G5, 1.0), m(A5, 1.0), m(G5, 1.0), m(F5, 1.0),
        m(D5, 2.0), r(2.0),
        // C (bars 9-12): soaring higher — the drop
        m(C5, 1.0), m(E5, 1.0), m(G5, 2.0),
        m(G5, 1.0), m(A5, 1.0), m(G5, 1.0), m(E5, 1.0),
        m(F5, 2.0), m(E5, 1.0), m(D5, 1.0),
        m(C5, 2.0), r(2.0),
        // Am (bars 13-16): resolve with emotion
        r(1.0), m(A4, 1.0), m(C5, 1.0), m(E5, 1.0),
        m(E5, 2.0), m(D5, 1.0), m(C5, 1.0),
        m(D5, 1.0), m(C5, 1.0), m(A4, 2.0),
        r(2.0), m(A4, 1.0), r(1.0),
    ];

    vec![trance_kick(), trance_bass(), trance_pad(), trance_pad2(), trance_pad3(), trance_arp(), melody]
}

// ─── Flight: same epic energy, mellower melody ───

fn flight_voices() -> Vec<Vec<VoiceNote>> {
    let m = |freq, beats| v(freq, beats, Wave::Square, 0.3);
    let melody = vec![
        // Dm: distant echo of the theme
        r(4.0),
        m(D5, 2.0), m(F5, 2.0),
        m(A5, 2.0), r(2.0),
        m(D5, 2.0), r(2.0),
        // Bb
        r(2.0), m(F5, 2.0),
        m(A5, 2.0), m(G5, 2.0),
        r(2.0), m(F5, 2.0),
        m(D5, 2.0), r(2.0),
        // C: hint of the soar
        m(C5, 2.0), m(E5, 2.0),
        m(G5, 2.0), r(2.0),
        m(F5, 2.0), r(2.0),
        m(C5, 2.0), r(2.0),
        // Am: resolve gently
        r(2.0), m(A4, 2.0),
        m(C5, 2.0), m(E5, 2.0),
        m(D5, 2.0), r(2.0),
        m(A4, 2.0), r(2.0),
    ];

    vec![trance_kick(), trance_bass(), trance_pad(), trance_pad2(), trance_pad3(), trance_arp(), melody]
}

// ─── Source ───

struct ChiptuneSource { samples: Vec<f32>, position: usize, sample_rate: u32 }

impl Iterator for ChiptuneSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.position >= self.samples.len() { return None; }
        let s = self.samples[self.position];
        self.position += 1;
        Some(s)
    }
}

impl Source for ChiptuneSource {
    fn current_frame_len(&self) -> Option<usize> { Some(self.samples.len() - self.position) }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(self.samples.len() as f64 / self.sample_rate as f64))
    }
}
