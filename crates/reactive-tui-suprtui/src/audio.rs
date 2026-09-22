//! Deterministic audio engine with a built-in test backend
//! (MED-003 … MED-007).
//!
//! Ports the observable `audio.zig` contract without miniaudio: no
//! threads, no hardware, no resampling. Mixing is sample-exact and
//! single-threaded so the MED mechanisms observe every frame. The
//! test backend supplies canned device lists; real backends would
//! implement the same delivery rules against hardware.

use std::collections::{HashMap, VecDeque};

/// Default sample rate for created engines.
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
/// Default playback channel count.
pub const DEFAULT_OUTPUT_CHANNELS: u8 = 2;
/// Maximum capture capacity in frames.
pub const MAX_CAPTURE_FRAMES: u32 = 1 << 20;

/// Audio failure. Every fallible operation reports; nothing panics on
/// bad input (MED-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioError {
    /// `start` found no playback device.
    NoDevice,
    /// Bad dimensions, empty data, unknown channel layout, or an
    /// out-of-range selection.
    InvalidArgument,
    /// Unknown sound, voice, or stream id.
    UnknownId,
    /// Write after close.
    Closed,
    /// Destination holds less than one frame.
    FrameTooSmall,
    /// Corrupt sound bytes.
    InvalidData,
}

/// Address of a loaded sound. Ids are monotonic and never reused.
pub type SoundId = u32;
/// Address of a playing voice.
pub type VoiceId = u32;
/// Address of an open PCM stream.
pub type StreamId = u32;
/// Address of a mix group. Group 0 is the default group and always exists.
pub type GroupId = u32;
/// Default mix group.
pub const DEFAULT_GROUP: GroupId = 0;
/// Volume clamp ceiling (mirrors the reference wrapper).
pub const MAX_VOLUME: f32 = 4.0;

/// Engine construction options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    pub sample_rate: u32,
    pub playback_channels: u8,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            playback_channels: DEFAULT_OUTPUT_CHANNELS,
        }
    }
}

/// One listed device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub default_device: bool,
}

/// Canned device lists for the test backend.
#[derive(Debug, Clone)]
pub struct TestBackend {
    playback: Vec<DeviceInfo>,
    capture: Vec<DeviceInfo>,
}

impl TestBackend {
    /// Two playback devices (first default) and one default capture
    /// device.
    pub fn standard() -> Self {
        Self {
            playback: vec![
                DeviceInfo {
                    name: "Test Playback".to_string(),
                    default_device: true,
                },
                DeviceInfo {
                    name: "Test Playback 2".to_string(),
                    default_device: false,
                },
            ],
            capture: vec![DeviceInfo {
                name: "Test Capture".to_string(),
                default_device: true,
            }],
        }
    }

    /// No playback devices: `start` reports [`AudioError::NoDevice`].
    pub fn no_playback() -> Self {
        Self {
            playback: Vec::new(),
            capture: vec![DeviceInfo {
                name: "Test Capture".to_string(),
                default_device: true,
            }],
        }
    }

