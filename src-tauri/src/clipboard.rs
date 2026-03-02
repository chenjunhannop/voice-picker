use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use std::process::Command;

/// 获取剪贴板文本
pub fn get_clipboard_text(app: &AppHandle) -> String {
    app.clipboard().read_text().unwrap_or_default()
}

/// 获取选中的文本（通过模拟 Cmd+C 复制）
pub fn get_selected_text(app: &AppHandle) -> String {
    // 1. 保存当前剪贴板内容
    let old_clipboard = get_clipboard_text(app);

    // 2. 模拟 Cmd+C 复制选中的文本
    simulate_cmd_c();

    // 3. 等待复制操作完成
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 4. 读取新的剪贴板内容（选中的文本）
    let selected_text = get_clipboard_text(app);

    // 5. 恢复原有剪贴板内容
    let _ = app.clipboard().write_text(&old_clipboard);

    // 6. 返回选中的文本
    selected_text
}

/// 模拟 Cmd+C 按键
#[cfg(target_os = "macos")]
fn simulate_cmd_c() {
    // 使用 osascript 模拟按键
    let _ = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to keystroke "c" using command down"#
        ])
        .output();
}

#[cfg(not(target_os = "macos"))]
fn simulate_cmd_c() {
    // 其他平台暂不支持
}
