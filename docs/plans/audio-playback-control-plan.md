# VoicePicker 前端页面实现语音暂停/继续功能 - 实现计划

## 一、架构设计说明

### 1.1 当前架构分析

**现有架构:**
```
┌─────────────────────────────────────────────────────────────┐
│                    前端 (Vue 3 + TypeScript)                 │
│  - App.vue (主应用)                                          │
│  - PlayerControls.vue (播放控制 - 已有暂停/继续 UI)          │
│  - StatusIndicator.vue (状态指示器)                          │
│  - SettingsPanel.vue (设置面板)                              │
│  - HistoryList.vue (历史记录)                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Tauri IPC (invoke/listen)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              后端 (Rust + Tauri 2.0)                         │
│  - main.rs (Tauri 命令：play_audio, pause_audio, ...)        │
│  - AppState (共享状态：status, current_audio, settings)      │
│  └──────────────────────────────────────────────────────────┤
│  - tts/engine.rs (TTS 服务：synthesize)                       │
│  - audio/player.rs (音频播放：rodio 库)                       │
│  - hotkey.rs (快捷键处理)                                    │
│  - config/settings.rs (配置管理)                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Python TTS 服务 (127.0.0.1:8765)                │
│  - python/tts_service.py                                    │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 问题分析

**当前暂停/继续功能的问题:**

1. **`audio/player.rs`** 使用 `rodio::Sink` 的阻塞式播放 (`play_once` 函数)，等待播放完成才返回
2. **`main.rs`** 中的 `pause_audio()` 和 `resume_audio()` 命令是空的 TODO 实现
3. **没有全局的 `Sink` 实例引用**，无法在暂停/继续时访问
4. **缺少播放进度跟踪**机制

### 1.3 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│                    前端层                                    │
│  - PlayerControls.vue (增强：进度条、播放/暂停/继续按钮)      │
│  - 新增：AudioPlayerWindow (独立播放控制窗口)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   命令层 (Tauri Commands)                    │
│  - play_audio() - 播放音频                                    │
│  - pause_audio() - 暂停播放                                   │
│  - resume_audio() - 继续播放                                  │
│  - stop_audio() - 停止播放                                    │
│  - get_playback_status() - 获取播放状态                       │
│  - seek_audio(position: f32) - 跳转进度                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 音频管理器 (AudioManager)                    │
│  - 持有 Sink 引用 (可暂停/继续/跳转)                          │
│  - 播放状态追踪 (Idle/Playing/Paused/Stopped)                │
│  - 进度计算 (当前时间/总时长)                                 │
│  - 线程安全 (Arc<Mutex<...>>)                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rodio 音频库                                │
│  - Sink - 音频接收器 (支持 pause()/resume())                 │
│  - StreamHandle - 流句柄                                     │
│  - Decoder - 音频解码器                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、需要创建/修改的文件列表

### 2.1 后端文件 (Rust)

| 文件路径 | 操作 | 说明 |
|---------|------|------|
| `src-tauri/src/audio/manager.rs` | **新建** | 音频管理器，核心音频控制逻辑 |
| `src-tauri/src/audio/mod.rs` | **修改** | 导出新的 manager 模块 |
| `src-tauri/src/audio/player.rs` | **修改** | 重构为使用 AudioManager |
| `src-tauri/src/main.rs` | **修改** | 更新 AppState 和 Tauri 命令 |
| `src-tauri/src/lib.rs` | **修改** | 添加新的状态类型和命令 |

### 2.2 前端文件 (Vue 3 + TypeScript)

| 文件路径 | 操作 | 说明 |
|---------|------|------|
| `src/components/PlayerControls.vue` | **修改** | 增强播放控制功能 |
| `src/components/AudioPlayerWindow.vue` | **新建** | 独立播放控制窗口组件 |
| `src/stores/audioStore.ts` | **新建** | Pinia 音频状态管理 (可选) |
| `src/composables/useAudioPlayer.ts` | **新建** | 音频播放器 Composition API |

---

## 三、文件内容详细说明

### 3.1 `src-tauri/src/audio/manager.rs` (新建)

**作用:** 音频管理器，提供完整的音频播放控制功能

**核心内容:**
```rust
use rodio::{Decoder, Sink, OutputStream, StreamHandle};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 播放状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Idle,
    Playing,
    Paused,
    Stopped,
}

