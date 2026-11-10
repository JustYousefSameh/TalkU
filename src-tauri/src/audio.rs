use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct AudioPlayer {
    _stream: MixerDeviceSink,
    active_player: Mutex<Option<Player>>,
}

impl AudioPlayer {
    pub fn init() -> Result<Self, String> {
        let stream = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Failed to initialize audio output sink: {e}"))?;

        Ok(Self {
            _stream: stream,
            active_player: Mutex::new(None),
        })
    }
}

/// Tauri command to play a bundled audio resource by name (e.g. "connected" or "disconnected")
#[tauri::command]
pub fn play_audio(
    sound_name: String,
    app: AppHandle,
    player_state: State<'_, AudioPlayer>,
) -> Result<(), String> {
    // Resolve the path to the bundled resource dynamically
    let resource_path = app
        .path()
        .resolve(
            format!("resources/{}.wav", sound_name),
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|e| format!("Failed to resolve resource path: {e}"))?;

    let file = File::open(&resource_path)
        .map_err(|e| format!("Failed to open audio file at {:?}: {e}", resource_path))?;

    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio file: {e}"))?;

    // Connect a new Player to the mixer obtained from the stream sink
    let player = Player::connect_new(player_state._stream.mixer());

    player.append(source);
    player.play();

    let mut active = player_state.active_player.lock().unwrap();
    *active = Some(player);

    Ok(())
}

/// Tauri command to stop current audio playback
#[tauri::command]
pub fn stop_audio(player_state: State<'_, AudioPlayer>) -> Result<(), String> {
    let mut active = player_state.active_player.lock().unwrap();
    if let Some(player) = active.take() {
        player.stop();
    }
    Ok(())
}