    /// No capture devices.
    pub fn no_capture() -> Self {
        Self {
            playback: vec![DeviceInfo {
                name: "Test Playback".to_string(),
                default_device: true,
            }],
            capture: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct Sound {
    samples: Vec<f32>,
    channels: u8,
}

#[derive(Debug, Clone)]
struct Voice {
    sound: SoundId,
    position_frames: usize,
    volume: f32,
    pan: f32,
    group: GroupId,
    looping: bool,
}

#[derive(Debug)]
struct Stream {
    channels: u8,
    queued: VecDeque<f32>,
    volume: f32,
    pan: f32,
    group: GroupId,
    closed: bool,
    frames_written: u64,
    frames_consumed: u64,
}

/// Capture ring statistics (MED-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureStats {
    pub frames_received: u64,
    pub frames_read: u64,
    pub frames_dropped: u64,
    pub buffered_frames: u32,
    pub capacity_frames: u32,
}

#[derive(Debug)]
struct CaptureBuffer {
    channels: u8,
    capacity_frames: u32,
    data: VecDeque<f32>,
    received: u64,
    read: u64,
    dropped: u64,
}

impl CaptureBuffer {
    fn new(channels: u8, capacity_frames: u32) -> Result<Self, AudioError> {
        if !(1..=8).contains(&channels) || capacity_frames == 0 {
            return Err(AudioError::InvalidArgument);
        }
        if capacity_frames > MAX_CAPTURE_FRAMES {
            return Err(AudioError::InvalidArgument);
        }
        Ok(Self {
            channels,
            capacity_frames,
            data: VecDeque::new(),
            received: 0,
            read: 0,
            dropped: 0,
        })
    }

    /// Record delivered frames. Overflow drops the newest complete
    /// frames (oldest are preserved) and counts them.
    fn record(&mut self, samples: &[f32]) -> Result<(), AudioError> {
        if !samples.len().is_multiple_of(usize::from(self.channels)) {
            return Err(AudioError::InvalidArgument);
        }
        let frames = samples.len() / usize::from(self.channels);
        self.received += frames as u64;
        let free = self.capacity_frames.saturating_sub(self.buffered_frames()) as usize;
        let keep = free.min(frames);
        self.data
            .extend(samples[..keep * usize::from(self.channels)].iter());
        self.dropped += (frames - keep) as u64;
        Ok(())
    }

    /// Read recorded frames in order. Returns completed frames; a
    /// smaller-than-one-frame destination is an error, an empty buffer
    /// reads zero and touches nothing.
    fn read(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        let channels = usize::from(self.channels);
        if output.len() < channels {
            return Err(AudioError::FrameTooSmall);
        }
        let frames = (output.len() / channels).min(self.buffered_frames() as usize);
        for slot in output[..frames * channels].iter_mut() {
            *slot = self.data.pop_front().unwrap_or(0.0);
        }
        self.read += frames as u64;
        Ok(frames)
    }

    fn buffered_frames(&self) -> u32 {
        (self.data.len() / usize::from(self.channels)) as u32
    }

    fn stats(&self) -> CaptureStats {
        CaptureStats {
            frames_received: self.received,
            frames_read: self.read,
            frames_dropped: self.dropped,
            buffered_frames: self.buffered_frames(),
            capacity_frames: self.capacity_frames,
        }
    }
}

/// Engine-wide counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EngineStats {
    pub sounds_loaded: u32,
    pub voices_active: u32,
    pub streams_open: u32,
    pub frames_mixed: u64,
}

/// Per-stream counters (MED-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StreamStats {
    pub frames_written: u64,
    pub frames_consumed: u64,
    pub buffered_frames: u32,
}

/// Voice launch options.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayOptions {
    pub volume: f32,
    pub pan: f32,
    pub group: GroupId,
    pub looping: bool,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            group: DEFAULT_GROUP,
            looping: false,
        }
    }
}

/// Stream construction options.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamOptions {
    pub channels: u8,
    pub volume: f32,
    pub pan: f32,
    pub group: GroupId,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            channels: 2,
            volume: 1.0,
            pan: 0.0,
            group: DEFAULT_GROUP,
        }
    }
}

fn clamp_volume(v: f32) -> f32 {
    v.clamp(0.0, MAX_VOLUME)
}

fn clamp_pan(p: f32) -> f32 {
    p.clamp(-1.0, 1.0)
}

/// Linear-balance pan gains: full left at -1, equal at 0, full right
/// at +1.
fn pan_gains(pan: f32) -> (f32, f32) {
    let pan = clamp_pan(pan);
    if pan >= 0.0 {
        (1.0 - pan, 1.0)
    } else {
        (1.0, 1.0 + pan)
    }
}

/// Caller-owned deterministic audio engine (test backend built in).
pub struct AudioEngine {
    sample_rate: u32,
    output_channels: u8,
    backend: TestBackend,
    started: bool,
    has_device: bool,
    selected_playback: Option<usize>,
    selected_capture: Option<usize>,
    sounds: HashMap<SoundId, Sound>,
    next_sound_id: SoundId,
    voices: HashMap<VoiceId, Voice>,
    next_voice_id: VoiceId,
    group_names: HashMap<String, GroupId>,
    group_volumes: HashMap<GroupId, f32>,
    next_group_id: GroupId,
    master_volume: f32,
    streams: HashMap<StreamId, Stream>,
    next_stream_id: StreamId,
    capture: Option<CaptureBuffer>,
    capture_running: bool,
    tap: Option<TapBuffer>,
    tap_enabled: bool,
    frames_mixed: u64,
}