/// 播放进度信息
#[derive(Debug, Clone)]
pub struct PlaybackProgress {
    pub current_secs: f32,
    pub total_secs: f32,
    pub percent: f32,
}

/// 音频管理器（单例，线程安全）
pub struct AudioManager {
    /// 音频流句柄（保持设备活跃）
    stream_handle: Option<StreamHandle>,
    /// 音频 Sink
    sink: Option<Arc<Sink>>,
    /// 当前状态
    state: PlaybackState,
    /// 当前音频数据（用于重新播放）
    current_audio_data: Option<Vec<u8>>,
    /// 播放开始时间（用于计算进度）
    play_start_time: Option<Instant>,
    /// 暂停时的位置（秒）
    paused_position: Option<f32>,
    /// 音频总时长（秒）
    total_duration: Option<f32>,
}

impl AudioManager {
    /// 创建新的音频管理器
    pub fn new() -> Self { ... }

    /// 获取单例实例（全局访问）
    pub fn instance() -> Arc<Mutex<Self>> { ... }

    /// 播放音频数据
    pub fn play(&mut self, audio_data: Vec<u8>) -> Result<(), String> { ... }

    /// 暂停播放
    pub fn pause(&mut self) -> Result<(), String> { ... }

    /// 继续播放
    pub fn resume(&mut self) -> Result<(), String> { ... }

    /// 停止播放
    pub fn stop(&mut self) -> Result<(), String> { ... }

    /// 获取当前播放状态
    pub fn get_state(&self) -> PlaybackState { ... }

    /// 获取播放进度
    pub fn get_progress(&self) -> Option<PlaybackProgress> { ... }

    /// 跳转到指定位置
    pub fn seek(&mut self, position_secs: f32) -> Result<(), String> { ... }
}
```

**关键点:**
- 使用 `Arc<Mutex<AudioManager>>` 实现线程安全的单例
- 保持 `OutputStream` 和 `StreamHandle` 活跃，避免音频设备释放
- 使用 `Sink::pause()` 和 `Sink::play()` 方法实现暂停/继续
- 手动跟踪播放时间计算进度（rodio 不直接支持进度查询）

### 3.2 `src-tauri/src/audio/mod.rs` (修改)

**内容:**
```rust
pub mod player;
pub mod manager;

pub use manager::{AudioManager, PlaybackState, PlaybackProgress};
```

### 3.3 `src-tauri/src/audio/player.rs` (修改)

**修改后内容:**
```rust
use crate::audio::manager::AudioManager;

/// 播放音频（使用 AudioManager）
pub fn play_once(audio_data: &[u8]) -> Result<(), String> {
    let mut manager = AudioManager::instance().lock().unwrap();
    manager.play(audio_data.to_vec())
}

/// 暂停播放
pub fn pause() -> Result<(), String> {
    let mut manager = AudioManager::instance().lock().unwrap();
    manager.pause()
}

/// 继续播放
pub fn resume() -> Result<(), String> {
    let mut manager = AudioManager::instance().lock().unwrap();
    manager.resume()
}

