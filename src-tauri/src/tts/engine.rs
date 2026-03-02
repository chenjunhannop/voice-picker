use serde::Deserialize;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("HTTP 请求失败：{0}")]
    HttpError(String),
    #[error("Python 服务未启动")]
    ServiceNotRunning,
    #[error("JSON 错误：{0}")]
    JsonError(String),
    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),
}

// TTS 服务状态
#[derive(Debug, Clone)]
pub struct TtsService {
    host: String,
    port: u16,
    base_url: String,
}

impl TtsService {
    pub fn new() -> Self {
        let host = "127.0.0.1".to_string();
        let port = 8765;
        let base_url = format!("http://{}:{}", host, port);

        Self { host, port, base_url }
    }

    /// 检查服务是否运行
    pub fn is_running(&self) -> bool {
        let response = reqwest::blocking::get(format!("{}/health", self.base_url));
        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    eprintln!("[TTS] 健康检查通过");
                    true
                } else if status.as_u16() == 502 {
                    // 502 表示服务正在启动中，继续等待
                    eprintln!("[TTS] 服务正在启动中 (502)...");
                    true  // 返回 true 表示服务进程正在运行
                } else {
                    eprintln!("[TTS] 健康检查失败：状态码={:?}", status);
                    false
                }
            }
            Err(e) => {
                eprintln!("[TTS] 健康检查错误：{}", e);
                false
            }
        }
    }

    /// 等待服务完全就绪（模型加载完成）
    pub fn wait_until_ready(&self, timeout_secs: u64) -> Result<(), TtsError> {
        eprintln!("[TTS] 等待服务就绪...");
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(timeout_secs) {
            let response = reqwest::blocking::get(format!("{}/health", self.base_url));
            match response {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let body = resp.text().unwrap_or_default();
                    eprintln!("[TTS] 健康检查：状态码={}, 响应={}", status, body);

                    if status == 200 {
                        // 尝试解析 JSON 检查模型状态
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            if json.get("model_loaded").and_then(|v| v.as_bool()) == Some(true) {
                                eprintln!("[TTS] 服务已就绪（模型已加载）");
                                return Ok(());
                            } else {
                                eprintln!("[TTS] 服务正在加载模型...");
                            }
                        } else {
                            eprintln!("[TTS] 服务已就绪（无法解析 JSON）");
                            return Ok(());
                        }
                    } else if status == 502 {
                        eprintln!("[TTS] 服务正在启动中 (502)...");
                    } else {
                        eprintln!("[TTS] 健康检查失败：状态码={}", status);
                    }
                }
                Err(e) => {
                    eprintln!("[TTS] 健康检查失败：{}", e);
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        Err(TtsError::ServiceNotRunning)
    }

    /// 启动服务
    pub fn start(&self) -> Result<(), TtsError> {
        // 先检查服务是否已经在运行
        if self.is_running() {
            eprintln!("[TTS] 服务已在运行");
            return Ok(());
        }

        let project_root = get_project_root();
        let python_path = get_python_path()?;
        let script_path = project_root.join("python").join("tts_service.py");

        eprintln!("[TTS] 启动 HTTP 服务：{}:{} --preload", self.host, self.port);
        eprintln!("[TTS] Python 路径：{:?}", python_path);
        eprintln!("[TTS] 脚本路径：{:?}", script_path);

        // 在后台启动 Python 服务
        Command::new(&python_path)
            .arg(&script_path)
            .arg("--preload")
            .arg("--host")
            .arg(&self.host)
            .arg("--port")
            .arg(self.port.to_string())
            // 输出 Python 服务的日志以便调试
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        // 等待服务启动（最多 30 秒）
        for i in 0..60 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if self.is_running() {
                eprintln!("[TTS] 服务已启动，耗时 {}ms", (i + 1) * 500);
                return Ok(());
            }
        }

        Err(TtsError::ServiceNotRunning)
    }

    /// 合成语音
    pub async fn synthesize(
        &self,
        text: &str,
        speed: Option<f32>,
        volume: Option<f32>,
    ) -> Result<Vec<u8>, TtsError> {
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/synthesize", self.base_url))
            .json(&serde_json::json!({
                "text": text,
                "speed": speed.unwrap_or(1.0),
                "volume": volume.unwrap_or(1.0)
            }))
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| TtsError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(TtsError::HttpError(format!("HTTP 错误：{}", error)));
        }

        #[derive(Deserialize)]
        struct SynthesizeResponse {
            success: bool,
            audio_data: String,
            error: Option<String>,
        }

        let result: SynthesizeResponse = response
            .json()
            .await
            .map_err(|e| TtsError::JsonError(format!("JSON 解析错误：{}", e)))?;

        if !result.success {
            return Err(TtsError::HttpError(result.error.unwrap_or_default()));
        }

        // 解码 Base64 音频数据
        use base64::Engine;
        let audio_data = base64::engine::general_purpose::STANDARD
            .decode(&result.audio_data)
            .map_err(|e| TtsError::HttpError(format!("Base64 解码失败：{}", e)))?;

        Ok(audio_data)
    }
}

