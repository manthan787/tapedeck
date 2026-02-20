use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppMode;
use crate::messages::UiEvent;

/// Map keyboard input to UiEvent based on current mode
pub fn handle_key(key: KeyEvent, mode: AppMode, selected_track: usize) -> Option<UiEvent> {
    // Global keys (all modes)
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => return Some(UiEvent::Quit),
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(UiEvent::SaveProject);
        }
        KeyCode::Char(' ') => return Some(UiEvent::TogglePlayPause),
        KeyCode::Tab => return Some(UiEvent::CycleMode),
        _ => {}
    }

    // Track selection (1-4) in non-synth modes
    match key.code {
        KeyCode::Char(c @ '1'..='4') if mode != AppMode::Synth => {
            let track = (c as usize) - ('1' as usize);
            return Some(UiEvent::SelectTrack(track));
        }
        _ => {}
    }

    // Mode-specific keys
    match mode {
        AppMode::Tape => handle_tape_key(key, selected_track),
        AppMode::Synth => handle_synth_key(key),
        AppMode::Drum => handle_drum_key(key),
        AppMode::Mixer => handle_mixer_key(key, selected_track),
    }
}

fn handle_tape_key(key: KeyEvent, selected_track: usize) -> Option<UiEvent> {
    match key.code {
        KeyCode::Char('r') => Some(UiEvent::StartRecord),
        KeyCode::Char('m') => Some(UiEvent::MuteTrack(selected_track)),
        KeyCode::Char('s') => Some(UiEvent::SoloTrack(selected_track)),
        KeyCode::Left => Some(UiEvent::Seek(-44100)),
        KeyCode::Right => Some(UiEvent::Seek(44100)),
        KeyCode::Char('[') => Some(UiEvent::Seek(-44100 * 5)),
        KeyCode::Char(']') => Some(UiEvent::Seek(44100 * 5)),
        KeyCode::Char('a') => Some(UiEvent::ArmTrack(selected_track)),
        _ => None,
    }
}

fn handle_synth_key(key: KeyEvent) -> Option<UiEvent> {
    // QWERTY piano mapping
    // Bottom row: Z=C3, S=C#3, X=D3, D=D#3, C=E3, V=F3, G=F#3, B=G3, H=G#3, N=A3, J=A#3, M=B3
    // Top row: Q=C4, 2=C#4, W=D4, 3=D#4, E=E4, R=F4, 5=F#4, T=G4, 6=G#4, Y=A4, 7=A#4, U=B4
    let note = match key.code {
        // C3 = MIDI 48
        KeyCode::Char('z') => Some(48u8),
        KeyCode::Char('s') => Some(49),
        KeyCode::Char('x') => Some(50),
        KeyCode::Char('d') => Some(51),
        KeyCode::Char('c') => Some(52),
        KeyCode::Char('v') => Some(53),
        KeyCode::Char('g') => Some(54),
        KeyCode::Char('b') => Some(55),
        KeyCode::Char('h') => Some(56),
        KeyCode::Char('n') => Some(57),
        KeyCode::Char('j') => Some(58),
        KeyCode::Char('m') => Some(59),
        // C4 = MIDI 60
        KeyCode::Char('q') => Some(60),
        KeyCode::Char('2') => Some(61),
        KeyCode::Char('w') => Some(62),
        KeyCode::Char('3') => Some(63),
        KeyCode::Char('e') => Some(64),
        KeyCode::Char('r') => Some(65),
        KeyCode::Char('5') => Some(66),
        KeyCode::Char('t') => Some(67),
        KeyCode::Char('6') => Some(68),
        KeyCode::Char('y') => Some(69),
        KeyCode::Char('7') => Some(70),
        KeyCode::Char('u') => Some(71),
        _ => None,
    };

    if let Some(n) = note {
        return Some(UiEvent::NoteOn(n, 0.8));
    }

    match key.code {
        KeyCode::Left => Some(UiEvent::SelectEngine(0)),  // prev
        KeyCode::Right => Some(UiEvent::SelectEngine(1)), // next (handled as delta in control)
        KeyCode::Up => Some(UiEvent::SetParam(0, 0.05)),  // increment
        KeyCode::Down => Some(UiEvent::SetParam(0, -0.05)), // decrement
        _ => None,
    }
}