/// 停止播放
pub fn stop() -> Result<(), String> {
    let mut manager = AudioManager::instance().lock().unwrap();
    manager.stop()
}
```

### 3.4 `src-tauri/src/main.rs` (修改)

**主要修改:**

1. **更新 AppState:**
```rust
// 移除 current_audio 字段，改用 AudioManager 管理
pub struct AppState {
    pub status: Mutex<TtsStatus>,
    pub settings: Mutex<AppSettings>,
}
```

2. **更新 Tauri 命令:**
```rust
#[tauri::command]
fn play_audio(state: State<'_, AppState>) -> Result<(), String> {
    // 更新状态
    *state.status.lock().unwrap() = TtsStatus::Playing;
    // 使用 AudioManager 播放
    audio::manager::play_audio_data()
}

#[tauri::command]
fn pause_audio(state: State<'_, AppState>) -> Result<(), String> {
    *state.status.lock().unwrap() = TtsStatus::Paused;
    audio::manager::pause()
}

#[tauri::command]
fn resume_audio(state: State<'_, AppState>) -> Result<(), String> {
    *state.status.lock().unwrap() = TtsStatus::Playing;
    audio::manager::resume()
}

#[tauri::command]
fn stop_audio(state: State<'_, AppState>) -> Result<(), String> {
    *state.status.lock().unwrap() = TtsStatus::Idle;
    audio::manager::stop()
}

#[tauri::command]
fn get_playback_status() -> Result<PlaybackStatus, String> {
    // 返回详细播放状态
    audio::manager::get_playback_status()
}

#[tauri::command]
fn seek_audio(position: f32) -> Result<(), String> {
    audio::manager::seek(position)
}
```

3. **添加新命令到 invoke_handler:**
```rust
.invoke_handler(tauri::generate_handler![
    get_clipboard_content,
    set_status,
    synthesize_tts,
    play_audio,
    pause_audio,
    resume_audio,
    stop_audio,
    get_playback_status,  // 新增
    seek_audio,           // 新增
    get_settings,
    save_settings,
    reset_settings
])
```

### 3.5 `src/components/PlayerControls.vue` (修改)

**主要修改:**

1. **添加继续按钮:**
```vue
<button
  @click="handleResume"
  :disabled="!isPaused"
  class="control-btn"
  title="继续"
>
  <svg viewBox="0 0 24 24" width="24" height="24">
    <path fill="currentColor" d="M8 5v14l11-7z"/>
  </svg>
</button>
```

2. **增强进度条:**
```vue
<div class="progress">
  <div class="progress-bar" @click="handleSeek">
    <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
    <div class="progress-thumb" :style="{ left: progressPercent + '%' }"></div>
  </div>
  <div class="time-display">
    <span>{{ formatTime(currentTime) }}</span>
    <span>{{ formatTime(totalTime) }}</span>
  </div>
</div>
```

3. **添加进度轮询:**
```typescript
let progressInterval: number | null = null

const startProgressPolling = () => {
  progressInterval = window.setInterval(async () => {
    const progress = await invoke<PlaybackProgress>('get_playback_progress')
    progressPercent.value = progress.percent
    currentTime.value = progress.current_secs
    totalTime.value = progress.total_secs
  }, 500)
}
```

### 3.6 `src/components/AudioPlayerWindow.vue` (新建)

**作用:** 独立的浮窗式播放控制器，可在播放时显示

**内容大纲:**
```vue
<template>
  <div class="audio-player-window" :class="{ collapsed: isCollapsed }">
    <div class="player-header">
      <span class="player-title">正在播放</span>
      <button @click="toggleCollapse" class="collapse-btn">
        <svg>...</svg>
      </button>
      <button @click="closeWindow" class="close-btn">
        <svg>...</svg>
      </button>
    </div>

    <div class="player-content">
      <!-- 进度条 -->
      <div class="progress-section">...</div>

      <!-- 控制按钮 -->
      <div class="controls-section">
        <button @click="handleStop" title="停止">...</button>
        <button @click="handlePrevious" title="上一个">...</button>
        <button @click="handlePlayPause" title="播放/暂停">...</button>
        <button @click="handleNext" title="下一个">...</button>
      </div>

      <!-- 速度控制 -->
      <div class="speed-section">...</div>
    </div>
  </div>
