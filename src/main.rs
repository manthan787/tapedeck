#![allow(dead_code, unused_imports)]

mod app;
mod audio;
mod constants;
mod effects;
mod input;
mod messages;
mod project;
mod sequencer;
mod synth;
mod tape;
mod ui;

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{AppMode, AppState};
use crate::audio::buffer::{downsample_track, SharedBuffers};
use crate::audio::engine::AudioEngine;
use crate::constants::*;
use crate::messages::*;
use crate::ui::views::drum_view::DrumView;
use crate::ui::views::mixer_view::MixerView;
use crate::ui::views::synth_view::SynthView;
use crate::ui::views::tape_view::TapeView;
use crate::ui::views::View;
use crate::ui::widgets::keyboard_hint::KeyboardHintWidget;
use crate::ui::widgets::mode_indicator::ModeIndicatorWidget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup channels ---
    let (audio_cmd_tx, audio_cmd_rx): (Sender<AudioCmd>, Receiver<AudioCmd>) =
        bounded(CHANNEL_CAPACITY);
    let (audio_msg_tx, audio_msg_rx): (Sender<AudioMsg>, Receiver<AudioMsg>) =
        bounded(CHANNEL_CAPACITY);

    // --- Audio engine setup ---
    let engine = AudioEngine::new();
    let buffers = Arc::clone(&engine.buffers);

    let (_output_stream, _input_stream) = match engine.start(audio_cmd_rx, audio_msg_tx) {
        Ok(streams) => streams,
        Err(e) => {
            eprintln!("Warning: Audio engine failed to start: {}", e);
            eprintln!("Running in UI-only mode (no audio).");
            return run_ui_loop(audio_cmd_tx, audio_msg_rx, buffers);
        }
    };

    // Keep streams alive and run UI
    run_ui_loop(audio_cmd_tx, audio_msg_rx, buffers)
}

