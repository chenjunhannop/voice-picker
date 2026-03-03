#!/bin/bash
# VoicePicker 快捷键测试脚本

echo "=== VoicePicker 快捷键测试 ==="
echo ""

# 清空日志
rm -f /tmp/voicepicker.log /tmp/voicepicker_stdout.log

# 重启应用
pkill -f "voice-picker" 2>/dev/null
cd /Users/chenjunhan/VoicePicker/src-tauri
./target/release/voice-picker > /tmp/voicepicker_stdout.log 2>&1 &
sleep 3

echo "应用已启动"
echo ""
echo "请在选中文本后按下 Cmd+Option+X"
echo "然后运行：cat /tmp/voicepicker.log"
echo "或查看标准输出：cat /tmp/voicepicker_stdout.log"
echo ""

# 监控日志 30 秒
for i in $(seq 1 30); do
    if [ -f /tmp/voicepicker.log ]; then
        echo "=== 检测到日志输出 ==="
        cat /tmp/voicepicker.log
        break
    fi
    sleep 1
done