</template>
```

### 3.7 `src/composables/useAudioPlayer.ts` (新建)

**作用:** 封装音频播放逻辑的 Composition API

**内容大纲:**
```typescript
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface PlaybackProgress {
  current_secs: number
  total_secs: number
  percent: number
}

export function useAudioPlayer() {
  const isPlaying = ref(false)
  const isPaused = ref(false)
  const progress = ref<PlaybackProgress>({ current_secs: 0, total_secs: 0, percent: 0 })

  const play = async (text: string) => { ... }
  const pause = async () => { ... }
  const resume = async () => { ... }
  const stop = async () => { ... }
  const seek = async (position: number) => { ... }

  // 进度轮询
  let pollingInterval: number | null = null
  const startPolling = () => { ... }
  const stopPolling = () => { ... }

  onUnmounted(() => {
    stopPolling()
  })

  return {
    isPlaying,
    isPaused,
    progress,
    play,
    pause,
    resume,
    stop,
    seek
  }
}
```

---

## 四、实现步骤顺序

### 阶段一：后端核心实现 (Rust)

| 步骤 | 任务 | 预计时间 |
|-----|------|---------|
| 1.1 | 创建 `audio/manager.rs` - AudioManager 基础结构 | 30 分钟 |
| 1.2 | 实现 `AudioManager::new()` 和单例模式 | 15 分钟 |
| 1.3 | 实现 `play()` 方法 - 音频播放 | 30 分钟 |
| 1.4 | 实现 `pause()` / `resume()` / `stop()` 方法 | 30 分钟 |
| 1.5 | 实现 `get_progress()` - 进度计算 | 30 分钟 |
| 1.6 | 实现 `seek()` - 进度跳转 | 30 分钟 |
| 1.7 | 更新 `audio/mod.rs` 导出模块 | 5 分钟 |
| 1.8 | 更新 `main.rs` Tauri 命令 | 30 分钟 |
| 1.9 | 编译测试和调试 | 30 分钟 |

**阶段一小计：约 3.5 小时**

### 阶段二：前端基础实现 (Vue 3)

| 步骤 | 任务 | 预计时间 |
|-----|------|---------|
| 2.1 | 修改 `PlayerControls.vue` - 添加继续按钮 | 20 分钟 |
| 2.2 | 增强进度条组件 - 可点击跳转 | 30 分钟 |
| 2.3 | 实现进度轮询逻辑 | 30 分钟 |
| 2.4 | 添加时间显示 (当前时间/总时长) | 15 分钟 |
| 2.5 | 创建 `useAudioPlayer.ts` composable | 40 分钟 |
| 2.6 | 集成 composable 到 PlayerControls | 20 分钟 |
| 2.7 | 前端样式优化 | 30 分钟 |

**阶段二小计：约 3 小时**

### 阶段三：高级功能

| 步骤 | 任务 | 预计时间 |
|-----|------|---------|
| 3.1 | 创建 `AudioPlayerWindow.vue` 浮窗组件 | 40 分钟 |
| 3.2 | 实现浮窗拖拽功能 | 30 分钟 |
| 3.3 | 添加键盘快捷键支持（空格暂停/继续）| 30 分钟 |
| 3.4 | 实现播放历史快速切换 | 30 分钟 |
| 3.5 | 添加播放速度控制 | 20 分钟 |
| 3.6 | 集成到主应用 | 20 分钟 |

**阶段三小计：约 2.5 小时**

### 阶段四：测试与优化

| 步骤 | 任务 | 预计时间 |
|-----|------|---------|
| 4.1 | 后端单元测试 | 30 分钟 |
| 4.2 | 前端组件测试 | 30 分钟 |
| 4.3 | 集成测试（暂停/继续/跳转）| 40 分钟 |
| 4.4 | 性能优化（内存、CPU） | 30 分钟 |
| 4.5 | Bug 修复和边界情况处理 | 40 分钟 |
| 4.6 | 文档更新 | 20 分钟 |

**阶段四小计：约 3 小时**

**总计预计时间：约 12 小时**

---

## 五、技术难点和解决方案

### 5.1 难点一：rodio 进度追踪

**问题:** rodio 库不直接提供播放进度查询 API

**解决方案:**
```rust
// 记录播放开始时间
play_start_time: Option<Instant>,

