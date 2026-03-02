use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 获取配置文件路径
/// macOS: ~/Library/Application Support/VoicePicker/settings.json
fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VoicePicker")
        .join("settings.json")
}

/// 确保配置目录存在
fn ensure_config_dir() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let config_dir = config_path.parent()
        .ok_or("无法获取配置目录")?;
    fs::create_dir_all(config_dir)?;
    Ok(())
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// TTS 语速 (0.5 - 2.0)
    pub tts_speed: f32,
    /// TTS 音量 (0.0 - 1.0)
    pub tts_volume: f32,
    /// 全局快捷键
    pub global_hotkey: String,
    /// 自动播放
    pub auto_play: bool,
    /// 主题
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tts_speed: 1.0,
            tts_volume: 1.0,
            global_hotkey: "Cmd+Option+X".to_string(),
            auto_play: true,
            theme: Theme::System,
        }
    }
}

impl AppSettings {
    /// 加载配置
    /// - 文件不存在：返回默认值
    /// - 文件格式错误：返回错误
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = get_config_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let settings: Self = serde_json::from_str(&content)?;
        Ok(settings)
    }

    /// 保存配置
    /// - 使用原子写入（临时文件 + 重命名）避免损坏
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        ensure_config_dir()?;

        let path = get_config_path();
        let content = serde_json::to_string_pretty(self)?;

        // 原子写入：先写临时文件，再重命名
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, &content)?;
        fs::rename(&temp_path, &path)?;

        Ok(())
    }

    /// 重置配置为默认值
    pub fn reset() -> Result<(), Box<dyn std::error::Error>> {
        let path = get_config_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
