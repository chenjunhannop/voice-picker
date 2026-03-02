use rodio::{Decoder, Sink, OutputStream};
use std::io::Cursor;

/// 播放音频数据（阻塞直到播放完成）
pub fn play_once(audio_data: &[u8]) -> Result<(), String> {
    let (_stream, stream_handle) =
        OutputStream::try_default().map_err(|e| format!("音频输出失败：{}", e))?;

    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| format!("创建音频接收器失败：{}", e))?;

    let source = Decoder::new(Cursor::new(audio_data.to_vec()))
        .map_err(|e| format!("解码音频失败：{}", e))?;

    sink.append(source);

    // 等待播放完成
    while !sink.empty() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
