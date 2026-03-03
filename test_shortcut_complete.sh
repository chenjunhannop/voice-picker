#!/bin/bash
# VoicePicker 快捷键完整测试脚本
# 需要 macOS 辅助功能权限

echo "╔════════════════════════════════════════════════════════╗"
echo "║          VoicePicker 快捷键功能测试脚本                 ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# 检查辅助功能权限
echo "1. 检查辅助功能权限..."
if osascript -e 'tell application "System Events" to keystroke "a"' 2>&1 | grep -q "error"; then
    echo "   ❌ 没有辅助功能权限"
    echo ""
    echo "   请在 系统设置 → 隐私与安全性 → 辅助功能 中添加终端应用"
    echo ""
    exit 1
else
    echo "   ✅ 辅助功能权限已授予"
fi

# 清空日志
rm -f /tmp/voicepicker.log /tmp/voicepicker_stdout.log
echo ""
echo "2. 已清空日志"

# 启动 TTS 服务
echo ""
echo "3. 启动 TTS 服务..."
pkill -f "tts_service.py" 2>/dev/null
nohup /Users/chenjunhan/VoicePicker/python/.venv/bin/python3 /Users/chenjunhan/VoicePicker/python/tts_service.py --preload --host 127.0.0.1 --port 8765 > /tmp/tts.log 2>&1 &
sleep 10
curl -s http://127.0.0.1:8765/health | python3 -m json.tool && echo "   ✅ TTS 服务已启动" || echo "   ❌ TTS 服务启动失败"

# 启动 VoicePicker 应用
echo ""
echo "4. 启动 VoicePicker 应用..."
pkill -f "voice-picker" 2>/dev/null
cd /Users/chenjunhan/VoicePicker/src-tauri
nohup ./target/release/voice-picker > /tmp/voicepicker_stdout.log 2>&1 &
sleep 3
ps aux | grep voice-picker | grep -v grep > /dev/null && echo "   ✅ 应用已启动" || echo "   ❌ 应用启动失败"

# 等待用户准备
echo ""
echo "═══════════════════════════════════════════════════════"
echo "请按以下步骤操作："
echo "1. 打开任意应用（如 Safari、备忘录等）"
echo "2. 选中一段文本"
echo "3. 按下 Cmd+Option+X"
echo "4. 等待 10 秒"
echo "═══════════════════════════════════════════════════════"
echo ""
read -p "完成后按回车继续..."

# 检查日志
echo ""
echo "═══════════════════════════════════════════════════════"
echo "测试结果："
echo "═══════════════════════════════════════════════════════"

echo ""
echo "应用日志:"
cat /tmp/voicepicker_stdout.log 2>/dev/null | tail -20

echo ""
echo "快捷键处理日志:"
if [ -f /tmp/voicepicker.log ]; then
    cat /tmp/voicepicker.log
else
    echo "日志文件不存在 - 快捷键可能未被触发"
fi

echo ""
echo "═══════════════════════════════════════════════════════"
echo "如果日志显示'快捷键被触发'但没有声音，请检查："
echo "1. TTS 服务是否运行：curl http://127.0.0.1:8765/health"
echo "2. 音频输出设备是否正确"
echo "3. 系统音量是否打开"
echo "═══════════════════════════════════════════════════════"
