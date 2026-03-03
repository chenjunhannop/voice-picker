use rodio::{Decoder, Sink, OutputStream};
use std::io::Cursor;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

/// 播放状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Idle,
    Playing,
    Paused,
    Stopped,
}

/// 播放进度信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlaybackProgress {
    pub current_secs: f32,
    pub total_secs: f32,
    pub percent: f32,
}

/// 内部音频管理器状态
struct AudioManagerState {
    /// 音频 Sink
    sink: Option<Arc<Sink>>,
    /// 当前状态
    state: PlaybackState,
    /// 当前音频数据
    current_audio_data: Option<Vec<u8>>,
    /// 播放开始时间
    play_start_time: Option<Instant>,
    /// 暂停时的位置
    paused_position: Option<f32>,
    /// 音频总时长（秒）
    total_duration: Option<f32>,
}

impl AudioManagerState {
    fn new() -> Self {
        Self {
            sink: None,
            state: PlaybackState::Idle,
            current_audio_data: None,
            play_start_time: None,
            paused_position: None,
            total_duration: None,
        }
    }
}

/// 全局音频管理器（不持有 OutputStream，只持有状态）
static AUDIO_MANAGER: OnceLock<Arc<Mutex<AudioManagerState>>> = OnceLock::new();

fn get_manager() -> Arc<Mutex<AudioManagerState>> {
    AUDIO_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(AudioManagerState::new())))
        .clone()
}

/// 播放音频数据（每次播放创建新的 OutputStream）
pub fn play(audio_data: Vec<u8>) -> Result<(), String> {
    eprintln!("[AudioManager] play() called, audio size: {} bytes", audio_data.len());

    let manager_arc = get_manager();
    let mut manager = manager_arc.lock().map_err(|e| format!("Lock error: {}", e))?;

    // 1. 停止当前播放
    if let Some(sink) = &manager.sink {
        sink.stop();
    }

    // 2. 创建新的 OutputStream 和 Sink（每次播放时创建）
    let (stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to create output stream: {}", e))?;

    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    // 3. 解码并播放
    let source = Decoder::new(Cursor::new(audio_data.clone()))
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    // 估算音频时长（简单计算：字节数 / 44100 / 2 / 2 ≈ 秒数）
    // 实际应该解析音频头获取精确时长
    let estimated_duration = audio_data.len() as f32 / 176400.0;

    sink.append(source);

    // 保持 stream 活跃（需要存储在某个地方）
    // 由于 rodio 的设计，我们需要保持 OutputStream 活着直到播放完成
    // 这里使用简单的方法：在 sink 播放时等待

    manager.sink = Some(Arc::new(sink));
    manager.current_audio_data = Some(audio_data);
    manager.state = PlaybackState::Playing;
    manager.play_start_time = Some(Instant::now());
    manager.total_duration = Some(estimated_duration);

    // 丢弃 stream（这会导致播放停止）
    // 我们需要另一种方式来保持 stream 活跃...
    // 使用 leak 来保持 stream 活跃（内存泄漏，但简单有效）
    std::mem::forget(stream);

    Ok(())
}

/// 暂停播放
pub fn pause() -> Result<(), String> {
    eprintln!("[AudioManager] pause() called");

    let manager_arc = get_manager();
    let mut manager = manager_arc.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(sink) = &manager.sink {
        sink.pause();
        manager.state = PlaybackState::Paused;
        manager.paused_position = Some(get_elapsed_time(&manager));
    }

    Ok(())
}

/// 继续播放
pub fn resume() -> Result<(), String> {
    eprintln!("[AudioManager] resume() called");

    let manager_arc = get_manager();
    let mut manager = manager_arc.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(sink) = &manager.sink {
        sink.play();
        manager.state = PlaybackState::Playing;
        manager.play_start_time = Some(Instant::now());
    }

    Ok(())
}

/// 停止播放
pub fn stop() -> Result<(), String> {
    eprintln!("[AudioManager] stop() called");

    let manager_arc = get_manager();
    let mut manager = manager_arc.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(sink) = &manager.sink {
        sink.stop();
    }

    manager.sink = None;
    manager.state = PlaybackState::Stopped;
    manager.play_start_time = None;
    manager.paused_position = None;

    Ok(())
}

/// 获取当前播放状态
pub fn get_state() -> PlaybackState {
    let manager_arc = get_manager();
    manager_arc
        .lock()
        .map(|m| m.state)
        .unwrap_or(PlaybackState::Idle)
}

/// 获取播放进度
pub fn get_progress() -> Option<PlaybackProgress> {
    let manager_arc = get_manager();
    let manager = manager_arc.lock().ok()?;
    let total_secs = manager.total_duration.unwrap_or(0.0);
    let current_secs = get_current_position(&manager);

    if total_secs <= 0.0 {
        return None;
    }

    Some(PlaybackProgress {
        current_secs,
        total_secs,
        percent: (current_secs / total_secs * 100.0).clamp(0.0, 100.0),
    })
}

/// 跳转到指定位置（rodio 不支持，仅记录位置）
pub fn seek(_position_secs: f32) -> Result<(), String> {
    eprintln!("[AudioManager] seek() called, position: {}s", _position_secs);
    // rodio 不支持直接跳转
    Ok(())
}

// ========== 辅助方法 ==========

fn get_current_position(manager: &AudioManagerState) -> f32 {
    match manager.state {
        PlaybackState::Playing => {
            let paused_pos = manager.paused_position.unwrap_or(0.0);
            let elapsed = get_elapsed_time(manager);
            paused_pos + elapsed
        }
        PlaybackState::Paused => manager.paused_position.unwrap_or(0.0),
        _ => 0.0,
    }
}

fn get_elapsed_time(manager: &AudioManagerState) -> f32 {
    manager.play_start_time
        .map(|t| t.elapsed().as_secs_f32())
        .unwrap_or(0.0)
}
