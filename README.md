# 🎛️ Tapedeck

**A 4-track studio that fits inside your terminal.**

A terminal-based 4-track cassette recorder inspired by the Teenage Engineering OP-1. Record from your mic, layer synthesizers, program drum beats, and mix it all down — with animated spinning reels, VU meters, and that warm tape sound. Built in Rust.

## Features

- **4-Track Recording** — Record from mic input, overdub across 4 independent tracks with per-track arm/mute/solo
- **5 Synth Engines** — Sine, bandlimited Saw, 2-op FM, Karplus-Strong plucked string, and filtered Noise — playable via QWERTY keyboard with 8-voice polyphony
- **Drum Sequencer** — 16-step pattern sequencer with 6 synthesized instruments (kick, snare, hi-hat, clap, tom, rim), synced to tape position
- **5 Effects** — Reverb, ping-pong delay, resonant filter (LP/HP/BP), tape distortion, and chorus — per-track with bypass
- **Tape Simulation** — Wow, flutter, tape saturation, hiss, and high-frequency rolloff for authentic lo-fi warmth
- **Animated Cassette UI** — Braille-rendered spinning reels that grow/shrink as tape advances, color-coded transport states
- **Mixer View** — 4-channel faders with pan, level, VU meters, mute/solo
- **Project Save/Load** — Exports tracks as 32-bit float WAV files with JSON metadata

## Quick Start

```
cargo run
```

Requires Rust 1.70+ and a working audio output device. Mic input is optional (the app warns but doesn't crash without one).

## Controls

| Key | Action |
|-----|--------|
| `Tab` | Cycle modes: Tape → Synth → Drum → Mixer |
| `Space` | Play / Pause |
| `1`-`4` | Select track |
| `A` | Arm selected track for recording |
| `R` | Start recording to armed track |
| `M` | Mute track |
| `S` | Solo track |
| `[` / `]` | Rewind / fast-forward (5 sec) |
| `←` / `→` | Seek (1 sec) / navigate engines or steps |
| `↑` / `↓` | Adjust parameter, BPM, or level |
| `Ctrl+S` | Save project |
| `Q` | Quit |

**Synth mode**: `Z`-`M` plays C3–B3, `Q`-`U` plays C4–B4 (chromatic, black keys on the upper row).

**Drum mode**: `Z`-`K` toggles steps 1–16 for the selected instrument.

## Architecture

Three threads communicate via lock-free `crossbeam` channels:

```
UI Thread (60fps)        Control Thread          Audio Thread (44.1kHz)
┌────────────────┐      ┌────────────────┐      ┌──────────────────────┐
│ ratatui render  │─────>│ App state      │─────>│ cpal callback        │
│ crossterm input │<─────│ Command router │<─────│ Synth + Effects      │
└────────────────┘      └────────────────┘      │ Mixer + Tape sim     │
                                                 └──────────────────────┘
```

## Tech Stack

| Component | Crate |
|-----------|-------|
| Audio I/O | `cpal` |
| TUI | `ratatui` + `crossterm` |
| Concurrency | `crossbeam-channel` |
| WAV files | `hound` |
| Serialization | `serde` + `serde_json` |

## Project Structure

```
src/
  main.rs              Entry point, thread setup, event loop
  app.rs               App state and mode management
  audio/               cpal streams, track buffers, transport, mixer
  synth/engines/       Sine, Saw, FM, String, Noise synthesizers
  effects/             Reverb, Delay, Filter, Distortion, Chorus
  sequencer/           16-step drum sequencer with BPM clock
  tape/                Wow/flutter/saturation simulation
  ui/views/            Tape, Synth, Drum, Mixer screen layouts
  ui/widgets/          Cassette, VU meter, waveform, knobs, step grid
  project/             WAV + JSON save/load
```

## License

MIT
