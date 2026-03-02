#!/bin/bash
# VoicePicker 快捷键测试脚本

echo "=== VoicePicker 快捷键测试 ==="
echo ""

# 1. 启动 TTS 服务
echo "1. 启动 TTS 服务..."
pkill -f "tts_service" 2>/dev/null
nohup /Users/chenjunhan/VoicePicker/python/.venv/bin/python3 /Users/chenjunhan/VoicePicker/python/tts_service.py --preload --host 127.0.0.1 --port 8765 > /tmp/tts.log 2>&1 &
echo "   TTS 服务已启动 (PID: $!)"

echo "   等待服务启动..."
sleep 10

# 2. 测试 TTS 服务
echo ""
echo "2. 测试 TTS 服务..."
curl -s http://127.0.0.1:8765/health
echo ""

# 3. 使用 AppleScript 发送快捷键
echo ""
echo "3. 发送快捷键 Cmd+Option+X..."
osascript -e 'tell application "System Events" to key code 28 using {command down, option down}'

echo "   已发送快捷键"

# 4. 等待并查看日志
echo ""
echo "4. 等待并查看日志..."
sleep 5

echo ""
echo "=== 日志文件内容 ==="
cat /tmp/voicepicker.log 2>/dev/null || echo "日志文件不存在"

echo ""
echo "=== 完成 ==="
