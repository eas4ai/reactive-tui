//! media-audio commitment tests (MED-003 … MED-007), ported from
//! `tests/audio_test.zig` against the deterministic test backend.

use suprtui::audio::{
    AudioEngine, AudioError, DEFAULT_GROUP, EngineOptions, PlayOptions, StreamOptions, TestBackend,
};

fn make_engine() -> AudioEngine {
    AudioEngine::new(EngineOptions::default(), TestBackend::standard()).unwrap()
}

fn make_mixer() -> AudioEngine {
    let mut engine = make_engine();
    engine.start_mixer().unwrap();
    engine
}

fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((36 + samples.len() * 2) as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&44100u32.to_le_bytes());
    out.extend_from_slice(&88200u32.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&((samples.len() * 2) as u32).to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn has_signal(samples: &[f32]) -> bool {
    samples.iter().any(|s| s.abs() > 0.0)
}

// ---------------------------------------------------------------------------
// MED-003: fallible lifecycle, safe stops.
// ---------------------------------------------------------------------------

#[test]
fn req_003_engine_lifecycle() {
    // Defaults and custom options.
    let engine = make_engine();
    assert_eq!(engine.sample_rate(), 48_000);
    assert_eq!(engine.output_channels(), 2);
    assert!(!engine.is_started());

    let custom = AudioEngine::new(
        EngineOptions {
            sample_rate: 44_100,
            playback_channels: 4,
        },
        TestBackend::standard(),
    )
    .unwrap();
    assert_eq!(custom.sample_rate(), 44_100);
    assert_eq!(custom.output_channels(), 4);

    // Invalid construction fails instead of panicking.
    assert!(matches!(
        AudioEngine::new(
            EngineOptions {
                sample_rate: 0,
                playback_channels: 2
            },
            TestBackend::standard()
        ),
        Err(AudioError::InvalidArgument)
    ));
    assert!(matches!(
        AudioEngine::new(
            EngineOptions {
                sample_rate: 48_000,
                playback_channels: 0
            },
            TestBackend::standard()
        ),
        Err(AudioError::InvalidArgument)
    ));

    // Stopping an unstarted engine is safe.
    let mut engine = make_engine();
    engine.stop().unwrap();
    assert!(!engine.is_started());

    // Start finds the default playback device.
    engine.start().unwrap();
    assert!(engine.is_started());
    assert!(engine.has_device());
    engine.stop().unwrap();
    assert!(!engine.is_started());
    assert!(!engine.has_device());

    // No playback device: start reports, mixer still works.
    let mut bare = AudioEngine::new(EngineOptions::default(), TestBackend::no_playback()).unwrap();
    assert_eq!(bare.start(), Err(AudioError::NoDevice));
    assert!(!bare.is_started());
    assert!(!bare.has_device());
    bare.start_mixer().unwrap();
    assert!(bare.is_started());
    assert!(!bare.has_device());
    bare.stop().unwrap();
    assert!(!bare.is_started());
}

// ---------------------------------------------------------------------------
// MED-004: streams account, shape, and close.
// ---------------------------------------------------------------------------

#[test]
fn req_004_streams() {
    // Preflight rejects bad channels and unknown groups.
    let mut engine = make_mixer();
    assert_eq!(
        engine.create_stream(StreamOptions {
            channels: 0,
            ..StreamOptions::default()
        }),
        Err(AudioError::InvalidArgument)
    );
    assert_eq!(
        engine.create_stream(StreamOptions {
            group: u32::MAX,
            ..StreamOptions::default()
        }),
        Err(AudioError::UnknownId)
    );

    // Writes are accounted and consumed in order.
    let id = engine.create_stream(StreamOptions::default()).unwrap();
    assert_eq!(engine.write_stream(id, &[0.5, -0.5, 0.25, -0.25]), Ok(2));
    let stats = engine.stream_stats(id).unwrap();
    assert_eq!(stats.frames_written, 2);
    assert_eq!(stats.frames_consumed, 0);
    assert_eq!(stats.buffered_frames, 2);
    let mut out = [0.0f32; 4];
    assert_eq!(engine.mix_to_buffer(&mut out, 2), Ok(2));
    let stats = engine.stream_stats(id).unwrap();
    assert_eq!(stats.frames_written, 2);
    assert_eq!(stats.frames_consumed, 2);
    assert_eq!(stats.buffered_frames, 0);

    // Partial frames fail; unknown streams fail.
    assert_eq!(
        engine.write_stream(id, &[1.0]),
        Err(AudioError::InvalidArgument)
    );
    assert_eq!(
        engine.write_stream(999, &[1.0, 1.0]),
        Err(AudioError::UnknownId)
    );

    // Volume applies.
    let mut engine = make_mixer();
    let quiet = engine
        .create_stream(StreamOptions {
            volume: 0.2,
            ..StreamOptions::default()
        })
        .unwrap();
    engine.write_stream(quiet, &[1.0, 1.0]).unwrap();
    let mut out = [0.0f32; 2];
    engine.mix_to_buffer(&mut out, 2).unwrap();
    assert!((out[0] - 0.2).abs() < 0.0001, "volume {}", out[0]);
    assert!((out[1] - 0.2).abs() < 0.0001);

    // Pan steers a mono source across the pair.
    let mut engine = make_mixer();
    let right = engine
        .create_stream(StreamOptions {
            pan: 1.0,
            ..StreamOptions::default()
        })
        .unwrap();
    engine.write_stream(right, &[1.0, 1.0]).unwrap();
    let mut out = [0.0f32; 2];
    engine.mix_to_buffer(&mut out, 2).unwrap();
    assert!((out[0]).abs() < 0.0001, "pan left {}", out[0]);
    assert!((out[1] - 1.0).abs() < 0.0001, "pan right {}", out[1]);

    // Group silence applies.
    let mut engine = make_mixer();
    let group = engine.create_group("fx").unwrap();
    let voice = engine
        .create_stream(StreamOptions {
            group,
            ..StreamOptions::default()
        })
        .unwrap();
    engine.write_stream(voice, &[1.0, 1.0]).unwrap();
    engine.set_group_volume(group, 0.0).unwrap();
    let mut out = [0.0f32; 2];
    engine.mix_to_buffer(&mut out, 2).unwrap();
    assert_eq!(out, [0.0, 0.0]);

    // Close rejects later writes; queued audio still mixes.
    let mut engine = make_mixer();
    let id = engine.create_stream(StreamOptions::default()).unwrap();
    engine.write_stream(id, &[0.5, 0.5]).unwrap();
    engine.close_stream(id).unwrap();
    assert_eq!(engine.stream_closed(id), Ok(true));
    assert_eq!(
        engine.write_stream(id, &[0.5, 0.5]),
        Err(AudioError::Closed)
    );
    let mut out = [0.0f32; 2];
    engine.mix_to_buffer(&mut out, 2).unwrap();
    assert!((out[0] - 0.5).abs() < 0.0001);
    assert_eq!(engine.close_stream(999), Err(AudioError::UnknownId));
}

// ---------------------------------------------------------------------------
// MED-005: devices list, agree, and select.
// ---------------------------------------------------------------------------

#[test]
fn req_005_devices() {
    let mut engine = make_engine();
    engine.refresh_devices().unwrap();

    let playback = engine.playback_devices();
    assert_eq!(playback.len(), 2);
    assert!(!playback[0].name.is_empty());
    assert!(playback[0].default_device);
    let capture = engine.capture_devices();
    assert_eq!(capture.len(), 1);
    assert!(capture[0].default_device);

    // Repeat listings of the unchanged backend agree.
    let first_name = playback[0].name.clone();
    engine.refresh_devices().unwrap();
    assert_eq!(engine.playback_devices().len(), 2);
    assert_eq!(engine.playback_devices()[0].name, first_name);
    assert_eq!(engine.capture_devices().len(), 1);

    // Selection round-trips; out-of-range fails.
    engine.select_playback(0).unwrap();
    assert_eq!(engine.selected_playback(), Some(0));
    engine.clear_playback_selection();
    assert_eq!(engine.selected_playback(), None);
    // Exact boundary: len is out of range, len - 1 is valid.
    assert_eq!(engine.select_playback(2), Err(AudioError::InvalidArgument));
    engine.select_playback(1).unwrap();
    assert_eq!(engine.selected_playback(), Some(1));
    assert_eq!(engine.select_playback(99), Err(AudioError::InvalidArgument));

    engine.select_capture(0).unwrap();
    assert_eq!(engine.selected_capture(), Some(0));
    engine.clear_capture_selection();
    assert_eq!(engine.selected_capture(), None);
    assert_eq!(engine.select_capture(1), Err(AudioError::InvalidArgument));
    assert_eq!(engine.select_capture(7), Err(AudioError::InvalidArgument));
}

// ---------------------------------------------------------------------------
// MED-006: capture delivers recorded frames with matching stats.
// ---------------------------------------------------------------------------

#[test]
fn req_006_capture() {
    // Exact interleaved order with partial and empty reads.
    let mut engine = make_engine();
    engine.open_capture(2, 5).unwrap();
    engine
        .record_capture(&[1.0, 10.0, 2.0, 20.0, 3.0, 30.0])
        .unwrap();

    let mut output = [99.0f32; 8];
    assert_eq!(engine.read_capture(&mut output[..4]), Ok(2));
    assert_eq!(output[..4], [1.0, 10.0, 2.0, 20.0]);
    assert_eq!(output[4], 99.0);

    output.fill(99.0);
    assert_eq!(engine.read_capture(&mut output[..8]), Ok(1));
    assert_eq!(output[..2], [3.0, 30.0]);
    assert_eq!(output[2], 99.0);

    output.fill(77.0);
    assert_eq!(engine.read_capture(&mut output[..8]), Ok(0));
    assert_eq!(output[0], 77.0);

    let stats = engine.capture_stats().unwrap();
    assert_eq!(stats.frames_received, 3);
    assert_eq!(stats.frames_read, 3);
    assert_eq!(stats.frames_dropped, 0);
    assert_eq!(stats.buffered_frames, 0);
    assert_eq!(stats.capacity_frames, 5);

    // Order survives the ring wrap.
    let mut engine = make_engine();
    engine.open_capture(1, 4).unwrap();
    engine.record_capture(&[1.0, 2.0, 3.0]).unwrap();
    let mut prefix = [0.0f32; 2];
    assert_eq!(engine.read_capture(&mut prefix), Ok(2));
    assert_eq!(prefix, [1.0, 2.0]);
    engine.record_capture(&[4.0, 5.0, 6.0]).unwrap();
    let mut output = [0.0f32; 4];
    assert_eq!(engine.read_capture(&mut output), Ok(4));
    assert_eq!(output, [3.0, 4.0, 5.0, 6.0]);

    // Overflow drops the newest complete frames and counts them.
    let mut engine = make_engine();
    engine.open_capture(2, 3).unwrap();
    engine
        .record_capture(&[1.0, 10.0, 2.0, 20.0, 3.0, 30.0, 4.0, 40.0, 5.0, 50.0])
        .unwrap();
    let mut prefix = [0.0f32; 2];
    assert_eq!(engine.read_capture(&mut prefix), Ok(1));
    assert_eq!(prefix, [1.0, 10.0]);
    engine.record_capture(&[6.0, 60.0, 7.0, 70.0]).unwrap();
    let mut output = [0.0f32; 6];
    assert_eq!(engine.read_capture(&mut output), Ok(3));
    assert_eq!(output, [2.0, 20.0, 3.0, 30.0, 6.0, 60.0]);
    let stats = engine.capture_stats().unwrap();
    assert_eq!(stats.frames_received, 7);
    assert_eq!(stats.frames_read, 4);
    assert_eq!(stats.frames_dropped, 3);
    assert_eq!(stats.buffered_frames, 0);

    // Bad sizes fail before allocating; small destinations fail.
    let mut engine = make_engine();
    assert_eq!(engine.open_capture(0, 4), Err(AudioError::InvalidArgument));
    assert_eq!(engine.open_capture(2, 0), Err(AudioError::InvalidArgument));
    assert_eq!(
        engine.open_capture(2, u32::MAX),
        Err(AudioError::InvalidArgument)
    );
    engine.open_capture(2, 4).unwrap();
    assert_eq!(
        engine.record_capture(&[1.0]),
        Err(AudioError::InvalidArgument)
    );
    assert_eq!(
        engine.read_capture(&mut [0.0f32; 1]),
        Err(AudioError::FrameTooSmall)
    );

    // Stop is idempotent and preserves storage for draining.
    engine.record_capture(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    engine.stop_capture().unwrap();
    engine.stop_capture().unwrap();
    assert!(!engine.capture_running());
    let mut output = [0.0f32; 4];
    assert_eq!(engine.read_capture(&mut output), Ok(2));
    assert_eq!(output, [1.0, 2.0, 3.0, 4.0]);
}

// ---------------------------------------------------------------------------
// MED-007: id-addressed sounds, voices, groups, mixer, tap.
// ---------------------------------------------------------------------------

#[test]
fn req_007_sound_ids() {
    // Ids are monotonic and never reused.
    let mut engine = make_mixer();
    let first = engine.load_pcm(vec![0.0, 0.5], 1, 48_000).unwrap();
    let second = engine.load_pcm(vec![0.0, 0.5], 1, 48_000).unwrap();
    assert!(first > 0 && second == first + 1);
    engine.unload_sound(first).unwrap();
    assert_eq!(engine.sound_count(), 1);
    assert_eq!(engine.unload_sound(first), Err(AudioError::UnknownId));
    let third = engine.load_pcm(vec![0.0, 0.5], 1, 48_000).unwrap();
    assert!(third > second);
    assert_eq!(
        engine.load_pcm(vec![], 1, 48_000),
        Err(AudioError::InvalidArgument)
    );
    assert_eq!(
        engine.load_pcm(vec![1.0], 0, 48_000),
        Err(AudioError::InvalidArgument)
    );

    // WAV loading works; corrupt bytes fail.
    let wav = wav_bytes(&[0, 8000, -8000, 12_000, -12_000, 0]);
    let wav_id = engine.load_wav(&wav).unwrap();
    assert!(engine.sound_count() >= 3);
    assert_eq!(engine.load_wav(b"RIFFxxxx"), Err(AudioError::InvalidData));
    let _ = wav_id;

    // Play/stop voices; unload stops active voices.
    let voice = engine
        .play(
            second,
            PlayOptions {
                looping: true,
                ..PlayOptions::default()
            },
        )
        .unwrap();
    assert!(voice > 0);
    assert!(engine.voice_active(voice));
    assert_eq!(
        engine.play(999, PlayOptions::default()),
        Err(AudioError::UnknownId)
    );
    engine.stop_voice(voice).unwrap();
    assert!(!engine.voice_active(voice));
    assert_eq!(engine.stop_voice(voice), Err(AudioError::UnknownId));

    let doomed = engine.play(second, PlayOptions::default()).unwrap();
    engine.unload_sound(second).unwrap();
    assert!(!engine.voice_active(doomed));

    // Groups: create/dedup, move, clamped volumes.
    let group_a = engine.create_group("group-a").unwrap();
    let group_b = engine.create_group("group-b").unwrap();
    assert_eq!(engine.create_group("group-a").unwrap(), group_a);
    assert_eq!(engine.create_group(""), Err(AudioError::InvalidArgument));
    let sound = engine.load_pcm(vec![0.5, 0.5], 1, 48_000).unwrap();
    let voice = engine
        .play(
            sound,
            PlayOptions {
                group: group_a,
                looping: true,
                ..PlayOptions::default()
            },
        )
        .unwrap();
    engine.set_voice_group(voice, group_b).unwrap();
    assert_eq!(engine.voice_group(voice), Some(group_b));
    assert_eq!(
        engine.set_voice_group(voice, 999),
        Err(AudioError::UnknownId)
    );
    assert_eq!(
        engine.set_voice_group(999, group_b),
        Err(AudioError::UnknownId)
    );
    engine.set_group_volume(group_b, 2.5).unwrap();
    assert!((engine.group_volume(group_b).unwrap() - 2.5).abs() < 0.0001);
    engine.set_group_volume(group_b, 8.0).unwrap();
    assert!((engine.group_volume(group_b).unwrap() - 4.0).abs() < 0.0001);
    assert_eq!(
        engine.set_group_volume(999, 1.0),
        Err(AudioError::UnknownId)
    );
    engine.set_master_volume(1.7);
    assert!((engine.master_volume() - 1.7).abs() < 0.0001);
    engine.set_master_volume(-3.0);
    assert_eq!(engine.master_volume(), 0.0);
    engine.set_master_volume(1.0);

    // Mixed output carries signal.
    let mut mixed = [0.0f32; 128];
    assert_eq!(engine.mix_to_buffer(&mut mixed, 2), Ok(64));
    assert!(has_signal(&mixed));

    // Mono downmix averages the stereo pair with a unity clamp.
    let mut stereo_engine = make_mixer();
    let mut mono_engine = make_mixer();
    let stereo_sound = stereo_engine
        .load_pcm(vec![0.5, -0.5, 0.8, -0.8, 0.5, -0.5], 2, 48_000)
        .unwrap();
    let mono_sound = mono_engine
        .load_pcm(vec![0.5, -0.5, 0.8, -0.8, 0.5, -0.5], 2, 48_000)
        .unwrap();
    stereo_engine
        .play(
            stereo_sound,
            PlayOptions {
                looping: true,
                ..PlayOptions::default()
            },
        )
        .unwrap();
    mono_engine
        .play(
            mono_sound,
            PlayOptions {
                looping: true,
                ..PlayOptions::default()
            },
        )
        .unwrap();
    let mut stereo = [0.0f32; 128];
    let mut mono = [0.0f32; 64];
    stereo_engine.mix_to_buffer(&mut stereo, 2).unwrap();
    mono_engine.mix_to_buffer(&mut mono, 1).unwrap();
    for i in 0..64 {
        let expected = ((stereo[i * 2] + stereo[i * 2 + 1]) * 0.5).clamp(-1.0, 1.0);
        assert!((mono[i] - expected).abs() < 0.0001, "frame {i}");
    }

    // Extra channels stay zero.
    let mut quad = [0.0f32; 256];
    stereo_engine.mix_to_buffer(&mut quad, 4).unwrap();
    for frame in 0..64 {
        assert_eq!(quad[frame * 4 + 2], 0.0);
        assert_eq!(quad[frame * 4 + 3], 0.0);
    }

    // Tap captures mixed frames.
    stereo_engine.enable_tap(true, 256).unwrap();
    let mut warm = [0.0f32; 256];
    stereo_engine.mix_to_buffer(&mut warm, 2).unwrap();
    let mut tapped = [0.0f32; 128];
    let frames = stereo_engine.read_tap(&mut tapped).unwrap();
    assert!(frames > 0);
    assert!(has_signal(&tapped[..frames * 2]));
    stereo_engine.enable_tap(false, 0).unwrap();

    // Stats report the live counters.
    let stats = stereo_engine.engine_stats();
    assert!(stats.sounds_loaded >= 1);
    assert!(stats.voices_active >= 1);
    assert!(stats.frames_mixed > 0);

    // The default group always exists.
    assert_eq!(engine.group_volume(DEFAULT_GROUP), Some(1.0));
}