// 全局服务实例（用于向后兼容）
static TTS_SERVICE_STARTED: AtomicBool = AtomicBool::new(false);
static TTS_SERVICE: once_cell::sync::Lazy<Arc<TtsService>> =
    once_cell::sync::Lazy::new(|| Arc::new(TtsService::new()));

/// 合成 TTS 音频（向后兼容的异步函数）
pub async fn synthesize(
    text: &str,
    speed: Option<f32>,
    volume: Option<f32>,
) -> Result<Vec<u8>, TtsError> {
    // 确保服务已启动
    if !TTS_SERVICE_STARTED.load(Ordering::Relaxed) {
        // 尝试启动服务
        let service = TTS_SERVICE.clone();
        match service.start() {
            Ok(()) => {
                TTS_SERVICE_STARTED.store(true, Ordering::Relaxed);
            }
            Err(e) => {
                eprintln!("[TTS] 服务启动失败：{}", e);
                return Err(e);
            }
        }
    }

    TTS_SERVICE.synthesize(text, speed, volume).await
}

/// 合成 TTS 音频（同步版本，用于快捷键调用）
pub fn synthesize_blocking(
    text: &str,
    speed: Option<f32>,
    volume: Option<f32>,
) -> Result<Vec<u8>, TtsError> {
    eprintln!("[TTS] synthesize_blocking 被调用");
    eprintln!("[TTS] 基础 URL: {}", TTS_SERVICE.base_url);

    // 创建不使用代理的 HTTP 客户端（避免系统代理干扰）
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(10))
        .no_proxy()  // 禁用代理，直接连接本地服务
        .build()
        .map_err(|e| {
            eprintln!("[TTS] 客户端创建失败：{}", e);
            TtsError::HttpError(e.to_string())
        })?;

    // 重试逻辑：最多重试 3 次
    for attempt in 1..=3 {
        eprintln!("[TTS] 发送合成请求 (尝试 {}/{})...", attempt, 3);

        let result = client
            .post(format!("{}/synthesize", TTS_SERVICE.base_url))
            .json(&serde_json::json!({
                "text": text,
                "speed": speed.unwrap_or(1.0),
                "volume": volume.unwrap_or(1.0)
            }))
            .send();

        match result {
            Ok(response) => {
                let status = response.status();
                eprintln!("[TTS] 响应状态码：{}", status);

                if status.as_u16() == 502 {
                    eprintln!("[TTS] 服务繁忙 (502)，等待 2 秒后重试...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    continue;
                }

                if !status.is_success() {
                    let error = response.text().unwrap_or_default();
                    eprintln!("[TTS] HTTP 错误：{} {}", status, error);
                    return Err(TtsError::HttpError(format!("HTTP 错误：{}", error)));
                }

                #[derive(Deserialize)]
                struct SynthesizeResponse {
                    success: bool,
                    audio_data: String,
                    error: Option<String>,
                }

                let result: SynthesizeResponse = response
                    .json()
                    .map_err(|e| {
                        eprintln!("[TTS] JSON 解析错误：{}", e);
                        TtsError::JsonError(format!("JSON 解析错误：{}", e))
                    })?;

                if !result.success {
                    let error_msg = result.error.unwrap_or_default();
                    eprintln!("[TTS] 合成失败：{}", error_msg);
                    return Err(TtsError::HttpError(error_msg));
                }

                // 解码 Base64 音频数据
                use base64::Engine;
                let audio_data = base64::engine::general_purpose::STANDARD
                    .decode(&result.audio_data)
                    .map_err(|e| TtsError::HttpError(format!("Base64 解码失败：{}", e)))?;

                eprintln!("[TTS] 合成成功，音频长度：{} 字节", audio_data.len());
                return Ok(audio_data);
            }
            Err(e) => {
                eprintln!("[TTS] HTTP 请求失败 (尝试 {}): {}", attempt, e);
                if attempt < 3 {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }

    Err(TtsError::ServiceNotRunning)
}

// 向后兼容的函数（保持原有接口）
fn get_project_root() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir).parent().unwrap().to_path_buf();
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn get_python_path() -> Result<PathBuf, TtsError> {
    let project_root = get_project_root();

    let venv_python = project_root
        .join("python")
        .join(".venv")
        .join("bin")
        .join("python3");

    if venv_python.exists() {
        return Ok(venv_python);
    }

    Ok(PathBuf::from("python3"))
}