// 暂停时记录位置
paused_position: Option<f32>,

// 计算当前进度
fn get_progress(&self) -> Option<PlaybackProgress> {
    match self.state {
        PlaybackState::Playing => {
            let elapsed = self.play_start_time?
                .elapsed()
                .as_secs_f32();
            let position = self.paused_position.unwrap_or(0.0) + elapsed;
            Some(PlaybackProgress {
                current_secs: position,
                total_secs: self.total_duration?,
                percent: position / self.total_duration? * 100.0,
            })
        }
        PlaybackState::Paused => Some(PlaybackProgress {
            current_secs: self.paused_position?,
            total_secs: self.total_duration?,
            percent: self.paused_position? / self.total_duration? * 100.0,
        }),
        _ => None,
    }
}
```

### 5.2 难点二：音频设备生命周期

**问题:** `OutputStream` 被释放后音频设备会停止工作

**解决方案:**
```rust
// 在 AudioManager 中保持 stream_handle 活跃
pub struct AudioManager {
    stream_handle: Option<StreamHandle>,
    // ...
}

impl AudioManager {
    fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            stream_handle: Some(stream_handle),
            // ...
        }
    }
}
```

### 5.3 难点三：线程安全

**问题:** Tauri 命令在多线程调用，需要线程安全的 AudioManager

**解决方案:**
```rust
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static AUDIO_MANAGER: Lazy<Arc<Mutex<AudioManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(AudioManager::new())));

pub fn get_manager() -> Arc<Mutex<AudioManager>> {
    AUDIO_MANAGER.clone()
}
```

---

## 六、API 设计

### 6.1 后端 Tauri 命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `play_audio` | - | `Result<(), String>` | 播放当前音频 |
| `pause_audio` | - | `Result<(), String>` | 暂停播放 |
| `resume_audio` | - | `Result<(), String>` | 继续播放 |
| `stop_audio` | - | `Result<(), String>` | 停止播放 |
| `get_playback_status` | - | `Result<PlaybackStatus, String>` | 获取播放状态 |
| `seek_audio` | `position: f32` | `Result<(), String>` | 跳转到指定位置（秒） |
| `synthesize_and_play` | `text: String, speed: f32, volume: f32` | `Result<(), String>` | 合成并播放 |

### 6.2 前端事件

| 事件 | 负载 | 说明 |
|------|------|------|
| `tts-status` | `string` | TTS 状态更新 |
| `playback-progress` | `PlaybackProgress` | 播放进度更新 |
| `playback-complete` | `{ text: string }` | 播放完成 |

---

## 七、文件结构

```
VoicePicker/
├── src-tauri/
│   └── src/
│       ├── audio/
│       │   ├── mod.rs          # 模块导出
│       │   ├── manager.rs      # 音频管理器 (新建)
│       │   └── player.rs       # 音频播放器 (修改)
│       ├── main.rs             # Tauri 命令 (修改)
│       └── lib.rs              # 状态类型 (修改)
├── src/
│   ├── components/
│   │   ├── PlayerControls.vue      # 播放控制 (修改)
│   │   └── AudioPlayerWindow.vue   # 浮窗组件 (新建)
│   ├── composables/
│   │   └── useAudioPlayer.ts       # 音频 Composable (新建)
│   └── stores/
│       └── audioStore.ts           # 音频 Store (可选新建)
└── docs/
    └── plans/
        └── audio-playback-control-plan.md  # 本计划文档
```