fn handle_drum_key(key: KeyEvent) -> Option<UiEvent> {
    // Z-M row toggles steps 0-11, and comma/period/slash for 12-14, plus extra
    let step = match key.code {
        KeyCode::Char('z') => Some(0usize),
        KeyCode::Char('x') => Some(1),
        KeyCode::Char('c') => Some(2),
        KeyCode::Char('v') => Some(3),
        KeyCode::Char('b') => Some(4),
        KeyCode::Char('n') => Some(5),
        KeyCode::Char('m') => Some(6),
        KeyCode::Char(',') => Some(7),
        KeyCode::Char('a') => Some(8),
        KeyCode::Char('s') => Some(9),
        KeyCode::Char('d') => Some(10),
        KeyCode::Char('f') => Some(11),
        KeyCode::Char('g') => Some(12),
        KeyCode::Char('h') => Some(13),
        KeyCode::Char('j') => Some(14),
        KeyCode::Char('k') => Some(15),
        _ => None,
    };

    if let Some(s) = step {
        return Some(UiEvent::ToggleStep(0, s)); // instrument selected separately
    }

    match key.code {
        KeyCode::Char('1'..='4') => {
            let inst = (key.code.to_string().parse::<usize>().unwrap_or(1)) - 1;
            Some(UiEvent::SelectInstrument(inst))
        }
        KeyCode::Up => Some(UiEvent::SetBpm(1.0)),
        KeyCode::Down => Some(UiEvent::SetBpm(-1.0)),
        KeyCode::Char('r') => Some(UiEvent::StartRecord),
        _ => None,
    }
}

fn handle_mixer_key(key: KeyEvent, selected_track: usize) -> Option<UiEvent> {
    match key.code {
        KeyCode::Up => Some(UiEvent::SetLevel(selected_track, 0.05)),
        KeyCode::Down => Some(UiEvent::SetLevel(selected_track, -0.05)),
        KeyCode::Left => Some(UiEvent::SetPan(selected_track, -0.1)),
        KeyCode::Right => Some(UiEvent::SetPan(selected_track, 0.1)),
        KeyCode::Char('m') => Some(UiEvent::MuteTrack(selected_track)),
        KeyCode::Char('s') => Some(UiEvent::SoloTrack(selected_track)),
        _ => None,
    }
}

/// Key labels for the hint bar
pub fn key_hints(mode: AppMode) -> Vec<(&'static str, &'static str)> {
    let mut hints = vec![
        ("Space", "Play/Pause"),
        ("Tab", "Mode"),
        ("Q", "Quit"),
    ];

    match mode {
        AppMode::Tape => {
            hints.insert(0, ("R", "Record"));
            hints.insert(1, ("A", "Arm"));
            hints.insert(2, ("1-4", "Track"));
            hints.insert(3, ("M", "Mute"));
            hints.insert(4, ("S", "Solo"));
            hints.insert(5, ("[/]", "Seek"));
        }
        AppMode::Synth => {
            hints.insert(0, ("Z-M", "Play"));
            hints.insert(1, ("R", "Record"));
            hints.insert(2, ("←/→", "Engine"));
            hints.insert(3, ("↑/↓", "Param"));
        }
        AppMode::Drum => {
            hints.insert(0, ("Z-K", "Steps"));
            hints.insert(1, ("1-4", "Inst"));
            hints.insert(2, ("↑/↓", "BPM"));
            hints.insert(3, ("R", "Record"));
        }
        AppMode::Mixer => {
            hints.insert(0, ("1-4", "Track"));
            hints.insert(1, ("↑/↓", "Level"));
            hints.insert(2, ("←/→", "Pan"));
            hints.insert(3, ("M", "Mute"));
            hints.insert(4, ("S", "Solo"));
        }
    }

    hints
}