fn run_ui_loop(
    audio_cmd_tx: Sender<AudioCmd>,
    audio_msg_rx: Receiver<AudioMsg>,
    buffers: Arc<Mutex<SharedBuffers>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // --- Terminal setup ---
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // --- App state ---
    let mut state = AppState::new();
    let mut tape_view = TapeView::new();

    let frame_duration = Duration::from_millis(1000 / UI_FPS);

    // --- Main loop ---
    loop {
        let frame_start = Instant::now();

        // --- Process audio messages (non-blocking) ---
        while let Ok(msg) = audio_msg_rx.try_recv() {
            match msg {
                AudioMsg::Position(pos) => state.position = pos,
                AudioMsg::Levels(levels) => state.levels = levels,
                AudioMsg::Peaks(peaks) => state.peaks = peaks,
                AudioMsg::MasterLevel(l, r) => state.master_level = (l, r),
            }
        }

        // --- Process keyboard input ---
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(evt) = input::handle_key(key, state.mode, state.selected_track) {
                        handle_ui_event(&mut state, evt, &audio_cmd_tx, &buffers);
                    }
                }
                // Handle note-off for synth keys on release
                if key.kind == KeyEventKind::Release && state.mode == AppMode::Synth {
                    let note = match key.code {
                        crossterm::event::KeyCode::Char('z') => Some(48u8),
                        crossterm::event::KeyCode::Char('s') => Some(49),
                        crossterm::event::KeyCode::Char('x') => Some(50),
                        crossterm::event::KeyCode::Char('d') => Some(51),
                        crossterm::event::KeyCode::Char('c') => Some(52),
                        crossterm::event::KeyCode::Char('v') => Some(53),
                        crossterm::event::KeyCode::Char('g') => Some(54),
                        crossterm::event::KeyCode::Char('b') => Some(55),
                        crossterm::event::KeyCode::Char('h') => Some(56),
                        crossterm::event::KeyCode::Char('n') => Some(57),
                        crossterm::event::KeyCode::Char('j') => Some(58),
                        crossterm::event::KeyCode::Char('m') => Some(59),
                        crossterm::event::KeyCode::Char('q') => Some(60),
                        crossterm::event::KeyCode::Char('2') => Some(61),
                        crossterm::event::KeyCode::Char('w') => Some(62),
                        crossterm::event::KeyCode::Char('3') => Some(63),
                        crossterm::event::KeyCode::Char('e') => Some(64),
                        crossterm::event::KeyCode::Char('4') => Some(65),
                        crossterm::event::KeyCode::Char('5') => Some(66),
                        crossterm::event::KeyCode::Char('t') => Some(67),
                        crossterm::event::KeyCode::Char('6') => Some(68),
                        crossterm::event::KeyCode::Char('y') => Some(69),
                        crossterm::event::KeyCode::Char('7') => Some(70),
                        crossterm::event::KeyCode::Char('u') => Some(71),
                        _ => None,
                    };
                    if let Some(n) = note {
                        let _ = audio_cmd_tx.try_send(AudioCmd::NoteOff(n));
                    }
                }
            }
        }

        if state.should_quit {
            break;
        }

        // --- Update waveform data periodically ---
        if tape_view.frame_count % 30 == 0 {
            if let Ok(bufs) = buffers.try_lock() {
                for i in 0..TRACK_COUNT {
                    state.waveform_data[i] = downsample_track(&bufs.tracks[i], 200);
                }
            }
        }

        // --- Render ---
        tape_view.frame_count += 1;
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = ui::layout::ScreenLayout::new(area);

            // Header: mode indicator
            frame.render_widget(
                ModeIndicatorWidget {
                    current: state.mode,
                },
                layout.header,
            );

            // Main content based on mode
            match state.mode {
                AppMode::Tape => tape_view.render(&state, frame, layout.main),
                AppMode::Synth => SynthView.render(&state, frame, layout.main),
                AppMode::Drum => DrumView.render(&state, frame, layout.main),
                AppMode::Mixer => MixerView.render(&state, frame, layout.main),
            }

            // Footer: keyboard hints
            let hints = input::key_hints(state.mode);
            frame.render_widget(KeyboardHintWidget { hints }, layout.footer);
        })?;

        // --- Frame rate limiting ---
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // --- Cleanup ---
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_ui_event(
    state: &mut AppState,
    event: UiEvent,
    audio_cmd_tx: &Sender<AudioCmd>,
    buffers: &Arc<Mutex<SharedBuffers>>,
) {
    match event {
        UiEvent::Quit => {
            state.should_quit = true;
        }
        UiEvent::TogglePlayPause => match state.transport {
            TransportDisplay::Stopped | TransportDisplay::Paused => {
                state.transport = TransportDisplay::Playing;
                let _ = audio_cmd_tx.try_send(AudioCmd::Play);
            }
            TransportDisplay::Playing => {
                state.transport = TransportDisplay::Paused;
                let _ = audio_cmd_tx.try_send(AudioCmd::Pause);
            }
            TransportDisplay::Recording => {
                state.transport = TransportDisplay::Playing;
                let _ = audio_cmd_tx.try_send(AudioCmd::StopRecord);
            }
        },
        UiEvent::StartRecord => {
            if let Some(track) = state.track_displays.iter().position(|t| t.armed) {
                state.transport = TransportDisplay::Recording;
                let _ = audio_cmd_tx.try_send(AudioCmd::Record(track));
            }
        }
        UiEvent::StopTransport => {
            state.transport = TransportDisplay::Stopped;
            state.position = 0;
            let _ = audio_cmd_tx.try_send(AudioCmd::Stop);
        }
        UiEvent::SelectTrack(track) => {
            if track < TRACK_COUNT {
                state.selected_track = track;
            }
        }
        UiEvent::ArmTrack(track) => {
            if track < TRACK_COUNT {
                let currently_armed = state.track_displays[track].armed;
                for t in &mut state.track_displays {
                    t.armed = false;
                }
                if !currently_armed {
                    state.track_displays[track].armed = true;
                }
            }
        }
        UiEvent::MuteTrack(track) => {
            if track < TRACK_COUNT {
                state.track_displays[track].muted = !state.track_displays[track].muted;
                let _ = audio_cmd_tx.try_send(AudioCmd::SetMute(
                    track,
                    state.track_displays[track].muted,
                ));
            }
        }
        UiEvent::SoloTrack(track) => {
            if track < TRACK_COUNT {
                state.track_displays[track].solo = !state.track_displays[track].solo;
                let _ = audio_cmd_tx.try_send(AudioCmd::SetSolo(
                    track,
                    state.track_displays[track].solo,
                ));
            }
        }
        UiEvent::Seek(delta) => {
            let new_pos = (state.position as i64 + delta).max(0) as usize;
            state.position = new_pos;
            let _ = audio_cmd_tx.try_send(AudioCmd::Seek(new_pos));
        }
        UiEvent::CycleMode => {
            state.mode = state.mode.next();
        }
        UiEvent::SetLevel(track, delta) => {
            if track < TRACK_COUNT {
                let new_level = (state.track_displays[track].level + delta).clamp(0.0, 1.0);
                state.track_displays[track].level = new_level;
                let _ = audio_cmd_tx.try_send(AudioCmd::SetLevel(track, new_level));
            }
        }
        UiEvent::SetPan(track, delta) => {
            if track < TRACK_COUNT {
                let new_pan = (state.track_displays[track].pan + delta).clamp(-1.0, 1.0);
                state.track_displays[track].pan = new_pan;
                let _ = audio_cmd_tx.try_send(AudioCmd::SetPan(track, new_pan));
            }
        }
        UiEvent::NoteOn(note, vel) => {
            let _ = audio_cmd_tx.try_send(AudioCmd::NoteOn(note, vel));
        }
        UiEvent::NoteOff(note) => {
            let _ = audio_cmd_tx.try_send(AudioCmd::NoteOff(note));
        }
        UiEvent::SelectEngine(dir) => {
            let count = synth::engines::ENGINE_COUNT;
            if dir == 0 {
                state.synth_engine = (state.synth_engine + count - 1) % count;
            } else {
                state.synth_engine = (state.synth_engine + 1) % count;
            }
            let _ = audio_cmd_tx.try_send(AudioCmd::SelectEngine(state.synth_engine));
        }
        UiEvent::SetParam(index, delta) => {
            if index < 4 {
                state.synth_params[index] = (state.synth_params[index] + delta).clamp(0.0, 1.0);
                let _ =
                    audio_cmd_tx.try_send(AudioCmd::SetParam(index, state.synth_params[index]));
            }
        }
        UiEvent::ToggleStep(instrument, step) => {
            let inst = if state.mode == AppMode::Drum {
                state.selected_instrument
            } else {
                instrument
            };
            if inst < 6 && step < 16 {
                state.drum_patterns[inst][step] = !state.drum_patterns[inst][step];
                let _ = audio_cmd_tx.try_send(AudioCmd::ToggleStep(inst, step));
            }
        }
        UiEvent::SetBpm(delta) => {
            state.bpm = (state.bpm + delta).clamp(40.0, 300.0);
            let _ = audio_cmd_tx.try_send(AudioCmd::SetBpm(state.bpm));
        }
        UiEvent::SelectInstrument(inst) => {
            if inst < 6 {
                state.selected_instrument = inst;
            }
        }
        UiEvent::ToggleTapeSim => {
            state.tape_sim_enabled = !state.tape_sim_enabled;
            let _ = audio_cmd_tx.try_send(AudioCmd::ToggleTapeSim);
        }
        UiEvent::SetTapeSpeed(speed) => {
            state.tape_speed = speed;
            let _ = audio_cmd_tx.try_send(AudioCmd::SetTapeSpeed(speed));
        }
        UiEvent::ToggleEffect(track, slot) => {
            if track < TRACK_COUNT && slot < 3 {
                state.effect_bypassed[track][slot] = !state.effect_bypassed[track][slot];
                let _ = audio_cmd_tx.try_send(AudioCmd::ToggleEffect(track, slot));
            }
        }
        UiEvent::SetEffectParam(track, slot, param, value) => {
            let _ = audio_cmd_tx.try_send(AudioCmd::SetEffectParam(track, slot, param, value));
        }
        UiEvent::SaveProject => {
            if let Ok(bufs) = buffers.lock() {
                let meta = project::metadata::ProjectMeta::new("tapedeck_project");
                let dir = std::path::Path::new("tapedeck_project");
                if let Err(e) = project::save::save_project(dir, &meta, &bufs) {
                    eprintln!("Save error: {}", e);
                }
            }
        }
        UiEvent::LoadProject(path) => {
            if let Ok(mut bufs) = buffers.lock() {
                let dir = std::path::Path::new(&path);
                match project::load::load_project(dir, &mut bufs) {
                    Ok(_meta) => {}
                    Err(e) => eprintln!("Load error: {}", e),
                }
            }
        }
    }
}
