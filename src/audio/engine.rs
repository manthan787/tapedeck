use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::audio::buffer::SharedBuffers;
use crate::audio::mixer::MixerState;
use crate::audio::transport::Transport;
use crate::constants::{SAMPLE_RATE, TRACK_COUNT, TRACK_SAMPLES};
use crate::effects;
use crate::messages::{AudioCmd, AudioMsg, RecordSource};
use crate::sequencer::clock::SequencerClock;
use crate::sequencer::drum_kit::DrumKit;
use crate::synth::engines;
use crate::synth::SynthEngine;
use crate::tape::simulation::TapeSimulation;

fn detect_loop_end(buffers: &SharedBuffers, exclude_track: Option<usize>) -> Option<usize> {
    let mut max_len = 0usize;
    for (idx, track) in buffers.tracks.iter().enumerate() {
        if Some(idx) == exclude_track {
            continue;
        }
        let len = track.len.load(Ordering::Relaxed).min(TRACK_SAMPLES);
        max_len = max_len.max(len);
    }
    (max_len > 0).then_some(max_len)
}

fn samples_per_beat(bpm: f32) -> usize {
    ((SAMPLE_RATE as f32 * 60.0 / bpm.clamp(40.0, 300.0)) as usize).max(1)
}

struct LevelMeter {
    sum_sq: f32,
    count: usize,
    peak: f32,
}

impl LevelMeter {
    fn new() -> Self {
        Self {
            sum_sq: 0.0,
            count: 0,
            peak: 0.0,
        }
    }

    fn push(&mut self, sample: f32) {
        self.sum_sq += sample * sample;
        self.count += 1;
        let abs = sample.abs();
        if abs > self.peak {
            self.peak = abs;
        }
    }

    fn take_rms(&mut self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let rms = (self.sum_sq / self.count as f32).sqrt();
        self.sum_sq = 0.0;
        self.count = 0;
        rms
    }

    fn take_peak(&mut self) -> f32 {
        let p = self.peak;
        self.peak *= 0.995;
        p
    }
}

