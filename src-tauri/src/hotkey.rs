use tauri::{AppHandle, Manager};
use tauri::Emitter;
use crate::{clipboard, tts, audio, AppState};
use std::time::Instant;
use std::fs::OpenOptions;
use std::io::Write;

fn log_to_file(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/voicepicker.log")
    {
        let _ = writeln!(file, "[VoicePicker] {}", message);
    }
}

pub fn handle_shortcut(app: &AppHandle) {
    let total_start = Instant::now();
    log_to_file("快捷键被触发");
    eprintln!("[VoicePicker] 快捷键被触发");

    // 1. 获取选中的文本
    let clip_start = Instant::now();
    let text = match clipboard::get_selected_text(app) {
        t if !t.is_empty() => t,
        _ => {
            log_error(app, "未选中文本");
            return;
        }
    };
    log_to_file(&format!("获取文本耗时：{:?}，文本：{}", clip_start.elapsed(), text));
    eprintln!("[VoicePicker] 获取文本耗时：{:?}，文本：{}", clip_start.elapsed(), text);

    if text.trim().is_empty() {
        log_error(app, "选中的文本为空");
        return;
    }

    // 2. 获取应用状态（包含设置）
    let state = app.state::<AppState>();
    let settings = state.get_settings();

    log_to_file(&format!("设置：语速={:.1}, 音量={:.1}", settings.tts_speed, settings.tts_volume));
    eprintln!("[VoicePicker] 设置：语速={:.1}, 音量={:.1}", settings.tts_speed, settings.tts_volume);

    // 3. 设置状态为处理中
    let _ = app.emit("tts-status", "正在生成语音...");

    // 4. 调用 TTS 引擎（使用设置）
    let tts_start = Instant::now();
    let audio_data = match std::thread::spawn({
        let text = text.clone();
        let speed = settings.tts_speed;
        let volume = settings.tts_volume;
        move || {
            tts::engine::synthesize_blocking(&text, Some(speed), Some(volume))
        }
    })
    .join()
    {
        Ok(Ok(data)) => {
            log_to_file(&format!("TTS 生成耗时：{:?}，音频长度：{} 字节", tts_start.elapsed(), data.len()));
            eprintln!("[VoicePicker] TTS 生成耗时：{:?}，音频长度：{} 字节", tts_start.elapsed(), data.len());
            data
        },
        Ok(Err(e)) => {
            log_error(app, &format!("TTS 生成失败：{}", e));
            return;
        }
        Err(_) => {
            log_error(app, "TTS 线程恐慌");
            return;
        }
    };

    // 5. 播放音频
    let _ = app.emit("tts-status", "正在播放...");
    log_to_file("开始播放音频");
    eprintln!("[VoicePicker] 开始播放音频");
    let play_start = Instant::now();
    if let Err(e) = audio::player::play_once(&audio_data) {
        log_error(app, &format!("播放失败：{}", e));
    } else {
        log_to_file(&format!("播放耗时：{:?}", play_start.elapsed()));
        log_to_file(&format!("总耗时：{:?}", total_start.elapsed()));
        eprintln!("[VoicePicker] 播放耗时：{:?}", play_start.elapsed());
        eprintln!("[VoicePicker] 总耗时：{:?}", total_start.elapsed());
        let _ = app.emit("tts-status", "播放完成");
    }
}

fn log_error(app: &AppHandle, message: &str) {
    log_to_file(&format!("错误：{}", message));
    eprintln!("[VoicePicker] {}", message);
    let _ = app.emit("tts-status", format!("错误：{}", message));
}
