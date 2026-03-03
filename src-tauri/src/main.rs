#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod tts;
pub mod audio;
pub mod config;
pub mod clipboard;
pub mod hotkey;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, Emitter, State,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::sync::Mutex;
use crate::config::settings::AppSettings;

// TTS 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TtsStatus {
    Idle,
    Playing,
    Paused,
    Error(String),
}

// 应用状态
pub struct AppState {
    pub status: Mutex<TtsStatus>,
    pub current_audio: Mutex<Option<Vec<u8>>>,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(TtsStatus::Idle),
            current_audio: Mutex::new(None),
            settings: Mutex::new(AppSettings::default()),
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, settings: AppSettings) {
        *self.settings.lock().unwrap() = settings;
    }
}

// TTS 请求
#[derive(Debug, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub speed: Option<f32>,
    pub volume: Option<f32>,
}

// TTS 响应
#[derive(Debug, Serialize)]
pub struct TtsResponse {
    pub success: bool,
    pub audio_data: Option<Vec<u8>>,
    pub error: Option<String>,
}

// 播放控制命令
#[derive(Debug, Deserialize)]
pub enum PlayCommand {
    Play,
    Pause,
    Resume,
    Stop,
}

#[tauri::command]
fn get_clipboard_content(app: tauri::AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_status(app: tauri::AppHandle, status: String) {
    let _ = app.emit("tts-status", status);
}

#[tauri::command]
async fn synthesize_tts(
    _app: tauri::AppHandle,
    request: TtsRequest,
    state: State<'_, AppState>,
) -> Result<TtsResponse, String> {
    // 调用 Python TTS 服务
    let result = tts::engine::synthesize(&request.text, request.speed, request.volume).await;

    match result {
        Ok(audio_data) => {
            *state.current_audio.lock().unwrap() = Some(audio_data.clone());

            Ok(TtsResponse {
                success: true,
                audio_data: Some(audio_data),
                error: None,
            })
        }
        Err(e) => Ok(TtsResponse {
            success: false,
            audio_data: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
fn play_audio(_app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let audio_data = state
        .current_audio
        .lock()
        .unwrap()
        .clone()
        .ok_or("No audio data")?;

    // 使用 AudioManager 播放（支持暂停/继续）
    audio::manager::play(audio_data)?;

    // 更新状态
    *state.status.lock().unwrap() = TtsStatus::Playing;

    Ok(())
}

#[tauri::command]
fn pause_audio(state: State<'_, AppState>) -> Result<(), String> {
    // 更新状态
    *state.status.lock().unwrap() = TtsStatus::Paused;
    // 使用 AudioManager 暂停
    audio::manager::pause()
}

#[tauri::command]
fn resume_audio(state: State<'_, AppState>) -> Result<(), String> {
    // 更新状态
    *state.status.lock().unwrap() = TtsStatus::Playing;
    // 使用 AudioManager 继续
    audio::manager::resume()
}

#[tauri::command]
fn stop_audio(state: State<'_, AppState>) -> Result<(), String> {
    // 更新状态
    *state.status.lock().unwrap() = TtsStatus::Idle;
    // 使用 AudioManager 停止
    audio::manager::stop()
}

#[tauri::command]
fn get_playback_status() -> Result<String, String> {
    let state = audio::manager::get_state();
    Ok(format!("{:?}", state))
}

#[tauri::command]
fn seek_audio(position: f32) -> Result<(), String> {
    audio::manager::seek(position)
}

#[tauri::command]
fn get_playback_progress() -> Result<crate::audio::manager::PlaybackProgress, String> {
    audio::manager::get_progress().ok_or("No playback progress available".to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.get_settings())
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    state.update_settings(settings.clone());
    settings.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn reset_settings(state: State<'_, AppState>) -> Result<(), String> {
    AppSettings::reset().map_err(|e| e.to_string())?;
    state.update_settings(AppSettings::default());
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_clipboard_content,
            set_status,
            synthesize_tts,
            play_audio,
            pause_audio,
            resume_audio,
            stop_audio,
            get_playback_status,
            seek_audio,
            get_playback_progress,
            get_settings,
            save_settings,
            reset_settings
        ])
        .setup(|app| {
            // 加载配置（在创建 AppState 之前）
            let settings = AppSettings::load().unwrap_or_else(|e| {
                eprintln!("加载配置失败：{}", e);
                AppSettings::default()
            });

            eprintln!("[Setup] 加载的快捷键：{}", settings.global_hotkey);

            // 使用加载的配置创建 AppState
            let app_state = AppState::new();
            app_state.update_settings(settings.clone());

            // 注册全局快捷键（使用配置中的快捷键）
            let shortcut: Shortcut = settings.global_hotkey.parse().unwrap();
            let app_handle = app.handle().clone();

            eprintln!("[Setup] 正在注册快捷键：{:?}", shortcut);

            match app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let app_handle = app_handle.clone();
                    std::thread::spawn(move || {
                        hotkey::handle_shortcut(&app_handle);
                    });
                }
            }) {
                Ok(_) => eprintln!("[Setup] 快捷键注册成功"),
                Err(e) => eprintln!("[Setup] 快捷键注册失败：{}", e),
            }

            // 创建托盘菜单
            let show = MenuItem::with_id(app, "show", "显示设置", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            eprintln!("[Setup] 设置完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