pub struct AudioEngine {
    pub buffers: Arc<Mutex<SharedBuffers>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            buffers: Arc::new(Mutex::new(SharedBuffers::new())),
        }
    }

    pub fn start(
        &self,
        cmd_rx: Receiver<AudioCmd>,
        msg_tx: Sender<AudioMsg>,
    ) -> Result<(cpal::Stream, Option<cpal::Stream>), Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        let output_device = host
            .default_output_device()
            .ok_or("No output device available")?;

        let output_config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffers_out = Arc::clone(&self.buffers);

        // Shared input ring buffer for mic recording
        let input_ring: Arc<Mutex<Vec<f32>>> =
            Arc::new(Mutex::new(Vec::with_capacity(SAMPLE_RATE as usize)));
        let input_ring_for_output = Arc::clone(&input_ring);

        // --- All audio state lives inside the output callback closure ---
        let mut transport = Transport::new();
        let mut mixer = MixerState::new();
        let mut track_meters: Vec<LevelMeter> =
            (0..TRACK_COUNT).map(|_| LevelMeter::new()).collect();
        let mut master_meter_l = LevelMeter::new();
        let mut master_meter_r = LevelMeter::new();
        let mut report_counter: usize = 0;
        let report_interval = SAMPLE_RATE as usize / 30;

        // Synth engine
        let mut synth_engine: Box<dyn SynthEngine> = engines::create_engine(0);

        // Per-track effect chains
        let mut effect_chains: [Vec<Box<dyn effects::Effect>>; TRACK_COUNT] = [
            vec![],
            vec![],
            vec![],
            vec![],
        ];

        // Drum sequencer
        let mut drum_kit = DrumKit::new();
        let mut seq_clock = SequencerClock::new(120.0);
        let mut drum_patterns = [[false; 16]; 6];
        // Free-running sample counter for drum preview when transport is stopped
        let mut free_counter: usize = 0;

        // Tape simulation
        let mut tape_sim = TapeSimulation::new();

        // Recording source
        let mut record_source = RecordSource::Internal;

        // Record count-in + metronome
        let count_in_beats: usize = 4;
        let click_len_samples: usize = (SAMPLE_RATE as usize / 40).max(1); // ~25ms click
        let mut pending_record_track: Option<usize> = None;
        let mut count_in_samples_remaining: usize = 0;
        let mut count_in_samples_to_next_click: usize = 0;
        let mut count_in_click_index: usize = 0;
        let mut click_samples_remaining: usize = 0;
        let mut click_phase: f64 = 0.0;
        let mut click_freq: f64 = 1600.0;
        let mut click_amp: f32 = 0.0;

        let msg_tx_out = msg_tx.clone();

        let output_stream = output_device.build_output_stream(
            &output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // --- Process commands ---
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        AudioCmd::Play => {
                            if transport.loop_enabled() {
                                if let Ok(bufs) = buffers_out.try_lock() {
                                    transport.set_loop_end(detect_loop_end(&bufs, None));
                                }
                            }
                            transport.play()
                        }
                        AudioCmd::SetLoopEnabled(enabled) => {
                            transport.set_loop_enabled(enabled);
                            if enabled {
                                if let Ok(bufs) = buffers_out.try_lock() {
                                    transport.set_loop_end(detect_loop_end(&bufs, None));
                                }
                            }
                        }
                        AudioCmd::Pause => transport.pause(),
                        AudioCmd::Stop => {
                            pending_record_track = None;
                            count_in_samples_remaining = 0;
                            count_in_samples_to_next_click = 0;
                            click_samples_remaining = 0;
                            transport.stop();
                            seq_clock.reset();
                            let _ = msg_tx_out.try_send(AudioMsg::CurrentStep(0));
                        }
                        AudioCmd::Record(track) => {
                            pending_record_track = Some(track);
                            count_in_samples_remaining = samples_per_beat(seq_clock.bpm()) * count_in_beats;
                            count_in_samples_to_next_click = 0; // first click immediately
                            count_in_click_index = 0;
                            click_samples_remaining = 0;

                            if transport.loop_enabled() {
                                if let Ok(bufs) = buffers_out.try_lock() {
                                    transport.set_loop_end(detect_loop_end(&bufs, Some(track)));
                                }
                            }
                            // Count-in always runs against playback for timing.
                            transport.play();
                        }
                        AudioCmd::StopRecord => {
                            pending_record_track = None;
                            count_in_samples_remaining = 0;
                            count_in_samples_to_next_click = 0;
                            click_samples_remaining = 0;
                            if transport.loop_enabled() && transport.loop_end().is_none() {
                                transport.set_loop_end(Some(transport.position.min(TRACK_SAMPLES)));
                            }
                            transport.stop_record()
                        }
                        AudioCmd::Seek(pos) => transport.seek(pos),
                        AudioCmd::SetLevel(track, val) => {
                            if track < TRACK_COUNT {
                                mixer.levels[track] = val;
                            }
                        }
                        AudioCmd::SetPan(track, val) => {
                            if track < TRACK_COUNT {
                                mixer.pans[track] = val;
                            }
                        }
                        AudioCmd::SetMute(track, val) => {
                            if track < TRACK_COUNT {
                                mixer.mutes[track] = val;
                            }
                        }
                        AudioCmd::SetSolo(track, val) => {
                            if track < TRACK_COUNT {
                                mixer.solos[track] = val;
                            }
                        }
                        AudioCmd::NoteOn(note, vel) => {
                            synth_engine.note_on(note, vel);
                        }
                        AudioCmd::NoteOff(note) => {
                            synth_engine.note_off(note);
                        }
                        AudioCmd::SelectEngine(idx) => {
                            synth_engine = engines::create_engine(idx);
                        }
                        AudioCmd::SetParam(idx, val) => {
                            synth_engine.set_param(idx, val);
                        }
                        AudioCmd::ToggleStep(inst, step) => {
                            if inst < 6 && step < 16 {
                                drum_patterns[inst][step] = !drum_patterns[inst][step];
                            }
                        }
                        AudioCmd::SetBpm(bpm) => {
                            seq_clock.set_bpm(bpm);
                        }
                        AudioCmd::ToggleTapeSim => {
                            tape_sim.enabled = !tape_sim.enabled;
                        }
                        AudioCmd::SetTapeSpeed(_speed) => {
                            // Variable speed playback (future enhancement)
                        }
                        AudioCmd::ToggleEffect(track, slot) => {
                            if track < TRACK_COUNT && slot < effect_chains[track].len() {
                                let bypassed = effect_chains[track][slot].is_bypassed();
                                effect_chains[track][slot].set_bypass(!bypassed);
                            }
                        }
                        AudioCmd::SetEffectParam(track, slot, param, val) => {
                            if track < TRACK_COUNT && slot < effect_chains[track].len() {
                                effect_chains[track][slot].set_param(param, val);
                            }
                        }
                        AudioCmd::SetRecordSource(src) => {
                            record_source = src;
                        }
                    }
                }

                // --- Drain mic input ring buffer ---
                let mut mic_samples: Vec<f32> = Vec::new();
                if let Ok(mut ring) = input_ring_for_output.try_lock() {
                    if !ring.is_empty() {
                        mic_samples = std::mem::take(&mut *ring);
                    }
                }
                let mut mic_read_pos = 0;

                // Never block/panic in the realtime callback. If buffers are contended,
                // we skip tape read/write for this callback chunk and keep live monitoring running.
                let mut bufs_guard = buffers_out.try_lock().ok();

                // --- Generate output frame by frame ---
                for frame in data.chunks_mut(2) {
                    let playing = transport.is_playing() && transport.position < TRACK_SAMPLES;

                    // --- Synth output (always generates, even when not recording) ---
                    let mut synth_buf = [0.0f32; 1];
                    synth_engine.process(&mut synth_buf);
                    let synth_sample = synth_buf[0];

                    // --- Drum sequencer ---
                    // Runs against tape position when playing, free-running when stopped
                    let seq_pos = if playing {
                        transport.position
                    } else {
                        free_counter
                    };
                    free_counter = free_counter.wrapping_add(1);

                    let (step, new_step) = seq_clock.tick(seq_pos);
                    if new_step {
                        let _ = msg_tx_out.try_send(AudioMsg::CurrentStep(step));
                        for inst in 0..6 {
                            if drum_patterns[inst][step] {
                                drum_kit.trigger(inst);
                            }
                        }
                    }
                    let drum_sample = drum_kit.process();

                    // --- Record count-in ---
                    if let Some(track) = pending_record_track {
                        if count_in_samples_remaining == 0 {
                            // Start recording at loop start for tighter overdubs.
                            transport.seek(0);
                            seq_clock.reset();
                            let _ = msg_tx_out.try_send(AudioMsg::CurrentStep(0));
                            transport.record(track);
                            pending_record_track = None;
                        } else {
                            if count_in_samples_to_next_click == 0 {
                                let accented = count_in_click_index % count_in_beats == 0;
                                click_freq = if accented { 1900.0 } else { 1500.0 };
                                click_amp = if accented { 0.32 } else { 0.22 };
                                click_phase = 0.0;
                                click_samples_remaining = click_len_samples;
                                count_in_click_index = count_in_click_index.wrapping_add(1);
                                count_in_samples_to_next_click = samples_per_beat(seq_clock.bpm());
                            }
                            count_in_samples_to_next_click =
                                count_in_samples_to_next_click.saturating_sub(1);
                            count_in_samples_remaining = count_in_samples_remaining.saturating_sub(1);
                        }
                    }

                    let mut metronome_sample = 0.0f32;
                    if click_samples_remaining > 0 {
                        let env = click_samples_remaining as f32 / click_len_samples as f32;
                        metronome_sample = (click_phase * std::f64::consts::TAU).sin() as f32 * click_amp * env;
                        click_phase += click_freq / SAMPLE_RATE as f64;
                        if click_phase >= 1.0 {
                            click_phase -= 1.0;
                        }
                        click_samples_remaining = click_samples_remaining.saturating_sub(1);
                    }

                    // --- Recording: write selected source to armed track ---
                    if let Some(rec_track) = transport.recording_track {
                        if transport.position < TRACK_SAMPLES {
                            let mut rec_sample = 0.0f32;

                            // Mic input
                            if matches!(record_source, RecordSource::Mic | RecordSource::All) {
                                if mic_read_pos < mic_samples.len() {
                                    rec_sample += mic_samples[mic_read_pos];
                                    mic_read_pos += 1;
                                }
                            }

                            // Synth output
                            if matches!(
                                record_source,
                                RecordSource::Synth | RecordSource::All | RecordSource::Internal
                            ) {
                                rec_sample += synth_sample;
                            }

                            // Drum output
                            if matches!(
                                record_source,
                                RecordSource::Drum | RecordSource::All | RecordSource::Internal
                            ) {
                                rec_sample += drum_sample;
                            }

                            if let Some(bufs) = bufs_guard.as_mut() {
                                bufs.tracks[rec_track].data[transport.position] = rec_sample;
                                let current_len = bufs.tracks[rec_track]
                                    .len
                                    .load(std::sync::atomic::Ordering::Relaxed);
                                if transport.position >= current_len {
                                    bufs.tracks[rec_track]
                                        .len
                                        .store(transport.position + 1, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        }
                    }

                    // --- Read track data + apply per-track effects ---
                    if playing {
                        let mut track_samples = [0.0f32; TRACK_COUNT];
                        for t in 0..TRACK_COUNT {
                            let mut sample = if let Some(bufs) = bufs_guard.as_ref() {
                                bufs.tracks[t].read(transport.position)
                            } else {
                                0.0
                            };

                            // Apply per-track effects
                            if !effect_chains[t].is_empty() {
                                let mut buf = [sample];
                                for fx in &mut effect_chains[t] {
                                    if !fx.is_bypassed() {
                                        fx.process(&mut buf);
                                    }
                                }
                                sample = buf[0];
                            }

                            track_samples[t] = sample;
                            track_meters[t].push(sample);
                        }

                        let (mut left, mut right) = mixer.mix(&track_samples);

                        // Avoid doubling/echo: when a source is actively being recorded,
                        // don't also add a parallel live monitor path for that same source.
                        let recording = transport.recording_track.is_some();
                        let monitor_synth = !recording
                            || !matches!(
                                record_source,
                                RecordSource::Synth | RecordSource::All | RecordSource::Internal
                            );
                        let monitor_drum = !recording
                            || !matches!(
                                record_source,
                                RecordSource::Drum | RecordSource::All | RecordSource::Internal
                            );

                        if monitor_synth {
                            left += synth_sample * 0.5;
                            right += synth_sample * 0.5;
                        }

                        if monitor_drum {
                            left += drum_sample * 0.5;
                            right += drum_sample * 0.5;
                        }

                        // Count-in click monitor
                        left += metronome_sample;
                        right += metronome_sample;

                        // Tape simulation
                        tape_sim.process_stereo(&mut left, &mut right);

                        frame[0] = left.clamp(-1.0, 1.0);
                        frame[1] = right.clamp(-1.0, 1.0);

                        master_meter_l.push(left);
                        master_meter_r.push(right);

                        transport.advance();
                    } else {
                        // When stopped, still output synth + drums for live preview
                        let left = (synth_sample + drum_sample) * 0.5 + metronome_sample;
                        let right = (synth_sample + drum_sample) * 0.5 + metronome_sample;
                        frame[0] = left.clamp(-1.0, 1.0);
                        frame[1] = right.clamp(-1.0, 1.0);
                        master_meter_l.push(left);
                        master_meter_r.push(right);
                    }

                    report_counter += 1;
                    if report_counter >= report_interval {
                        report_counter = 0;
                        let _ = msg_tx_out.try_send(AudioMsg::Position(transport.position));
                        let levels = [
                            track_meters[0].take_rms(),
                            track_meters[1].take_rms(),
                            track_meters[2].take_rms(),
                            track_meters[3].take_rms(),
                        ];
                        let peaks = [
                            track_meters[0].take_peak(),
                            track_meters[1].take_peak(),
                            track_meters[2].take_peak(),
                            track_meters[3].take_peak(),
                        ];
                        let _ = msg_tx_out.try_send(AudioMsg::Levels(levels));
                        let _ = msg_tx_out.try_send(AudioMsg::Peaks(peaks));
                        let _ = msg_tx_out.try_send(AudioMsg::MasterLevel(
                            master_meter_l.take_rms(),
                            master_meter_r.take_rms(),
                        ));
                    }
                }
            },
            |err| {
                eprintln!("Audio output error: {}", err);
            },
            None,
        )?;

        output_stream.play()?;

        // --- Input stream (mic) ---
        let input_ring_writer = Arc::clone(&input_ring);
        let input_stream = if let Some(input_device) = host.default_input_device() {
            let input_config = StreamConfig {
                channels: 1,
                sample_rate: SampleRate(SAMPLE_RATE),
                buffer_size: cpal::BufferSize::Default,
            };

            let stream = input_device.build_input_stream(
                &input_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut ring) = input_ring_writer.try_lock() {
                        ring.extend_from_slice(data);
                        if ring.len() > SAMPLE_RATE as usize {
                            let drain = ring.len() - SAMPLE_RATE as usize;
                            ring.drain(..drain);
                        }
                    }
                },
                |err| {
                    eprintln!("Audio input error: {}", err);
                },
                None,
            );

            match stream {
                Ok(s) => {
                    s.play()?;
                    Some(s)
                }
                Err(e) => {
                    eprintln!("Warning: Could not open mic input: {}", e);
                    None
                }
            }
        } else {
            eprintln!("Warning: No input device found. Recording from mic disabled.");
            None
        };

        Ok((output_stream, input_stream))
    }
}