#[derive(Debug)]
struct TapBuffer {
    channels: u8,
    capacity_frames: u32,
    data: VecDeque<f32>,
}

impl AudioEngine {
    /// Create an engine. Zero rates or channel counts are rejected.
    pub fn new(options: EngineOptions, backend: TestBackend) -> Result<Self, AudioError> {
        if options.sample_rate == 0 || options.playback_channels == 0 {
            return Err(AudioError::InvalidArgument);
        }
        Ok(Self {
            sample_rate: options.sample_rate,
            output_channels: options.playback_channels,
            backend,
            started: false,
            has_device: false,
            selected_playback: None,
            selected_capture: None,
            sounds: HashMap::new(),
            next_sound_id: 1,
            voices: HashMap::new(),
            next_voice_id: 1,
            group_names: HashMap::new(),
            group_volumes: HashMap::new(),
            next_group_id: 1,
            master_volume: 1.0,
            streams: HashMap::new(),
            next_stream_id: 1,
            capture: None,
            capture_running: false,
            tap: None,
            tap_enabled: false,
            frames_mixed: 0,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn output_channels(&self) -> u8 {
        self.output_channels
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn has_device(&self) -> bool {
        self.has_device
    }

    /// Start hardware playback. Requires a selected or default
    /// playback device (MED-003).
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.started {
            return Ok(());
        }
        if self.selected_playback.is_some() {
            self.started = true;
            self.has_device = true;
            return Ok(());
        }
        if self.backend.playback.iter().any(|d| d.default_device) {
            self.started = true;
            self.has_device = true;
            return Ok(());
        }
        Err(AudioError::NoDevice)
    }

    /// Enable mixing without any playback device (MED-003).
    pub fn start_mixer(&mut self) -> Result<(), AudioError> {
        self.started = true;
        self.has_device = false;
        Ok(())
    }

    /// Stop playback. Safe on an unstarted engine (MED-003).
    pub fn stop(&mut self) -> Result<(), AudioError> {
        self.started = false;
        self.has_device = false;
        Ok(())
    }

    // -- devices (MED-005) --

    /// Re-list backend devices. The test backend is static, so repeat
    /// listings agree.
    pub fn refresh_devices(&mut self) -> Result<(), AudioError> {
        let selected_ok = self
            .selected_playback
            .is_none_or(|i| i < self.backend.playback.len());
        if !selected_ok {
            self.selected_playback = None;
        }
        let capture_ok = self
            .selected_capture
            .is_none_or(|i| i < self.backend.capture.len());
        if !capture_ok {
            self.selected_capture = None;
        }
        Ok(())
    }

    pub fn playback_devices(&self) -> &[DeviceInfo] {
        &self.backend.playback
    }

    pub fn capture_devices(&self) -> &[DeviceInfo] {
        &self.backend.capture
    }

    pub fn select_playback(&mut self, index: usize) -> Result<(), AudioError> {
        if index >= self.backend.playback.len() {
            return Err(AudioError::InvalidArgument);
        }
        self.selected_playback = Some(index);
        Ok(())
    }

    pub fn clear_playback_selection(&mut self) {
        self.selected_playback = None;
    }

    pub fn selected_playback(&self) -> Option<usize> {
        self.selected_playback
    }

    pub fn select_capture(&mut self, index: usize) -> Result<(), AudioError> {
        if index >= self.backend.capture.len() {
            return Err(AudioError::InvalidArgument);
        }
        self.selected_capture = Some(index);
        Ok(())
    }

    pub fn clear_capture_selection(&mut self) {
        self.selected_capture = None;
    }

    pub fn selected_capture(&self) -> Option<usize> {
        self.selected_capture
    }

    // -- sounds (MED-007) --

    /// Load interleaved f32 PCM. Ids are monotonic and never reused.
    pub fn load_pcm(
        &mut self,
        samples: Vec<f32>,
        channels: u8,
        _sample_rate: u32,
    ) -> Result<SoundId, AudioError> {
        if samples.is_empty() || !(1..=8).contains(&channels) {
            return Err(AudioError::InvalidArgument);
        }
        if !samples.len().is_multiple_of(usize::from(channels)) {
            return Err(AudioError::InvalidArgument);
        }
        let id = self.next_sound_id;
        self.next_sound_id += 1;
        self.sounds.insert(id, Sound { samples, channels });
        Ok(id)
    }

    /// Load a PCM-16 or float32 WAV file.
    pub fn load_wav(&mut self, bytes: &[u8]) -> Result<SoundId, AudioError> {
        let (samples, channels) = decode_wav(bytes)?;
        self.load_pcm(samples, channels, self.sample_rate)
    }

    /// Unload a sound, stopping its active voices. Unknown ids fail;
    /// unloaded ids are never reused (MED-007).
    pub fn unload_sound(&mut self, id: SoundId) -> Result<(), AudioError> {
        if self.sounds.remove(&id).is_none() {
            return Err(AudioError::UnknownId);
        }
        self.voices.retain(|_, voice| voice.sound != id);
        Ok(())
    }

    pub fn sound_count(&self) -> usize {
        self.sounds.len()
    }

    /// Group 0 always exists; named groups come from [`AudioEngine::create_group`].
    fn group_exists(&self, group: GroupId) -> bool {
        group == DEFAULT_GROUP || self.group_volumes.contains_key(&group)
    }

    /// Play a loaded sound, returning a voice id.
    pub fn play(&mut self, sound: SoundId, options: PlayOptions) -> Result<VoiceId, AudioError> {
        if !self.sounds.contains_key(&sound) {
            return Err(AudioError::UnknownId);
        }
        if !self.group_exists(options.group) {
            return Err(AudioError::UnknownId);
        }
        let id = self.next_voice_id;
        self.next_voice_id += 1;
        self.voices.insert(
            id,
            Voice {
                sound,
                position_frames: 0,
                volume: clamp_volume(options.volume),
                pan: clamp_pan(options.pan),
                group: options.group,
                looping: options.looping,
            },
        );
        Ok(id)
    }

    /// Stop an active voice.
    pub fn stop_voice(&mut self, id: VoiceId) -> Result<(), AudioError> {
        self.voices
            .remove(&id)
            .map(|_| ())
            .ok_or(AudioError::UnknownId)
    }

    /// Move a voice between groups. Unknown voices and groups fail.
    pub fn set_voice_group(&mut self, id: VoiceId, group: GroupId) -> Result<(), AudioError> {
        if !self.group_exists(group) {
            return Err(AudioError::UnknownId);
        }
        let voice = self.voices.get_mut(&id).ok_or(AudioError::UnknownId)?;
        voice.group = group;
        Ok(())
    }

    /// The group a voice currently belongs to.
    pub fn voice_group(&self, id: VoiceId) -> Option<GroupId> {
        self.voices.get(&id).map(|voice| voice.group)
    }

    /// Whether a voice is still active.
    pub fn voice_active(&self, id: VoiceId) -> bool {
        self.voices.contains_key(&id)
    }

    /// Create (or deduplicate) a named group at full volume.
    pub fn create_group(&mut self, name: &str) -> Result<GroupId, AudioError> {
        if name.is_empty() {
            return Err(AudioError::InvalidArgument);
        }
        if let Some(id) = self.group_names.get(name) {
            return Ok(*id);
        }
        let id = self.next_group_id;
        self.next_group_id += 1;
        self.group_names.insert(name.to_string(), id);
        self.group_volumes.insert(id, 1.0);
        Ok(id)
    }

    /// Set a group volume, clamped to silence..[`MAX_VOLUME`].
    pub fn set_group_volume(&mut self, group: GroupId, volume: f32) -> Result<(), AudioError> {
        if group == DEFAULT_GROUP {
            self.group_volumes
                .insert(DEFAULT_GROUP, clamp_volume(volume));
            return Ok(());
        }
        if !self.group_volumes.contains_key(&group) {
            return Err(AudioError::UnknownId);
        }
        self.group_volumes.insert(group, clamp_volume(volume));
        Ok(())
    }

    /// Set the master volume, clamped to silence..[`MAX_VOLUME`].
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = clamp_volume(volume);
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn group_volume(&self, group: GroupId) -> Option<f32> {
        if group == DEFAULT_GROUP {
            Some(
                self.group_volumes
                    .get(&DEFAULT_GROUP)
                    .copied()
                    .unwrap_or(1.0),
            )
        } else {
            self.group_volumes.get(&group).copied()
        }
    }

    fn effective_gain(&self, volume: f32, group: GroupId) -> f32 {
        volume * self.group_volume(group).unwrap_or(1.0) * self.master_volume
    }

    // -- streams (MED-004) --

    /// Open a PCM stream. Unknown groups and bad channel counts fail
    /// before anything is allocated.
    pub fn create_stream(&mut self, options: StreamOptions) -> Result<StreamId, AudioError> {
        if !(1..=8).contains(&options.channels) {
            return Err(AudioError::InvalidArgument);
        }
        if !self.group_exists(options.group) {
            return Err(AudioError::UnknownId);
        }
        let id = self.next_stream_id;
        self.next_stream_id += 1;
        self.streams.insert(
            id,
            Stream {
                channels: options.channels,
                queued: VecDeque::new(),
                volume: clamp_volume(options.volume),
                pan: clamp_pan(options.pan),
                group: options.group,
                closed: false,
                frames_written: 0,
                frames_consumed: 0,
            },
        );
        Ok(id)
    }

    /// Write interleaved samples. Every sample is accounted; writes
    /// after close fail; partial frames fail.
    pub fn write_stream(&mut self, id: StreamId, samples: &[f32]) -> Result<usize, AudioError> {
        let stream = self.streams.get_mut(&id).ok_or(AudioError::UnknownId)?;
        if stream.closed {
            return Err(AudioError::Closed);
        }
        if !samples.len().is_multiple_of(usize::from(stream.channels)) {
            return Err(AudioError::InvalidArgument);
        }
        let frames = samples.len() / usize::from(stream.channels);
        stream.queued.extend(samples.iter());
        stream.frames_written += frames as u64;
        Ok(frames)
    }

    /// Close a stream. Later writes fail; queued audio still mixes.
    pub fn close_stream(&mut self, id: StreamId) -> Result<(), AudioError> {
        let stream = self.streams.get_mut(&id).ok_or(AudioError::UnknownId)?;
        stream.closed = true;
        Ok(())
    }

    pub fn stream_stats(&self, id: StreamId) -> Result<StreamStats, AudioError> {
        let stream = self.streams.get(&id).ok_or(AudioError::UnknownId)?;
        Ok(StreamStats {
            frames_written: stream.frames_written,
            frames_consumed: stream.frames_consumed,
            buffered_frames: (stream.queued.len() / usize::from(stream.channels)) as u32,
        })
    }

    pub fn stream_closed(&self, id: StreamId) -> Result<bool, AudioError> {
        self.streams
            .get(&id)
            .map(|stream| stream.closed)
            .ok_or(AudioError::UnknownId)
    }

    // -- mixer --

    /// Mix voices and streams into interleaved output. Mono output
    /// averages the stereo pair clamped to unity; channels beyond
    /// stereo stay zero. Finished one-shot voices retire; looping
    /// voices wrap. Returns mixed frames.
    pub fn mix_to_buffer(&mut self, output: &mut [f32], channels: u8) -> Result<usize, AudioError> {
        if !(1..=8).contains(&channels) || output.is_empty() {
            return Err(AudioError::InvalidArgument);
        }
        let out_channels = usize::from(channels);
        if !output.len().is_multiple_of(out_channels) {
            return Err(AudioError::InvalidArgument);
        }
        let frames = output.len() / out_channels;
        output.fill(0.0);

        // Voices.
        let mut finished = Vec::new();
        let voice_ids: Vec<VoiceId> = self.voices.keys().copied().collect();
        for id in voice_ids {
            let Some(voice) = self.voices.get(&id).cloned() else {
                continue;
            };
            let Some(sound) = self.sounds.get(&voice.sound).cloned() else {
                finished.push(id);
                continue;
            };
            let sound_frames = sound.samples.len() / usize::from(sound.channels);
            let gain = self.effective_gain(voice.volume, voice.group);
            let (pan_l, pan_r) = pan_gains(voice.pan);
            let mut position = voice.position_frames;
            for frame in 0..frames {
                if position >= sound_frames {
                    if voice.looping && sound_frames > 0 {
                        position = 0;
                    } else {
                        break;
                    }
                }
                let base = position * usize::from(sound.channels);
                let (left, right) = if sound.channels == 1 {
                    let s = sound.samples[base] * gain;
                    (s * pan_l, s * pan_r)
                } else {
                    // Stereo pan narrows the opposite side.
                    let l = sound.samples[base] * gain * pan_l;
                    let r = sound.samples[base + 1] * gain * pan_r;
                    (l, r)
                };
                let slot = frame * out_channels;
                if out_channels == 1 {
                    output[slot] += (left + right) * 0.5;
                } else {
                    output[slot] += left;
                    output[slot + 1] += right;
                }
                position += 1;
            }
            if position >= sound_frames && !voice.looping {
                finished.push(id);
            } else if let Some(voice) = self.voices.get_mut(&id) {
                voice.position_frames = position % sound_frames.max(1);
            }
        }
        for id in finished {
            self.voices.remove(&id);
        }

        // Streams consume their queues in order.
        let stream_ids: Vec<StreamId> = self.streams.keys().copied().collect();
        for id in stream_ids {
            let Some((volume, pan, group, channels)) = self
                .streams
                .get(&id)
                .map(|s| (s.volume, s.pan, s.group, s.channels))
            else {
                continue;
            };
            let gain = self.effective_gain(volume, group);
            let (pan_l, pan_r) = pan_gains(pan);
            let Some(stream) = self.streams.get_mut(&id) else {
                continue;
            };
            let src_channels = usize::from(channels);
            let available = stream.queued.len() / src_channels;
            let count = available.min(frames);
            for frame in 0..count {
                let pair = if stream.channels == 1 {
                    let sample = stream.queued.pop_front().unwrap_or(0.0);
                    [sample * gain * pan_l, sample * gain * pan_r]
                } else {
                    let l = stream.queued.pop_front().unwrap_or(0.0);
                    let r = stream.queued.pop_front().unwrap_or(0.0);
                    // Drop extra source channels beyond stereo.
                    for _ in 2..src_channels {
                        stream.queued.pop_front();
                    }
                    [l * gain * pan_l, r * gain * pan_r]
                };
                let slot = frame * out_channels;
                if out_channels == 1 {
                    output[slot] += (pair[0] + pair[1]) * 0.5;
                } else {
                    output[slot] += pair[0];
                    output[slot + 1] += pair[1];
                }
            }
            stream.frames_consumed += count as u64;
        }

        // Mono output averages with a unity clamp (reference behavior).
        if out_channels == 1 {
            for sample in output.iter_mut() {
                *sample = sample.clamp(-1.0, 1.0);
            }
        }

        // Tap captures mixed frames while enabled.
        if self.tap_enabled
            && let Some(tap) = self.tap.as_mut()
        {
            let width = usize::from(tap.channels);
            for frame in 0..frames {
                // Resample the mixed output to the tap width.
                for c in 0..width {
                    let sample = if out_channels == 1 {
                        output[frame]
                    } else if c < out_channels {
                        output[frame * out_channels + c]
                    } else {
                        0.0
                    };
                    if tap.data.len() >= tap.capacity_frames as usize * width {
                        for _ in 0..width {
                            tap.data.pop_front();
                        }
                    }
                    tap.data.push_back(sample);
                }
            }
        }

        self.frames_mixed += frames as u64;
        Ok(frames)
    }

    /// Capture mixed output while enabled.
    pub fn enable_tap(&mut self, enable: bool, capacity_frames: u32) -> Result<(), AudioError> {
        if enable {
            if !(1..=8).contains(&self.output_channels) || capacity_frames == 0 {
                return Err(AudioError::InvalidArgument);
            }
            self.tap = Some(TapBuffer {
                channels: self.output_channels,
                capacity_frames,
                data: VecDeque::new(),
            });
            self.tap_enabled = true;
        } else {
            self.tap_enabled = false;
        }
        Ok(())
    }

    /// Drain tapped frames in order. Returns completed frames.
    pub fn read_tap(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        let Some(tap) = self.tap.as_mut() else {
            return Ok(0);
        };
        let width = usize::from(tap.channels);
        if output.len() < width {
            return Err(AudioError::FrameTooSmall);
        }
        let frames = (output.len() / width).min(tap.data.len() / width);
        for slot in output[..frames * width].iter_mut() {
            *slot = tap.data.pop_front().unwrap_or(0.0);
        }
        Ok(frames)
    }

    pub fn engine_stats(&self) -> EngineStats {
        EngineStats {
            sounds_loaded: self.sounds.len() as u32,
            voices_active: self.voices.len() as u32,
            streams_open: self.streams.values().filter(|s| !s.closed).count() as u32,
            frames_mixed: self.frames_mixed,
        }
    }

    // -- capture (MED-006) --

    /// Open the capture buffer. Bad sizes fail before allocating.
    pub fn open_capture(&mut self, channels: u8, capacity_frames: u32) -> Result<(), AudioError> {
        self.capture = Some(CaptureBuffer::new(channels, capacity_frames)?);
        self.capture_running = true;
        Ok(())
    }

    /// Deliver recorded frames from the (test) hardware.
    pub fn record_capture(&mut self, samples: &[f32]) -> Result<(), AudioError> {
        let Some(capture) = self.capture.as_mut() else {
            return Err(AudioError::InvalidArgument);
        };
        capture.record(samples)
    }

    /// Read recorded frames in order. Storage survives stops so
    /// draining works after [`AudioEngine::stop_capture`].
    pub fn read_capture(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        let Some(capture) = self.capture.as_mut() else {
            return Err(AudioError::InvalidArgument);
        };
        capture.read(output)
    }

    /// Stop capturing. Idempotent; storage is preserved for draining.
    pub fn stop_capture(&mut self) -> Result<(), AudioError> {
        self.capture_running = false;
        Ok(())
    }

    pub fn capture_running(&self) -> bool {
        self.capture_running
    }

    pub fn capture_stats(&self) -> Result<CaptureStats, AudioError> {
        self.capture
            .as_ref()
            .map(CaptureBuffer::stats)
            .ok_or(AudioError::InvalidArgument)
    }
}

/// Minimal WAV reader: PCM-16 or IEEE-float mono/stereo.
fn decode_wav(bytes: &[u8]) -> Result<(Vec<f32>, u8), AudioError> {
    if bytes.len() < 44
        || &bytes[0..4] != b"RIFF"
        || &bytes[8..12] != b"WAVE"
        || &bytes[12..16] != b"fmt "
    {
        return Err(AudioError::InvalidData);
    }
    let u16le = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
    let u32le =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let format = u16le(20);
    let channels = u16le(22);
    let bits = u16le(34);
    if !(1..=2).contains(&channels) || (format != 1 && format != 3) {
        return Err(AudioError::InvalidData);
    }
    if (format == 1 && bits != 16) || (format == 3 && bits != 32) {
        return Err(AudioError::InvalidData);
    }
    // Find the data chunk (skip extension chunks after fmt).
    let mut at = 12 + 8 + u32le(16) as usize;
    loop {
        if at + 8 > bytes.len() {
            return Err(AudioError::InvalidData);
        }
        let size = u32le(at + 4) as usize;
        if &bytes[at..at + 4] == b"data" {
            at += 8;
            break;
        }
        at += 8 + size;
    }
    let stride = usize::from(channels) * usize::from(bits / 8);
    if bytes.len() < at + stride {
        return Err(AudioError::InvalidData);
    }
    let frames = (bytes.len() - at) / stride;
    let mut samples = Vec::with_capacity(frames * usize::from(channels));
    for frame in 0..frames {
        for c in 0..usize::from(channels) {
            let base = at + frame * stride + c * usize::from(bits / 8);
            if format == 1 {
                let v = i16::from_le_bytes([bytes[base], bytes[base + 1]]);
                samples.push(f32::from(v) / 32768.0);
            } else {
                samples.push(f32::from_le_bytes([
                    bytes[base],
                    bytes[base + 1],
                    bytes[base + 2],
                    bytes[base + 3],
                ]));
            }
        }
    }
    Ok((samples, channels as u8))
}
